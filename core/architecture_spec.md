# NeQST Office 1-2-3 — Architecture Specification (Phase 0→Enterprise Readiness)

This document extends the long-term architecture blueprint of NeQST Office 1-2-3 with three roadmap pillars:

- NeQST Forms 1-2-3 (secure data collection)
- Enterprise scalability and identity management
- Unified Communications (UC)

It also codifies the formats strategy for ODF vs. OOXML in the context of a graph-native office suite.

## 0. Non-Negotiables (Security & Layering)

### 0.1 Workspace boundaries

- Production code and assets live strictly in `ProjectRoot/`.
- Diagnostics and heavy forensic dumps must be routed to the sibling directory `../ForensicData` (never inside `ProjectRoot/`).

### 0.2 Zero-Trust layered paradigm

NeQST is structured as strict layers with explicit validation boundaries:

1. **Boundary Layer (UI / Flutter):** no direct DB sockets, no raw filesystem privileges, no implicit trust of inbound messages.
2. **Orchestrator Layer (Controller):** validates and signs commands, enforces policies, schedules tasks, and runs continuous self-audit.
3. **Service/Module Layer:** mail, calendar, docs, forms ingestion, UC services (each isolated, provider-specified).
4. **Security Layer:** keystore abstraction, key lifecycle, hybrid PQC cryptography, and signature validation.

### 0.3 Cryptographic baseline (PQC-ready)

- Hybrid key establishment: **ML‑KEM (Kyber1024)** + **X25519** (hybrid mode) to derive symmetric session keys.
- Classical signatures: **Ed25519** (migration path to PQ signatures later).
- Token lifetime: ≤ 30 minutes; clock skew tolerance ± 30 seconds.
- Storage: record-level encryption with authenticated encryption (AEAD) and explicit envelope metadata; secrets are never hardcoded or stored as raw env vars.

### 0.4 Continuous self-audit & graceful degradation

The orchestrator runs a periodic (~60s) self-audit loop:

- scheduler drift / liveness checks
- queue backpressure
- configuration signature validity
- token expiry correctness and clock drift

On anomaly, the suite degrades gracefully (read-only mode, throttled retries, explicit user-visible error banners).

## 1. NeQST Forms 1-2-3 — Framework Strategy

NeQST Forms introduces secure, auditable data collection without breaking the sovereignty model. Forms can be hosted publicly while submissions remain end-to-end protected and only become usable when ingested into a trusted local NeQST environment.

### 1.1 Detached Cloud-Frontend API (isolated surface)

**Goal:** expose a minimal public submission surface that cannot escalate into the internal office runtime.

**Components**

- **Public Forms Web Frontend** (static or minimal SSR):
  - renders forms, client-side validation, accessibility
  - no privileged capabilities; no access to internal storage
- **Forms Submission API (Cloud Edge Service)**:
  - receives submissions and attachments
  - performs immediate cryptographic sealing (never persist plaintext)
  - stores encrypted payloads in an append-only store (object storage + immutable metadata index)
- **Delivery Channel** (pull or push):
  - pull via authenticated client sync (preferred for air-gapped / sovereign modes)
  - optional push via message queue for enterprise deployments

**Isolation controls**

- strict input validation and size limits at the edge
- explicit rate limiting + abuse detection
- no dynamic code execution, no server-side templating of user data
- content-type enforcement for attachments

### 1.2 Cryptographic workflow (public cloud sealing → E2EE transport → local decryption)

NeQST Forms supports two operational modes. Both guarantee that the cloud environment is not a trusted party for plaintext retention.

#### Mode A (recommended): client-side encryption (true E2EE at rest and in transit)

1. Browser generates an ephemeral submission key.
2. Browser performs hybrid key establishment using the organization’s published key material:
   - ML‑KEM + X25519 → shared secret
3. Browser encrypts payload + attachments with AEAD (e.g., XChaCha20‑Poly1305 or AES‑GCM) and uploads ciphertext.
4. Cloud stores ciphertext and minimal routing metadata.
5. NeQST Core ingests ciphertext locally, performs decryption inside the Security Layer, then writes decrypted structured data into SurrealDB.

#### Mode B (explicitly requested): cloud-side immediate public-key sealing (no plaintext persistence)

1. Browser submits plaintext over a hardened TLS channel to the cloud edge.
2. Cloud edge immediately encrypts the payload using recipients’ public keys:
   - hybrid establishment ML‑KEM + X25519 (server side)
   - AEAD sealing of payload and attachments
