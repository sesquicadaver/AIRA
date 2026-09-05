# AIRA Desktop UX — канон рішень

**Статус:** зафіксовано 2026-08-20; Phase E `#75`–`#106` **DONE** (2026-08-22); Phase F stabilization **DONE** (`#107`–`#119`, 2026-08-24). Активна черга: Phase G [`phase-g-plan.md`](phase-g-plan.md) `#120`–`#146`.  
**План виконання:** [`phase-e-plan.md`](phase-e-plan.md). Канон черги: [`QUEUE.md`](../QUEUE.md).  
**Provenance:** [`NEXT_PROBLEM.md`](../NEXT_PROBLEM.md) (**RESOLVED**).  
**Не змінює** Book 0–IV / C0–C1 / Core.  
**Posture:** Linux E1 = **Developer Preview** над local reference plane.

## 1. Персони

| Персона | Статус |
|---------|--------|
| **End-user** | основна — продукт **AIRA Desktop** |
| **Developer** | окрема редакція **AIRA Dev** (той самий runtime + повний CLI) |
| Оператор вузла | **поза scope** |

## 2. Запуск і UI

- Запуск node для end-user — **лише через GUI** (іконка / tray).
- CLI канон: **`aira desktop start|stop|status`** (shared lifecycle з GUI).
- Інтерактивний запуск (іконка меню, `aira-desktop`, `aira desktop gui`) **завжди відкриває вікно**.
- `open_ui_on_start` (default **on**) — лише для **login autostart** (`aira-desktop --from-autostart`). Знятий чекбокс не блокує повторний вхід у Параметри з іконки.
- «Open UI» = **нативне** вікно з вкладками Work / Node / Network / Settings (не голий JSON у браузері як єдиний UX).
- Мова UI: Українська / English (`ui-prefs.json` поруч із settings; не частина `desktop-settings` schema).
- Вкладка Work: людська відповідь (`result.result`) + `status` + `verification_status` на передньому плані; `problem_id` / `verified_artifact_id` і повний VRA JSON — згорнуті Details. Не сирий JSON як «відповідь AIRA».
- Локальні моделі — **не Core і не маркетплейс**. Канон: Artifact + Capability + Policy CSU (`aira models`); LLM Backend — зовнішній CSU (Book I §2). GUI Work: C1 `Calculate 2 + 2` → `execution-basic`; інший текст → `text.generate.local` (`POST /v1/problems`; MockBackend у CI; fail-closed без Phase D activate; не fake VERIFIED). Phase K [`phase-k-plan.md`](phase-k-plan.md) **DONE** @ RFC-0104.
- Автостарт після логіну — **налаштування** (`autostart_on_login`, default **off**); OS hooks: Linux `#78` (XDG), macOS `#87` (LaunchAgent), Windows `#91` (Startup/Registry).
- Local HTTP loopback (`aira-node --http`) у Desktop **завжди** увімкнений; mutating routes — лише з Desktop auth-контрактом ([phase-e §2.4](phase-e-plan.md)).
- GUI tech: **Rust-only** (egui/native tray). Без Node.js/web build dependency.

## 3. Network profiles

| ID | Назва | Поведінка | Desktop |
|----|-------|-----------|---------|
| **P0** | Лише локально | тільки `aira-node --http` loopback | **E1 default / єдиний у першому Linux-релізі** |
| **P1** | Приватна мережа (довірені) | + `peer listen --recv`; trust + address book | **E1.1+** (після E1) |
| **P2** | Приватна + авто-книга | P1 + `--dht --apply-book` | **E4** (`#94`–`#96`) |
| **P3** | Relay | `--relay` (+ TTL); dial `via` | **E4** Advanced (`#97`–`#99`) |
| **P4** | Gossip trust | `--gossip` (не з `--relay` одночасно) | **E4** Advanced (`#100`–`#102`) |
| **P5** | Федерація (pin) | wizard `federation join` | **E4** (`#103`–`#104`) |
| **P6** | STUN / discv / FIND | discovery prototypes | **E4** Dev / Advanced (`#105`) |

**Онбординг P1:** обмін файлом/QR (pubkey + опційно addr) — Addendum E1.1 (`#80`–`#85`).  
**Рішення 2026-08-20:** варіант **C** — E1 = лише **P0 + GUI**; P1 = E1.1.  
**Рішення E1.1 (2026-08-20):** default `peer_listen=127.0.0.1:49157`; non-loopback лише explicit; QR = PNG файл (без камери).  
**Рішення E4 (2026-08-22):** P2–P6 відкрито в [`phase-e-plan.md`](phase-e-plan.md) §4d → QUEUE `#94`–`#106` — **DONE**.
**Phase G (2026-08-24):** peer lifecycle P3/P4 CI stabilization (`#131`–`#132`); invite QR **camera** capture (`#133`); production packaging scripts (`#143`–`#145`).

Заборонено в default Desktop: `--allow-public-bind`, публічний STUN default, авто-trust невідомих peers, прихований auto-increment порту.

## 4. Редакції

| | AIRA Desktop | AIRA Dev |
|--|--------------|----------|
| Вхід | іконка / tray | GUI + повний CLI |
| Root | OS application-data | `--root` / кілька профілів |
| Settings / PID / logs | OS config + runtime + log dirs ([phase-e §2.1](phase-e-plan.md)) | `--root` або colocated |
| First run | wizard → init + identity + auth material (§2.4) | CLI без змін |
| Мережа | профілі з §3 | усі CLI-прапорці |

## 5. Послідовність ОС

```text
E0 (код) → E1 Linux (P0) → E1.1 P1+QR → E2 macOS (`#86`–`#89`) → E3 Windows (`#90`–`#93`) → E4 P2–P6 (`#94`–`#106`)
```

Поставка атомів: lifecycle → `.desktop` → tray/GUI → package (`#76`→`#79`) → E1.1 P1+QR (`#80`→`#85`).

## 6. Мінімальні Settings (E1)

| Key | Default | Примітка |
|-----|---------|----------|
| `network_profile` | `P0` | `P1` E1.1 (`#81`); `P2`–`P6` E4 (`#94`+); вищі за DONE-рівень — fail-closed |
| `open_ui_on_start` | `true` | вікно при **login autostart**; іконка завжди відкриває UI |
| `autostart_on_login` | `false` | Linux XDG `#78`; macOS LaunchAgent `#87`; Windows `#91` |
| `http_listen` | `127.0.0.1:8787` | fixed; conflict → fail або attach ([phase-e §2.3](phase-e-plan.md)) |
| `instance_id` | generated once | для attach-семантики |
| `peer_listen` | `null` @ P0; default `127.0.0.1:49157` @ P1 | обов’язковий при P1 |
| auth fields | per `#75`/`#76` | token-ref або IPC mode |

## 7. Посилання

- Plan / acceptance: [`phase-e-plan.md`](phase-e-plan.md)
- Local HTTP: [`local-node.md`](local-node.md)
- Peer (для майбутніх P1+): [`peer-link.md`](peer-link.md)
- Desktop P1 peer supervise: [`desktop-peer.md`](desktop-peer.md)
- Network profiles P0–P6 (E4): [`desktop-network-profiles.md`](desktop-network-profiles.md)
- PeerInvite file: [`desktop-invite.md`](desktop-invite.md)
- systemd (сервери, не Desktop): [`runbook-systemd.md`](runbook-systemd.md)
- Linux menu launcher: [`desktop-launcher.md`](desktop-launcher.md)
- Desktop GUI: [`desktop-gui.md`](desktop-gui.md)
