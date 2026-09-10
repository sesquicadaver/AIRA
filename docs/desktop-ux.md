# AIRA Desktop UX — канон рішень

**Статус:** Phase E `#75`–`#106` **DONE** (2026-08-22); Phase F `#107`–`#119` **DONE** (2026-08-24).  
**Черга:** Phase O [`phase-o-plan.md`](phase-o-plan.md) `#255`–`#265` **DONE** @ [`AIRA-RFC-0146`](../specs/rfc/AIRA-RFC-0146-phase-o-desktop-ux-help.md); QUEUE O closed. Phase P [`phase-p-plan.md`](phase-p-plan.md) (runtime-honest) `#266`–`#274` **DONE** @ RFC-0156; QUEUE P closed. Phase Q [`phase-q-plan.md`](phase-q-plan.md) `#275`–`#285` **DONE** @ [`AIRA-RFC-0164`](../specs/rfc/AIRA-RFC-0164-phase-q-cross-install-integrity.md); **QUEUE Q closed**; no OPEN Q atoms. Phase R [`phase-r-plan.md`](phase-r-plan.md) `#286`–`#294` **DONE** @ [`AIRA-RFC-0174`](../specs/rfc/AIRA-RFC-0174-phase-r-actionable-connection-ux.md) (`connection_cta` / `ui_connect_primary` / `mesh_language` / `problem_action` / Help connect / UK mesh / cold-start @ RFC-0175…RFC-0181; **QUEUE R closed**; no OPEN R atoms). Phase T [`phase-t-plan.md`](phase-t-plan.md) `#306`–`#312` **DONE** @ [`AIRA-RFC-0192`](../specs/rfc/AIRA-RFC-0192-phase-t-operation-lifecycle-honesty.md) (**QUEUE T closed**; no OPEN T atoms). Phase U [`phase-u-plan.md`](phase-u-plan.md) `#313`–`#322` **IN PROGRESS** (`#318` DONE @ RFC-0203; first OPEN `#322`; RFC-0198 reserved). Phase S [`phase-s-plan.md`](phase-s-plan.md) `#295`–`#305` **DONE** @ [AIRA-RFC-0182](../specs/rfc/AIRA-RFC-0182-phase-s-cross-boundary-integrity.md) (**QUEUE S closed**; RFC-0183…RFC-0191; no OPEN S atoms; `#304` no joint atomic); post-R `aira-current.md` @ `e5aee86`).
**Provenance:** [`NEXT_PROBLEM.md`](../NEXT_PROBLEM.md) (**RESOLVED** → E); UX/Help draft 2026-09-06 (`aira-gui.md` + `aira-desktop-ux-help-draft.md`, поза git).  
**Не змінює** Book 0–IV / C0–C1 / Core / `aira-core` ledger.  
**Posture:** Linux E1 = **Developer Preview** над local reference plane.

## 1. Персони

| Персона | Статус |
|---------|--------|
| **End-user** | основна — продукт **AIRA Desktop** |
| **Developer** | окрема редакція **AIRA Dev** (той самий runtime + повний CLI) |
| Оператор вузла | **поза scope** |

**Правило мови (Phase O):** користувач повинен розуміти, що відбувається, що доступно і що робити далі, **не знаючи** внутрішньої архітектури AIRA (`POST /v1/problems`, CSU, Phase D, fail-closed, mesh, bind — не основні підписи UI).

## 2. Запуск і оболонка

- Запуск node для end-user — **лише через GUI** (іконка / tray).
- CLI канон: **`aira desktop start|stop|status`** (shared lifecycle з GUI).
- Інтерактивний запуск (іконка меню, `aira-desktop`, `aira desktop gui`) **завжди відкриває вікно**.
- `open_ui_on_start` (default **on**) — лише для **login autostart** (`aira-desktop --from-autostart`).
- Мова UI: Українська / English (`ui-prefs.json` поруч із settings; не частина `desktop-settings` schema).
- GUI tech: **Rust-only** (egui/native). Без Node.js/web build dependency.
- Local HTTP loopback (`aira-node --http`) у Desktop **завжди** увімкнений; mutating routes — лише з Desktop auth-контрактом ([phase-e §2.4](phase-e-plan.md)).

