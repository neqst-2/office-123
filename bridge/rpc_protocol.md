# NeQST Local RPC Protocol (JSON-RPC-like over WebSocket)

This protocol defines the message boundary between the Flutter frontend (untrusted boundary layer) and the Rust core orchestrator (policy-enforcing controller + service layer).

Transport:

- WebSocket: `ws://127.0.0.1:9001/ws`
- Payload encoding: UTF-8 JSON text frames

Remote authentication:

- LocalMode (loopback `127.0.0.1`): no token required
- RemoteMode (non-loopback): token required via query or header:
  - `ws://<host>:9001/ws?token=<PASETO_V4_PUBLIC>`
  - or header `x-neqst-remote-token: <PASETO_V4_PUBLIC>`

PASETO token format (v4.public):

- String envelope: `v4.public.<payload_b64url>.<sig_b64url>`
- Signature input is PASETO PAE (pre-auth encoding) of:
  - `header = "v4.public."`
  - `payload = <raw JSON bytes>`
  - `footer = ""` (not used in this phase)
  - `implicit_assertion = ""` (not used in this phase)
- Payload JSON claims (minimum):
  - `iat` (unix seconds)
  - `exp` (unix seconds)
  - `aud` = `neqst-remote`
  - `nonce` (base64url)

## Envelope formats

### Request

```json
{
  "id": "uuid-string",
  "method": "domain:action",
  "params": {}
}
```

### Response

```json
{
  "id": "uuid-string",
  "result": {},
  "error": null
}
```

### Error object

```json
{
  "id": "uuid-string",
  "result": null,
  "error": {
    "code": -32000,
    "message": "db_error",
    "data": {}
  }
}
```

## Error codes

- `-32700` parse_error
- `-32601` method_not_found
- `-32602` invalid_params
- `-32000` db_error

## Methods

### pim:get_unread_mails

Request params:

```json
{ "limit": 50 }
```

Result:

```json
{
  "items": [
    {
      "id": "...",
      "message_id": "...",
      "subject": "...",
      "date_received": "...",
      "is_read": false,
      "size": 123,
      "e2ee_protected": true,
      "body_text_raw": "BASE64(NONCE+CIPHERTEXT+TAG)",
      "body_text_plain": "decrypted preview"
    }
  ]
}
```

### pim:ingest_mail

Request params:

```json
{
  "message_id": "<rfc822-message-id>",
  "subject": "Hello",
  "body_text": "plaintext",
  "body_html": "<p>plaintext</p>",
  "date_received": "2026-06-10T09:30:00+02:00",
  "is_read": false,
  "size": 1234
}
```

Result:

```json
{ "created": [ { "id": "...", "message_id": "...", "encryption_metadata": { "e2ee": true } } ] }
```

### pim:get_agenda

Request params:

```json
{ "start": "2026-06-10T00:00:00+02:00", "end": "2026-06-11T00:00:00+02:00" }
```

Result:

```json
{ "items": [ { "id": "...", "title": "...", "start_time": "...", "end_time": "...", "location": "..." } ] }
```

### doc:open_document

Request params:

```json
{ "path": "relative/path/to/file.ods", "filename": "file.ods", "mime_type": "application/vnd.oasis.opendocument.spreadsheet" }
```

Immediate response (non-blocking):

```json
{ "status": "queued", "task_id": "task-<unix>-<counter>" }
```

### task:progress (notification)

Server-to-client notification emitted over the same WebSocket connection while a background worker processes the queued task.

Envelope:

```json
{
  "method": "task:progress",
  "params": {
    "progress": { "Processing": { "task_id": "task-...", "percentage": 75, "status_message": "📄 ..." } }
  }
}
```

On completion, the notification includes the final metadata payload:

```json
{
  "method": "task:progress",
  "params": {
    "progress": { "Completed": { "task_id": "task-...", "result_summary": "document_processed" } },
    "result": {
      "meta": { "id": "...", "filename": "...", "storage_path": "...", "mime_type": "...", "file_size": 0 },
      "lok": { "available": true, "parts": 1, "structure_hash": "..." },
      "is_graph_enhanced": true
    }
  }
}
```

<<<<<<< HEAD
### sys:get_node_status

Returns the current health matrix of the local NeQST system node.

Request params:

```json
{}
```

Result:

```json
{
  "db_status": "connected",
  "lok_status": "ready",
  "lok_version": "26.2.1",
  "crypto_status": "active",
  "queue_backlog": 0,
  "active_peers": 0
}
```

=======
>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
### sync:generate_remote_token

LocalMode only. Generates a one-time remote access token for pairing.

Request params:

```json
{ "ttl_seconds": 900 }
```

Result:

```json
{ "token": "v4.public.<payload_b64url>.<sig_b64url>", "expires_unix": 1234567890 }
```

### sync:list_peers

Returns the current in-memory peer ledger (remote WebSocket sessions).

Request params:

```json
{}
```

Result:

```json
{ "peers": [ { "addr": "1.2.3.4:55555", "mode": "Remote", "connected_at_unix": 1234567890 } ] }
```

### sync:push_changes

Applies an atomic batch of document_meta upserts and linked_to edges.

Request params:

```json
{
  "documents": [ { "storage_path": "docs/a.ods", "filename": "a.ods", "mime_type": "...", "file_size": 10 } ],
  "edges": [ { "from": "mail:demo", "to": "document_meta:xyz", "context_anchor": "sheet:cell=R1C1" } ]
}
```

Result:

```json
{ "applied": [ { "documents": 1, "edges": 1 } ] }
```

### sync:pull_delta

Computes a delta set based on a timestamp.

Request params:

```json
{ "since": "2026-06-10T00:00:00Z" }
```

Result:

```json
{ "documents": [ ... ], "edges": [ ... ] }
```
