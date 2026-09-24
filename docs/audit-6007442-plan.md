# Audit gate + post-Y linear plan (`6007442`)

**Статус:** OPEN (D/A). Блок K `#376`–`#382` **DONE**. Phase Y `#359`–`#375` **DONE**. `#383` **DONE**. Перший рядок виконання — `#384`.  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md).  
**Джерело:** [`p-only/AIRA-full-audit-6007442-20260923.md`](../../p-only/AIRA-full-audit-6007442-20260923.md) (поза git). Перевірений `main`: `6007442`.  
**Зв’язок із Phase Y:** `#359`–`#375` **DONE**. K **DONE**. D `#383` **DONE**; `#384`–`#387` OPEN.  
**Не входить:** marketplace, ratings GUI, новий LLM-провайдер, Research Plane, встановлення Ollama, заміна egui.

Лінійність: один рядок — один PR. Не паралельний план поруч із Y.

## Порядок виконання

```text
#376 K1 firewall
  → #377 K2a CAS id bind
    → #378 K2b SQLite id bind
      → #379 K3 verified CLI name
        → #380 K4a catalog snapshot (no disk per paint)
          → #381 K4b prepare-and-run async job
            → #382 K5 applied gate for Prepare-and-run
              → #364 … #375 (Phase Y tail)
                → #383 installed-file ≠ Work ready (DONE)
                  → #384 … #387 (D docs)
                    → #388 … #389 (installed acceptance)
```

## Блок K — інженерні дефекти аудиту (`#376`–`#382`)

| # | Scope | Done when | Не в рядку |
|---|--------|-----------|------------|
| 376 | **DONE.** Dependency firewall: ребро через `dep.pkg → id_to_name`; тести дефісів/alias | `edges_from_resolve` бачить `aira-core → aira-node`; self-test ловить пропуск | зміна правил Core/CSU |
| 377 | **DONE.** CAS `resolve`: `descriptor.artifact_id == requested`; тест із **іншим валідним** підписаним B | `resolve(A)` не повертає B (`IdMismatch`) | зміна privacy policy |
| 378 | **DONE.** SQLite `get_by_object_id`: bind ID як у `open` | lookup(A) з чужим `object_id` у JSON → fail (`HandleBindMismatch`) | новий store |
| 379 | **DONE.** CLI-ім’я лише з перевіреної прив’язки; `host_ollama_model_ref(cli)==model_ref`; pointer = locator | пошкоджений pointer видимий; Make default не створює bind B | `#364` first-row |
| 380 | **DONE.** Каталог у фоновому snapshot; render = lookup, не O(N×S) I/O на кадр | Work/Settings не читають tip/cache на кожен paint | метадані HTTP |
| 381 | **DONE.** Prepare-and-run = один async job (slot → admit → submit), один inflight | UI не блокується; повтор не стартує другу роботу | фільтр списку |
| 382 | **DONE.** Одна оцінка **applied** executor + усі вибрані моделі; потім «кому потрібен slot» | Mock≠Process і Compare file+A/B узгоджені з Run; негативний тест кнопки→submit | повний `#367` E2E |

## Phase Y — хвіст після K (`#364`–`#375`)

Канон scope без змін: [`phase-y-plan.md`](phase-y-plan.md).  
`#367` **DONE** — M0 through-test + негативи K3–K5 (prepare fail, double-click, applied mismatch).
`#368` **DONE** — `/api/tags` parse/fetch; відсутнє поле = Unknown (без вигаданих `false`).
`#369` **DONE** — `/api/show` коли бракує будь-якого поля оцінки (caps відомі + місце Unknown → show).
`#370` **DONE** — in-memory cache (endpoint/name/digest), generation reject, parallel 2–4.
`#371` **DONE** — locality predicate (server-local vs this computer; empty/unknown/localhost/:cloud honesty).
`#372` **DONE** — unified fitness five states; completion+vision ready when local.
`#373` **DONE** — model selector search/Compatible/Show all; N of M; selection survives filter.
`#374` **DONE** — Compare enter clears B; exit restores solo pick; distinct full CLI names stay distinct.
`#375` **DONE** — EN/UK fitness labels + F1 Prepare-and-run; `:cloud` suffix is not locality proof; local-node row.

## Блок D — документація (`#383`–`#387`)

| # | Scope | Done when | Не в рядку |
|---|--------|-----------|------------|
| 383 | **DONE.** Installed-file / RFC-0240: файл видно, не executable через Ollama | критерій ≠ «Work ready» як generate | новий file-executor |
| 384 | C1 Executed vs VRA / OP-001 / matrix: три окремі статуси | без змішування smoke і verified pipeline | повернення калькулятора в GUI |
| 385 | Settlement receipt: один `$id` ↔ одна schema | JSON і schema-pack збігаються | federation runtime |
| 386 | `specs/README`: immutable baseline + amendments + пріоритет; RFC metadata | без другого канону | перепис Manifesto |
| 387 | Entry-points і зламані links (`demo`, `crypto`, implementation-status, `NEXT_PROBLEM`) | одне актуальне «перше OPEN» | масове DONE без E2E |

## Блок A — installed acceptance (`#388`–`#389`)

| # | Scope | Done when | Не в рядку |
|---|--------|-----------|------------|
| 388 | Acceptance на **конкретному SHA** після `#367` + K | GUI+CLI+HTTP дві моделі; негативи; SHA/Ollama/digest у evidence | перенос Analyze-396 як proof HEAD |
| 389 | Старі process-smoke / Analyze-396 — історичні snapshots | явна дата/SHA в шапці evidence | нові продуктові можливості |

## Правила

- Зберігати чинні fail-closed перевірки admission.
- Phase Y `#359`–`#375` **DONE**. Не оголошувати audit A DONE, доки `#388`–`#389` не закриті.
- Не додавати file-executor лише заради D1 / `#383`.
- Зелений CI не замінює installed-product acceptance (`#388`).