### 2.1. Цільова інформаційна архітектура (Phase O)

Три основні розділи + постійна команда довідки:

| Розділ | Питання користувача | Основний вміст |
|--------|---------------------|----------------|
| **Робота** | «Що я хочу зробити?» | Введення завдання, перебіг, результат, походження |
| **Стан системи** | «Чи все працює?» | Готовність, модель, з’єднання, останні події |
| **Параметри** | «Як змінити поведінку?» | Мова, запуск, моделі, участь у мережі |

**Довідка · F1** — постійно доступна команда оболонки (не вкладка). Розташування не змінюється між розділами.

Оболонка O4 (`#259` **DONE** @ [`AIRA-RFC-0150`](../specs/rfc/AIRA-RFC-0150-shell-ia-help-f1.md)): три розділи + **Довідка · F1** у chrome; Esc закриває лише довідку. Тіла екранів — `#260`–`#262`. Офлайн-shell F1 — `#263` **DONE** @ [`AIRA-RFC-0154`](../specs/rfc/AIRA-RFC-0154-offline-f1-shell.md); seed + link check — `#264` **DONE** @ [`AIRA-RFC-0155`](../specs/rfc/AIRA-RFC-0155-seed-help-topics.md). Історичний E1 layout **Work / Node / Network / Settings** більше не є primary nav.

### 2.2. Рядок стану (усі основні екрани)

Показувати незалежно:

1. **Робота:** готова / виконується / потрібна дія.
2. **Генерація тексту:** вибрана модель / не вибрана / тестовий режим.
3. **Мережа AIRA:** вимкнена / перевіряється / є з’єднання / з’єднання втрачено / не перевірено.

Невибрана LLM **не** означає недоступність детермінованих обчислень. Вимкнена мережа **не** означає збій локальної роботи.

Для моделі розрізняти: **вибрана** (налаштування) → **підготовлена** (backend підтвердив) → **використана для результату** (підтверджені дані цього виконання). Без даних: «Модель цього результату не визначена»; mock: «Тестова відповідь — модель не запускалася»; арифметика CSU: «Обчислено локальним обчислювачем».

### 2.3. Два рівні подробиць

Основний екран — людською мовою. **Технічні подробиці** розкривають точні параметри на тому ж екрані (ті самі дані й дозволи). Спрощення мови **без** спрощення істини: «цілісність файлу перевірено» ≠ «модель завжди права»; «адреса збережена» ≠ «з’єднання встановлено».

## 3. Робота (цільовий контракт)

O5 (`#260` **DONE** @ [`AIRA-RFC-0151`](../specs/rfc/AIRA-RFC-0151-work-screen-provenance.md)): user-language chrome, Ctrl+Enter / ⌘+Enter, draft retention, Origin/provenance line.

- Заголовок орієнтир: **«Що потрібно зробити?»**; кнопка **«Виконати»**.
- Enter = новий рядок; Ctrl+Enter (macOS: Cmd+Enter) = виконати, якщо дія доступна.
- Чернетка не губиться при зміні розділу, F1, помилці валідації чи невдалій відправці.
- Відсутність моделі не блокує всі типи завдань без перевірки потрібної capability.
- Прогрес лише з відомих станів runtime; без вигаданого відсотка / ETA.
- Один активний submit у базовому UI; повторне натискання не створює дубліката.
- **Скасувати** — лише якщо виконавець підтверджує скасування; закриття HTTP ≠ скасування обчислення.
- Результат: спочатку відповідь, далі статус виконання й перевірки **окремо**; mock / відсутність provenance — явно.

