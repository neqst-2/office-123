# LEGAL COMPLIANCE — NeQST Office 1-2-3

This document describes how NeQST Office 1-2-3 aggregates the LibreOffice core and LibreOfficeKit in a way intended to comply with the Mozilla Public License v2.0 (MPL 2.0) and related upstream licenses.

## 1) Packaging model: detached aggregation

- The LibreOffice runtime (including LibreOfficeKit shared libraries and program files) is bundled as a detached, modular directory.
- The NeQST Rust core dynamically loads LibreOfficeKit at runtime using a runtime-resolved path (no static linking to LibreOfficeKit).
- NeQST’s own components (Rust orchestrator, bridge protocol, Flutter UI, graph logic, and cryptographic systems) are distributed separately from the LibreOffice sources and remain logically independent.

## 2) No modification of LibreOffice binaries

- The shipped LibreOffice binaries are intended to be unmodified upstream builds.
- The installer places the upstream binaries into a versioned folder (`embedded/libreoffice/<version>/program/`) so that updates can be staged without overwriting previous stable versions.
- If a new LibreOffice runtime fails to initialize, NeQST rolls back to the stable version and records an NDJSON forensic event to the external forensic directory.

## 3) License texts and attribution

- The distribution must include the applicable upstream license texts for LibreOffice and its components.
- The distribution must include attribution notices as required by upstream.
- The installer scaffolds in `ProjectRoot/installer/` assume a build pipeline will copy upstream LICENSE/NOTICE files into the final package.

## 4) MPL 2.0 obligations (operational intent)

NeQST’s intent is to satisfy MPL 2.0 obligations by:

- treating LibreOffice/LibreOfficeKit as a separate aggregated component
- avoiding static linking of LibreOfficeKit into NeQST binaries
- keeping NeQST source files separate from upstream LibreOffice sources

If NeQST ever modifies MPL-covered LibreOffice source files, those modifications must be made available under MPL 2.0 as required. This project’s packaging plan is designed to avoid that requirement by default by shipping unmodified upstream binaries.

## 5) Trademarks and naming

LibreOffice is a trademark of The Document Foundation. Any use of trademarks must follow upstream trademark policies.

## 6) Disclaimer

This document is a technical compliance guide and does not constitute legal advice. A formal legal review is required before public distribution.