3. Cloud discards plaintext buffers after sealing; persistence is ciphertext-only.
4. NeQST Core pulls ciphertext; local decryption occurs inside the Security Layer; SurrealDB stores encrypted-at-rest fields or decrypted fields depending on the configured policy.

**Security note:** Mode B is operationally acceptable only if the cloud edge is treated as high-risk and is hardened accordingly (memory zeroization, minimal logs, no request body retention, strict secret-less design).

### 1.3 Local ingestion into SurrealDB (Calc/Base interoperability)

Forms ingestion becomes first-class graph data:

- `form_definition` (template, version, validation rules)
- `form_submission` (timestamp, origin, ciphertext envelope metadata)
- `attachment_blob` (ciphertext reference, content hash, MIME type)
- `linked_to` edges to:
  - `document_meta` (generated spreadsheets/reports)
  - `mail` (notifications, routing)
  - `calendar_event` (appointments derived from submissions)

This enables Calc and Base workflows without “export/import” fragmentation: submissions are graph-native and can be rendered into documents via deterministic transformations.

### 1.4 UI security indicators (mandatory)

Any UI surface that deploys or manages forms must show explicit security posture:

- **Deployment banner:** “Public Cloud Surface” vs “Sovereign Self-Hosted” vs “Air-Gapped”.
- **Encryption badge:** “Client‑Side E2EE” (Mode A) vs “Cloud Sealing” (Mode B).
- **Data residency warning:** required when submissions may traverse regions.
- **Key status indicator:** organization key validity, rotation date, and trust anchor.
- **Audit trail button:** shows immutable submission logs (timestamps, correlation IDs, signature validity).

If keys are invalid/expired, the UI must block deployment and show a high-signal warning (no silent fallback).

### 1.5 Road blocks (Forms)

- Browser PQC availability and performance constraints (hybrid mode is heavier).
- Attachment handling at scale (streaming encryption, chunking, dedup via hashes).
- Regulatory constraints (retention policies, right-to-delete vs append-only audit logs).
- Offline/low-connectivity submission modes (queueing + eventual upload).

## 2. Enterprise Scalability & Identity Management

Enterprise mode extends NeQST from a single embedded DB to a hybrid topology that supports teams of 5,000+ users with consistent access control across records and edges.

### 2.1 Data topology: embedded → clustered SurrealDB

NeQST supports three deployment tiers:

1. **Sovereign Local (default):** embedded SurrealDB in Rust; best for single user / small team.
2. **Hybrid Sync:** embedded SurrealDB per client + a centralized/clustered SurrealDB for shared datasets.
3. **Centralized/Clustered:** SurrealDB cluster as primary; clients operate as cache + offline queue.

**Switching/sync strategy**

- Maintain a stable logical schema across tiers; environment chooses storage backend.
- Sync is modeled as a signed command stream:
  - client submits signed mutations
  - server validates scopes and permissions
  - server emits signed state deltas for clients
- Conflict handling:
  - deterministic merge rules per domain (mail, calendar, docs, forms)
  - explicit conflict nodes in graph for human resolution when required

### 2.2 Identity integration: Samba 4 AD, LDAP, OpenID Connect

NeQST Identity must support both directory-native enterprises and modern SSO.

**Identity sources**

- **Samba 4 AD / LDAP:**
  - group membership and attributes mapped into NeQST “principals”
  - periodic sync with signed snapshots
- **OpenID Connect (OIDC):**
  - primary interactive login
  - short-lived tokens and refresh strategies within policy limits

**Principals**

- `user` (person)
- `service_account` (automation)
- `group` (team/unit)
- `role` (permission bundle)

### 2.3 Authorization blueprint: record-level + edge-level permissions

SurrealDB permissions must apply not only to nodes but also to edges to prevent graph traversal escalation.

**Core idea**

- Every record and relation is tagged with:
  - `tenant_id`
  - `owner_id`
  - `classification` (public/internal/confidential/restricted)
  - `acl` (explicit allow/deny lists by principal/group)

**SurrealDB mechanisms**

- Use **SCOPE** definitions for token issuance and scoped access.
- Use token-based access controls and short-lived sessions.
- Enforce read/write permissions on:
  - node tables (`mail`, `calendar_event`, `document_meta`, `form_submission`)
  - relation tables (`sent_by`, `linked_to`, future UC relations)

**Edge-level rule example (conceptual)**

- A user may read a `mail` record but may only traverse `linked_to` edges if:
  - the edge is visible to the same principal
  - the destination node is permitted
  - the traversal does not cross tenant boundaries

**Scaling to 5,000+ users**

