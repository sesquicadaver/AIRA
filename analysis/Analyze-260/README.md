# Analyze-260 — Landlock FS (QUEUE #225)

## Done
- Opt-in Linux Landlock ABI-1 on `ProcessBackend` child (`pre_exec` + `PR_SET_NO_NEW_PRIVS`)
- Fail-closed: `LANDLOCK_FAILED` / `LANDLOCK_UNSUPPORTED`; denied extra-allowlist read → CapsuleFailed
- RFC [`AIRA-RFC-0118`](../../specs/rfc/AIRA-RFC-0118-landlock-fs.md)
- Living `phase_m_landlock_225`; QUEUE `#225` **DONE**; first OPEN `#226`
- C1 2+2 unchanged; RFC-0117 file-free

## Out
seccomp (`#226`); netns (`#227`); missing-kernel policy (`#228`); OS vs AIRA-mediated schema (`#229`); RFC-0117 body; Core inference; GPU marketplace; requiring ollama in CI.
