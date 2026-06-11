NeQST Office 1-2-3 — Installer Scaffold

This directory contains packaging templates to build offline-first installers that aggregate:

1) the Rust core binary (NeQST orchestrator)
2) the Flutter production bundle (desktop/web)
3) the embedded LibreOffice 26.2.1 runtime directory (detached, dynamically loaded)

Templates are intentionally minimal and must be adapted to the target CI environment.
