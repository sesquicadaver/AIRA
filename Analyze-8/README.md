# Analyze-8 — Epic 0/1 Repository Bootstrap

**Статус:** Active  
**Цикл:** Analyze-8  
**Scope:** Issue Set #1–#7 (Epic 0 + Epic 1 snapshot)

## Ralplan (approved)

### Principles
1. Immutability: `Manifesto etc/**` і `Meditation_About/**` не змінюються
2. MVP freeze: без federation/GPU/LLM/blockchain/Research Plane
3. Conformance-first skeleton: workspace компілюється, fmt/clippy/test зелені
4. Specs у `specs/` = **копії** з Manifesto (не move)

### Decision
Реалізувати Epic 0 повністю + Issue #6–#7 (specs snapshot). Не реалізовувати runtime логіку (#22+).

### Options
- A: лише #1–#5 → відхилено (M0 без specs слабкий для наступного M1)
- B: #1–#7 → **обрано**
- C: повний MVP до #80 → поза циклом

### Acceptance
- `cargo metadata`, `cargo fmt --check`, `cargo clippy`, `cargo test` PASS
- CI workflow існує
- README anti-regression текст
- `specs/book-*.md` + governance docs
- Analyze-8 verification + deny-originals PASS
- `develop` branch існує
