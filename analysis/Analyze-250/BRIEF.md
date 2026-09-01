# BRIEF — Analyze-250

Add a real local CLI process backend for generate-local (ollama and/or llama.cpp-style argv) behind `GenerateBackend`. Missing binary → CapsuleFailed, never fake VERIFIED. CI/plane keep MockBackend. RFC-0105 constraints: `network=none` (AIRA opens no sockets; child may use loopback; no WAN); `shell=false` (fixed argv, never `sh -c`). Activate gate from `#214` still applies before spawn. C1 `Calculate 2 + 2` unchanged. Do not implement Desktop Work generate (`#216`) or file RFC-0104.
