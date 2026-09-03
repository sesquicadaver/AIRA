# Analyze-262 — netns (QUEUE #227)

## Done
- Opt-in Linux network namespace on `ProcessBackend` child (`pre_exec`: `CLONE_NEWUSER` + maps + `CLONE_NEWNET` before Landlock/seccomp)
- ollama-style + netns → `NETNS_BLOCKS_LOOPBACK` CapsuleFailed (host loopback not silently isolated)
- Fail-closed: `NETNS_FAILED` / `NETNS_UNSUPPORTED`
- RFC [`AIRA-RFC-0120`](../../specs/rfc/AIRA-RFC-0120-netns.md)
- Living `phase_m_netns_227`; QUEUE `#227` **DONE**; first OPEN `#228`
- C1 2+2 unchanged; RFC-0117 file-free

## Out
missing-kernel policy (`#228`); OS vs AIRA-mediated schema (`#229`); RFC-0117 body; Core inference; GPU marketplace; forcing ollama through netns; requiring ollama in CI.