Локальні моделі — **не Core і не маркетплейс**. Phase P `#269` **DONE** @ [`AIRA-RFC-0159`](../specs/rfc/AIRA-RFC-0159-model-triple-ux.md): selected ≠ ready ≠ used-in-result (strip/System/Settings observe-only). Phase Q `#283` **DONE** @ [`AIRA-RFC-0172`](../specs/rfc/AIRA-RFC-0172-backend-ne-model-used.md): used = model id/hash only (never `backend:*`). Phase Q `#284` **DONE** @ [`AIRA-RFC-0173`](../specs/rfc/AIRA-RFC-0173-invite-import-atomicity.md): invite import checks-before-writes + trust rollback. Reachability DIRECT requires endpoint+direction bind @ [`AIRA-RFC-0160`](../specs/rfc/AIRA-RFC-0160-reachability-endpoint-bind.md) (`#270`). Канон: Artifact + Capability + Policy CSU (`aira models`); LLM Backend — зовнішній CSU. C1 `Calculate 2 + 2` → `execution-basic`; інший текст → `text.generate.local`. Phase K **DONE** @ RFC-0104. Не fake VERIFIED.

## 4. Стан системи (цільовий контракт)

O6 (`#261` **DONE** @ [`AIRA-RFC-0152`](../specs/rfc/AIRA-RFC-0152-system-status-screen.md)): секції **Програма** / **Модель** / **З’єднання** / **Останні події** з чесними висновками.

Секції: **Програма**, **Модель**, **З’єднання**, **Останні події**.

Кожна: висновок людською мовою, час спостереження, максимум одна коригувальна дія, технічні подробиці. Phase U `#321` **DONE** @ RFC-0206: systemd/docs peer bind ∈ P_AIRA. Phase U `#320` **DONE** @ RFC-0205: AddressBook selective rollback. Phase U `#319` **DONE** @ RFC-0204: submit executor honesty (mock/reference label; env process bind). Phase U `#318` **DONE** @ RFC-0203: failed submit durable history. Phase U `#317` **DONE** @ RFC-0202: policy audit ID uniqueness. Phase U `#316` **DONE** @ RFC-0201: CLI/Desktop shared identity create fail-closed. Phase U `#315` **DONE** @ RFC-0200: result-by-problem ArtifactStore authority. Phase U `#314` **DONE** @ RFC-0199: capsule-sourced verification. Phase U `#313` **DONE**: wiring; first OPEN `#322`. Phase T `#312` **DONE** @ RFC-0192: consolidating close; QUEUE T closed. Phase T `#311` **DONE** @ RFC-0197: failed opt-in dial rolls back AddressBook candidate. Phase T `#310` **DONE** @ RFC-0196: Quit during Submit defers Stop→Close with waiting copy. Phase T `#309` **DONE** @ RFC-0195: observe fail sticky per pointer/cache version (no rehash storm). Phase S `#303` **DONE** @ RFC-0190: observe miss → background streaming hash (`OBSERVE_HASH_PENDING`). Phase S `#302` **DONE** @ RFC-0189: submit⊥lifecycle (`admit_submit_lifecycle`). Phase S `#301` **DONE** @ RFC-0188: stopped peer CTA → Start. Phase S `#300` **DONE** @ RFC-0187: Technical details opt-in `run_opt_in_peer_dial` + dial evidence. Phase S `#299` **DONE** @ RFC-0186: Connection always shows `conn_boundary_guidance` (setup ≠ remote; loopback ≠ dial). Phase R `#287` **DONE** @ RFC-0175: Connection primary CTA = `connection_cta::primary_connection_cta` (не лише Refresh). Phase R `#290` **DONE** @ RFC-0178: problem/strip next-step = `problem_action` (wire `help:`/`try:` secondary).

### 4.1. З’єднання — джерела істини (чесність Phase N / N-fix)

