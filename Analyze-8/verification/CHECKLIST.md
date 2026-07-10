# Verification Checklist — Analyze-8

## Immutability

- [x] Немає змін у `Manifesto etc/`
- [x] Немає змін у `Meditation_About/`

## Epic 0 / 1

- [x] README, LICENSE*, CONTRIBUTING, SECURITY, CHANGELOG
- [x] Cargo workspace resolver=2, members declared
- [x] rust-toolchain.toml, rustfmt.toml, clippy.toml
- [x] `.github/workflows/ci.yml`
- [x] `specs/book-0.md` … `book-v.md`
- [x] `specs/specification-control.md` … `repository-bootstrap.md`

## Commands

```bash
cargo metadata --format-version 1 --no-deps >/dev/null
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
bash Analyze-8/verification/deny-originals.sh
bash Analyze-8/verification/deny-foreign-workspace.sh
test -f specs/book-0.md && test -f specs/conformance.md
git status --porcelain -- 'Manifesto etc' 'Meditation_About'
```

## Result

```text
cargo metadata → OK
cargo fmt --check → OK
cargo clippy -D warnings → OK
cargo test --workspace → OK (14 crate smoke tests)
deny-originals.sh → OK
deny-foreign-workspace.sh → OK
specs/book-0..v + governance → present
Manifesto etc / Meditation_About → unchanged
```

**Verdict:** PASS
