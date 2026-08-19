# Ralplan — Analyze-72 / QUEUE #37

## Principles
1. Ceremony import uses file or stdin; `--secret-hex` stays demo-only (C1) and is not removed.
2. Fail-closed parse: trim ends → exactly 64 hex digits; no `0x` / inner whitespace / raw bytes.
3. Do not touch node `identity create` / `identity rotate`.
4. Never print the secret (errors name the flag/path, not the body).
5. `PATH=-` on a TTY fails closed (no interactive hang).

## Decision drivers
- QUEUE done-when: register **and** rotate read secret from file or stdin, not argv.
- `--secret-hex` is demo-only and stays (C1).
- aira-cli is bin-only; helper must still be unit-testable.

## Viable options
- **A (chosen):** `aira-cli` module `tenant_secret.rs` + `--secret-hex-file` on register/rotate. Pros: I/O stays in CLI; object persist unchanged. Cons: tests via bin `#[cfg(test)]`.
- **B (rejected):** export `parse_secret_hex` from `aira-object`, I/O in CLI — node parse stays private; B1 errors / 4KiB / TTY are CLI; argv parser frozen (C1). Not “stdin in object”.
- **C (rejected, C3):** also `identity create --secret-hex-file` — Out.

## API / CLI
- `parse_seed_hex(&str) -> Result<[u8;32]>`
- `load_from_reader(impl Read, is_tty: bool) -> Result<[u8;32]>` — **always** `take(4097)` inside (file and stdin).
- `load_seed_hex_file(path: &str)`: `-` → `stdin().is_terminal()` then `load_from_reader`; else file `take(4097)`. File named `-` via `./-`.
- `resolve_tenant_signing(secret_hex, secret_hex_file) -> Result<SigningKey>` for **both** Register and Rotate: file XOR hex XOR generate; both Some → err even without clap.
- `--secret-hex-file` `conflicts_with = "secret_hex"`. `--secret-hex` parse frozen (`trim` + decode).
- Errors: flag/path only; no `from_utf8_lossy`; no `{e}` wrapping raw/body. File parser must not `invalid …: {e}`.

## Pre-mortem
1. `PATH=-` on TTY hang → `is_terminal` fail-closed.
2. anyhow Display leaks seed → tests assert seed absent.
3. File >4KiB / 4097th byte → prefix parsed as seed → `take(4097)` + oversize, no prefix parse.

## Tests (`cargo test -p aira-cli`; no aira-object API change)
Parse/load in `tenant_secret.rs`; clap in `main.rs` `#[cfg(test)]` (`Cli` private). Add `[dev-dependencies] tempfile = { workspace = true }` to `aira-cli`.
1. parse: `"aa"*32+"\n"` ok; leading/trailing space ok; inner space fail; `0x` fail; 63/65 fail; empty fail; 32 raw bytes fail; invalid hex digit fail; non-UTF-8 fail without lossy
2. file: temp 64-hex → bytes match; missing file err; oversize err no partial
3. `load_from_reader`: `is_tty=true` → `refusing to read secret from a TTY`; `is_tty=false` + 64-hex → ok
4. clap XOR: `Cli::try_parse_from` **Err** for register **and** rotate (required `--csu-id`/`--publisher`)
5. `resolve_tenant_signing`: file XOR hex XOR generate; both Some → err
6. persist: seed → `save_csu_tenant_signing` **and** `rotate_csu_tenant_signing` → pubkey matches
7. every err Display: seed/hex body absent; flag or path present

## Docs
Prefer `--secret-hex-file` / `-` in clap help + `docs/crypto.md`. Drop #37 from `docs/local-node.md` / `docs/peer-link.md` non-goals. Do **not** invent QUEUE #38 in “наступний OPEN”. Living Spec includes rotate import + oversize.

## Architect (WATCH folded)
take(4097) inside `load_from_reader`; `load_from_reader` for tests; shared `resolve_tenant_signing`; no secret in errors; clap `try_parse_from`; tempfile; `-` is stdin.

## Critic
**APPROVE** (a3d3bf5f) after Architect CLEAR (10ad6d99). Residual accepted: argv `--secret-hex` stays; TTY only for `-`; argv `{e}` freeze.

## Done when
register **and** rotate import from file/stdin; tests above; docs.
