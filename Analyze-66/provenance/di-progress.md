# Deep-interview progress — Analyze-66

## Settled
- **Mechanism (Q1=A):** STUN Binding client → reflexive IP:port; no full ICE; no UDP peer sessions; mock STUN in venv.
- **Dial integration (Q2=A1):** Explicit CLI query; dial stays book→TCP; no STUN-per-dial.
- **Persistence (Q3=B1):** `peers/stun_reflexive.json` (addr + server + ts); dial does not read it.

## Open
- STUN server URL policy (default public vs required flag)
- Whether `dht announce --from-stun` (or equiv) is in this slice
- Non-goals / decision boundaries formal
- Wire format / RFC5769 vectors vs minimal Binding success
