# Analyze-261 — seccomp (QUEUE #226)

## Done
- Opt-in Linux seccomp BPF deny-list on `ProcessBackend` child (`pre_exec` + `PR_SET_NO_NEW_PRIVS` + `SECCOMP_SET_MODE_FILTER`)
- Forbidden syscall (e.g. `socket`) → SIGSYS → `SECCOMP_VIOLATION` CapsuleFailed
- Fail-closed: `SECCOMP_FAILED` / `SECCOMP_UNSUPPORTED`
- RFC [`AIRA-RFC-0119`](../../specs/rfc/AIRA-RFC-0119-seccomp.md)
- Living `phase_m_seccomp_226`; QUEUE `#226` **DONE**; first OPEN `#227`
- C1 2+2 unchanged; RFC-0117 file-free

## Out
netns (`#227`); missing-kernel policy (`#228`); OS vs AIRA-mediated schema (`#229`); RFC-0117 body; Core inference; GPU marketplace; requiring ollama in CI.
