# Матриця: Conformance / Profile scales

**Мета:** звести колізії імен шкал (TF4-002 / TF5-003 / TF6-001 / TF6-002 → TF7-001).

| Шкала | Джерело | Значення | Що вимірює | Нормативність |
|-------|---------|----------|------------|---------------|
| **AIRA-C0…C5** | SCD §10; Conformance §4; Schema §27; MVP Roadmap §3 | Local Core → Research-Capable | **Compatibility profile** реалізації | **Канон сумісності** |
| Book I **L0–L3** | Book I (Analyze-2) | Conformance levels Core/ABI | Рівень Core/ABI | Normative (Book I) — **не** те саме, що AIRA-C* |
| Book II **P0–P3** | Book II (Analyze-3) | Protocol profiles | Набір протоколів | Normative (Book II) |
| Book III **C0–C5** | Book III (Analyze-4) | CSU/role conformance naming | Ролі/контракти CSU | Normative — **колізія літери C** з AIRA-C* |
| Book IV **R0–R4** | Book IV §22–23 | Reference harness / Reference Profiles | Reference impl maturity | Reference only |
| Book V **RV0–RV4** | Book V §27 | Research CSU experiment profiles | Research isolation→promotion | Research only |
| Book V **L0–L4** escalation | Book V §10 | Ready→…→Cognitive Evolution | Метод ескалації Resolution | Research — **колізія L** з Book I |

## Рекомендована glossary (RFC-чернетка, не edit originals)

| Канонічна назва | Alias у docs | Примітка |
|-----------------|--------------|----------|
| `AIRA-C0…C5` | C0…C5 у MVP/Conformance | Єдиний profile для harness CLI |
| `ABI-L0…L3` | Book I L* | Перейменувати в docs via RFC |
| `PROTO-P0…P3` | Book II P* | OK якщо префікс PROTO |
| `CSU-C0…C5` або role matrix | Book III C* | Узгодити з AIRA-C* або відмовитись від C* |
| `REF-R0…R4` | Book IV R* | Reference only |
| `RES-RV0…RV4` | Book V RV* | Research only |
| `ESC-E0…E4` | Book V escalation L* | Уникнути L |

## MVP mapping

```text
MVP target = AIRA-C0 + AIRA-C1 (+ partial local AIRA-C2 stubs)
≈ Book IV R0 / Minimal Local Core
≠ Book V RV* / ESC-*
```
