# Правила Analyze-19

Local HTTP API (Roadmap M11).

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. Default listen address must be loopback (`127.0.0.1`)
3. HTTP handlers must call existing `LocalSession` / `CsuRegistry` / `run_profile` — no duplicate pipeline
4. No authentication / multi-tenant / federation in this cycle
5. Docs must match runnable `aira-node --http` paths

## Out of scope
TLS, public bind defaults, GitHub Issue creation, production crypto