| Значення | Джерело |
|----------|---------|
| Налаштований порт | збережена конфігурація |
| Порт, що реально слухається | запущений listener |
| Зовнішній endpoint | спостереження transport/family |
| Пряма доступність | чинна мережева перевірка |
| Живі з’єднання | authenticated sessions |
| Збережені учасники | AddressBook |
| Довірені учасники | TrustStore |
| Пошук учасників | стан provider + час останнього успіху |

UNKNOWN **не** показувати як OFFLINE. Кількість AddressBook **не** видавати за живі сесії. Локальна публікація ≠ з’єднання з глобальним provider.

## 5. Параметри (цільовий контракт)

O7 (`#262` **DONE** @ [`AIRA-RFC-0153`](../specs/rfc/AIRA-RFC-0153-settings-apply-lifecycle.md)): Saved ≠ Applied ≠ Restart needed; Phase P `#268` **DONE** @ [`AIRA-RFC-0158`](../specs/rfc/AIRA-RFC-0158-applied-from-runtime.md): Applied лише з підтвердженого runtime, інакше Explicitly Undefined; групи Загальні/Моделі/З’єднання/Додатково; закриття вікна ≠ Стоп.

Групи: **Загальні**, **Моделі**, **З’єднання**, **Додатково**.

Життєвий цикл зміни: **Редагується → Перевірено → Збережено → Застосовується → Застосовано** (+ за потреби **Потрібен перезапуск**). Збережені й застосовані значення видимі окремо.

Звичайний користувач бачить «Лише на цьому комп’ютері» та з’єднання з учасниками. Внутрішні **P0–P6** лишаються технічними позначеннями; P2 не перейменовувати в «Глобальна мережа» без нової поведінки.

**Закрити вікно** ≠ **Зупинити AIRA**.

## 6. Контракт даних і дій

Три межі (без універсального GUI framework / monitoring daemon):

1. **Стан:** типізований `SystemSnapshot` — read-only проєкція authoritative runtime/store. Якість даних: актуальні / застарілі / невідомі / недоступні (`DataQuality`; **DONE** @ RFC-0157 / `#267`: `observed_at` = measurement, `loaded_at` = projection load; bind ≠ listener proof). `request_repaint_after` **не** є оновленням даних; `refresh_status` (або еквівалент) — окремий канал.
2. **Дія:** стабільний ID, доступність, причина недоступності; runtime повторно перевіряє policy перед ефектом.
3. **Проблема:** стабільний код → локалізоване повідомлення → опційна команда → `help_id` (**DONE** @ RFC-0149 / `#258`; модуль `lexicon`).

GUI-state зберігає лише вкладку/фокус/чернетку/відкриті подробиці. Не джерело істини про мережу чи модель. Залежність `aira-node → aira-desktop` заборонена.

Блокувальний HTTP / важкі probes **не** в egui `update()` (**DONE** @ RFC-0148 / `#257`). Легкий статус — періодично у видимому вікні; дорогі діагностики — за явною дією. `request_repaint_after` ≠ оновлення даних.

## 7. Контекстна довідка F1

- F1 / «Довідка» відкриває тему для активного елемента → секція → екран → зміст.
- Працює **без** інтернету, моделі й навіть за зупиненого runtime (локальні матеріали версії).
- LLM **не** формує нормативну інструкцію.
- Формат теми: **Що це? Що зробити? Що має відбутися? Що робити, якщо не вдалося?**
- Спільний словник з `Labels`: `код → повідомлення → help_id`.
- Відкриття Help не очищає чернетку, не змінює налаштування, не скасовує завдання.
- Джерела: `docs/help/{uk,en}/`, вбудовані у збірку (`include_str!`); мінімальний plain Markdown renderer без remote HTML/скриптів (**DONE** @ RFC-0154 / `#263`).
- Пошук за id/title/body; **не** підміняє відкриту тему першим hit (`#273` @ RFC-0163).
- Контекст F1: **фокус елемента → секція → екран** (не stale `last_problem`); явна кнопка проблеми лишається `open_help(problem.help_id)`.
- Список тем / Related показує локалізовані заголовки статей; `HelpId` — внутрішній monospace.
- Contextual/`open_help` очищає `help_search`, щоб старий пошук не перебивав тему.
- Cross-links: `help:<id>` у Related; EN/UK parity (**DONE** @ RFC-0155 / `#264`).

