# Packaging Plan (Offline-First)

## Aggregated payload layout (installed)

Target install root (platform-specific):

- Windows: `%ProgramFiles%\\NeQST Office 1-2-3\\`
- Linux: `/opt/neqst-office-123/`
- macOS: `/Applications/NeQST Office 1-2-3.app/Contents/`

Recommended internal layout:

- `bin/neqst_core` (or `neqst_core.exe`)
- `frontend/` (Flutter bundle)
- `embedded/libreoffice/26.2.1/program/` (LibreOfficeKit + LO core runtime)
- `config/compatibility_matrix.json` (copied from `ProjectRoot/core/compatibility_matrix.json`)

## Runtime wiring

- `NEQST_LOK_INSTALL_PATH` should point to `embedded/libreoffice/26.2.1/program/`
- `NEQST_LOK_LIBRARY_PATH` should point to the platform’s LibreOfficeKit shared library inside that program directory

## Build-time staging (template)

- Stage Rust core release binary into `stage/bin/`
- Stage Flutter desktop build into `stage/frontend/`
- Stage LibreOffice runtime into `stage/embedded/libreoffice/26.2.1/program/`
- Copy `ProjectRoot/core/compatibility_matrix.json` into `stage/config/compatibility_matrix.json`

## Update strategy (template)

- Installer writes new runtime to a versioned directory: `embedded/libreoffice/<new_version>/program/`
- Installer updates a single configuration pointer (or env file) to mark the candidate core
- Core attempts candidate init; on failure it rolls back to stable and emits a forensic NDJSON event
