# Матриця: Meditation_About → Book II

**Джерела:** лише `Meditation_About/` цього репо  
**Ціль:** Book II

| Book II | § | Meditation | Сила | Примітка |
|---------|---|------------|------|----------|
| Discovery by Capability | §8 | 10 Discovery & Overlay | Strong→Superseded shape | Було resource/node/DHT; канон — Capability/CSU |
| Protocol stack layering | §3 | 7 Protocol Stack | Strong | Рівні незалежні; Book II = contract layer |
| Network/distributed exec | — | 5 Network Protocol | Historical | Алгоритми inference/KV → не Book II |
| CRP by capability/capsule | §10 | 15 Compute Routing Protocol | Direct rename | Compute→Capability Routing; algorithm out |
| Identity / Trust / Federation | §13–14 | 9 Identity Trust Federation | Strong | Wallet/Node demoted; crypto identity kept |
| Settlement / Audit | §15 | 4 Blockchain Protocol | Superseded as required | Blockchain → optional MAY |
| Envelope / interoperability | §6 | 7 (+ later ABI docs) | Partial | Канонічний envelope — Book II |
| Event/Artifact wire format | §11–12 | (через Book I + early fabric) | Strong | Book I runtime + Book II wire |

## Вердикт

Book II **нормалізує** ранні P2P протоколи (4,5,7,9,10,15) у transport-agnostic contracts: capability-first discovery/routing, optional settlement, federation autonomy, policy-before-action.
