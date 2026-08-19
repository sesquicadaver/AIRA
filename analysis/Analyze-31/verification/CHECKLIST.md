# Checklist — Analyze-31

- [x] `--backup` stages `*.tmp`, commits `.prev` + meta after success
- [x] Default rotate creates no `.prev`
- [x] Staging failure aborts rotate (fail closed); no orphan durable secret
- [x] Trust fail preserves prior `.prev` slot
- [x] Dir trap on `.prev` cleared; rotate still Ok (trust≡secret)
- [x] Unix 0600 on staged backup
- [x] CLI prints `backup <path>`
- [x] docs updated
- [x] No Manifesto / Meditation edits
- [x] tests + clippy PASS