- Precompute and cache group→role expansions at the orchestrator layer.
- Use indexed permission fields for fast filtering (`tenant_id`, `owner_id`, `classification`).
- Prefer “deny-by-default” and explicit grants for high-value datasets.

### 2.4 Road blocks (Enterprise)

- Keeping permission evaluation fast for deep traversals (edge permissions are non-negotiable).
- Multi-tenant isolation correctness (no cross-tenant graph leakage).
- Offline mode vs. enterprise policy (selective sync and wipe-on-revoke).
- Key lifecycle at org scale (rotation without breaking access to historical ciphertext).

## 3. Unified Communications (UC) Perspective

UC is treated as a peer domain to Mail/Calendar, not an external plugin. The core requirement is secure chat and future calling inside the same tab-centric Lotus-Moderne workspace.

### 3.1 Matrix protocol integration (Rust SDK)

**Goal:** decentralized secure team chat with enterprise governance options.

**Core components**

- **UC Service (Rust):**
  - Matrix SDK integration (room sync, encryption state, device verification)
  - policy gatekeeping (allowed homeservers, federation rules)
  - event stream normalization into NeQST’s internal command/event format
- **SurrealDB storage (optional/controlled):**
  - store only permitted metadata (room IDs, member lists, message indexes)
  - message bodies remain encrypted and/or stored in Matrix crypto store as required

**Security posture**

- Treat Matrix as untrusted input: every event is validated and normalized.
- Device verification UX is mandatory (no silent “verified” states).
- Audit trail: correlation IDs for every UC command and state transition.

### 3.2 WebRTC and SIP/PBX bridges (3CX class integration)

**Goal:** voice/video calls and PBX integration without breaking zero-trust boundaries.

**Interface requirements in Flutter**

- UC must run as tabs/panels inside the unified workspace:
  - chat room tabs
  - call session tabs (WebRTC surfaces)
  - side panel for participant list, transcripts, and recording indicators
- Mandatory call security indicators:
  - E2EE enabled/disabled
  - recording active indicator
  - SIP bridge indicator (calls leaving the E2EE domain)

**Bridge blueprint**

- **WebRTC module:** peer connections, ICE handling, media permissions, device enumeration (policy controlled).
- **SIP/PBX bridge service:** SIP signaling abstraction and policy layer:
  - rules for allowed destinations
  - logging and consent prompts
  - optional integration targets (e.g., 3CX) through a dedicated gateway service

### 3.3 Road blocks (UC)

- Secure device verification UX at scale (Matrix key trust management).
- Mixed-domain calls (WebRTC E2EE vs SIP bridge) with correct user warnings.
- Media privacy: device permission gating and safe defaults (muted/disabled on join).
- Storage and compliance: retention policies, legal hold, and per-tenant controls.

## 4. Formats Battleground: ODF vs OOXML

### 4.1 Import/export guarantees

- **Headless LibreOfficeKit** is the compatibility engine for:
  - loss-minimized import/export of Microsoft OOXML
  - stable rendering parity for legacy documents

### 4.2 Graph-native superpower: ODF unlocks reactive linking

NeQST’s differentiated capability is not “another file viewer”; it is a graph-reactive office runtime:

- ODF documents can embed stable semantic anchors (cells, paragraphs, ranges, fields).
- NeQST uses these anchors as first-class graph endpoints:
  - `linked_to` edges connect `mail`, `calendar_event`, `document_meta`, `form_submission`, and UC artifacts.

**Policy**

- OOXML remains fully supported for interoperability.
- Only ODF is guaranteed to unlock:
  - loss-free semantic anchors
  - deterministic, stable `context_anchor` addressing for `linked_to` edges
  - long-term reactive automation without vendor-specific drift

### 4.3 Road blocks (Formats)

- Stable anchor definition across editing operations (renames, row/column shifts).
- Bidirectional mapping complexity when importing OOXML into ODF semantics.
- Testing matrix size: rendering parity across desktop and web shells.

## 5. Reference Flows (text diagrams)

### 5.1 Forms ingestion into spreadsheet (graph-native)

1. Public form submission (Mode A or B)
2. Encrypted payload stored in cloud append-only store
3. NeQST Core pulls ciphertext → decrypts locally
4. Write `form_submission` node + attachments
5. Generate/Update `document_meta` for the spreadsheet report
6. Create `linked_to` edge with `context_anchor` pointing to a sheet range/cell

### 5.2 Unified workspace view (single shell, multi-domain)

- Dashboard tab (pinned)
- Mail tab (pinned) → action opens “Linked Spreadsheet” document tab
- Calendar tab (pinned)
- UC tabs (future) for Matrix rooms and call sessions
