# Матриця: Meditation_About → Book I

**Джерело:** `Meditation_About/` (це репо)  
**Ціль:** `Manifesto etc/Book I — Core Architecture & ABI v0.1.md`  
**Межі:** без зовнішніх робочих просторів

### Легенда

| Сила | Значення |
|------|----------|
| Direct | Майже 1:1 у канон |
| Strong | Той самий інтент, інша термінологія |
| Partial | Споріднено / частково |
| Superseded | Ідея замінена каноном |
| Tension | Потребує явного уточнення (не edit original) |

## Матриця

| Book I | § | Meditation | Сила | Примітка |
|--------|---|------------|------|----------|
| Microkernel / Core boundary | §3, §24 | 42 Microkernel | Direct | Ядро vs drivers → ядро vs CSU |
| Non-Goals (no LLM/GPU in core) | §2, §23 | 42, 53 | Strong | Lightweight + microkernel |
| Driver Model → CSU | §13–14 | 43 Driver Model | Superseded→CSU | Book I явно: Driver→CSU |
| Driver lifecycle | §14 | 43 lifecycle | Strong | Discovered…Archived узгоджено |
| Core Invariants | §11 | 19 Core Invariants | Strong | Intent/Capsule/Context; частина перейшла в Book 0 A* |
| Evolution & compatibility layers | §20–21 | 24 Evolution Model | Strong | Алгоритми еволюціонують, ABI стабільний |
| Lightweight / one job | §2–3 | 53 Lightweight Doctrine | Strong | Мінімальність ядра |
| Interop beyond binary ABI | §7–8 | 56 CIM / Semantic ABI | Tension | Book I = Stable ABI + Events; «Semantic ABI» не окремий шар у Book I |
| First principles → Core mission | §1 | 61 First Principles | Strong | PS/Context first → середовище для CSU |
| Event fabric | §8–9 | 42 Event Fabric ABI | Direct | Event Runtime |
| Policy Gate | §10 | 42 Policy Gate | Direct | |
| Security/Sandbox | §12 | 42 Security Boundary | Direct | |
| Execution State Machine in kernel | — | 42 ESM у ядрі | Tension | Book I не виділяє ESM як окрему підсистему Core; Capsule events замість |
| Capability as uncertainty reduction | §15 | 58/61 (через Book 0) | Strong | Узгоджено з Book 0 KPI |

## Вердикт

Book I кристалізує **42 + 43 + 19 + 53 + 24** у нормативне мікроядро з терміном **CSU**.  
Головні напруги для Analyze-3/RFC-чернеток: **56 Semantic ABI** (чи достатньо Event+Artifact контрактів) і **ESM у 42** vs відсутність окремого ESM у Book I.
