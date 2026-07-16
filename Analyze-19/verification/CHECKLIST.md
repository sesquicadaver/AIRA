# Verification checklist — Analyze-19

- [x] Ralplan approved (Roadmap M11)
- [x] `aira-node --http` serves M11 endpoints on loopback default
- [x] `cargo test -p aira-node` — 10 HTTP integration tests PASS
- [x] Docs: `docs/local-node.md` + README updated
- [x] `cargo clippy -p aira-node -p aira-protocol --all-targets -- -D warnings`
- [x] `Analyze-19/verification/deny-originals.sh`
- [x] Code review APPROVE + CLEAR