Каталог ID (shell `#263`; seed/link-check `#264` **DONE**): `start`, `work.submit`, `work.result`, `work.waiting`, `model.select`, `model.unavailable`, `network.connect`, `network.reachability`, `network.trust`, `settings.apply`, `node.lifecycle`.

## 8. Network profiles (E1–E4, без змін семантики)

| ID | Назва | Поведінка | Desktop |
|----|-------|-----------|---------|
| **P0** | Лише локально | тільки `aira-node --http` loopback | **E1 default** |
| **P1** | Приватна мережа (довірені) | + `peer listen --recv`; trust + address book | **E1.1+** |
| **P2** | Приватна + авто-книга | P1 + `--dht --apply-book` | **E4** |
| **P3** | Relay | `--relay` (+ TTL); dial `via` | **E4** Advanced |
| **P4** | Gossip trust | `--gossip` (не з `--relay` одночасно) | **E4** Advanced |
| **P5** | Федерація (pin) | wizard `federation join` | **E4** |
| **P6** | STUN / discv / FIND | discovery prototypes | **E4** Dev / Advanced |

Заборонено в default Desktop: `--allow-public-bind`, публічний STUN default, авто-trust невідомих peers, прихований auto-increment порту.

## 9. Редакції

| | AIRA Desktop | AIRA Dev |
|--|--------------|----------|
| Вхід | іконка / tray | GUI + повний CLI |
| Root | OS application-data | `--root` / кілька профілів |
| Settings / PID / logs | OS config + runtime + log dirs | `--root` або colocated |
| First run | wizard → init + identity + auth | CLI без змін |
| Мережа | профілі з §8 | усі CLI-прапорці |

## 10. Мінімальні Settings (E1 keys)

| Key | Default | Примітка |
|-----|---------|----------|
| `network_profile` | `P0` | вищі за DONE-рівень — fail-closed |
| `open_ui_on_start` | `true` | лише **login autostart** |
| `autostart_on_login` | `false` | Linux/macOS/Windows hooks |
| `http_listen` | `127.0.0.1:8787` | fixed |
| `instance_id` | generated once | attach-семантика |
| `peer_listen` | `null` @ P0; default `127.0.0.1:49157` @ P1 | обов’язковий при P1 |

## 11. Критерії приймання Phase O (канон)

| Перевірка | Результат |
|-----------|-----------|
| Перший запуск без моделі | Зрозуміло, що доступно; немає вигаданого імені моделі |
| F1 без node / мережі / LLM | Локальна тема відкривається |
| Повільне виконання | F1 і навігація не блокуються |
| UNKNOWN / stale мережа | Не як підтверджений OFFLINE/CONNECTED |
| AddressBook без сесій | Не рахуються як підключені |
| Mock / без model provenance | Явно позначено |
| F1 з помилки | Відповідна тема, не початок посібника |
| UK/EN | Збіг UI, Help і переходів |

Детальний порядок реалізації: [`phase-o-plan.md`](phase-o-plan.md).

## 12. Посилання

- Phase O plan: [`phase-o-plan.md`](phase-o-plan.md)
- Plan E / acceptance: [`phase-e-plan.md`](phase-e-plan.md)
- Desktop GUI (поточна реалізація): [`desktop-gui.md`](desktop-gui.md)
- Local HTTP: [`local-node.md`](local-node.md)
- Peer: [`peer-link.md`](peer-link.md)
- Network profiles P0–P6: [`desktop-network-profiles.md`](desktop-network-profiles.md)
- Packaging / launcher: [`desktop-packaging.md`](desktop-packaging.md), [`desktop-launcher.md`](desktop-launcher.md)
