# AIRA Desktop UX — канон рішень

**Статус:** зафіксовано 2026-08-20 (рішення розробника); уточнено 2026-08-20 (CLI, layout, GUI tech, port/HTTP).  
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
- Після старту відкривати UI — **налаштування** (`open_ui_on_start`, default **on** для Desktop).
- «Open UI» = **нативне** status/settings вікно (не голий JSON у браузері як єдиний UX).
- Автостарт після логіну — **налаштування** (`autostart_on_login`, default **off**); OS hooks: Linux `#78` (XDG), macOS `#87` (LaunchAgent).
- Local HTTP loopback (`aira-node --http`) у Desktop **завжди** увімкнений; mutating routes — лише з Desktop auth-контрактом ([phase-e §2.4](phase-e-plan.md)).
- GUI tech: **Rust-only** (egui/native tray). Без Node.js/web build dependency.

## 3. Network profiles

| ID | Назва | Поведінка | Desktop |
|----|-------|-----------|---------|
| **P0** | Лише локально | тільки `aira-node --http` loopback | **E1 default / єдиний у першому Linux-релізі** |
| **P1** | Приватна мережа (довірені) | + `peer listen --recv`; trust + address book | **E1.1+** (після E1) |
| **P2** | Приватна + авто-книга | P1 + `--dht --apply-book` | після P1 |
| **P3** | Relay | `--relay` (+ TTL); dial `via` | Advanced |
| **P4** | Gossip trust | `--gossip` (не з `--relay` одночасно) | Advanced |
| **P5** | Федерація (pin) | wizard `federation join` | окремий wizard |
| **P6** | STUN / discv / FIND | discovery prototypes | **Dev / Advanced only** |

**Онбординг P1:** обмін файлом/QR (pubkey + опційно addr) — Addendum E1.1 (`#80`–`#85`).  
**Рішення 2026-08-20:** варіант **C** — E1 = лише **P0 + GUI**; P1 = E1.1.  
**Рішення E1.1 (2026-08-20):** default `peer_listen=127.0.0.1:9797`; non-loopback лише explicit; QR = PNG файл (без камери); P2–P6 Out.

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
E0 (код) → E1 Linux (P0) → E1.1 P1+QR → E2 macOS (`#86`–`#89`) → E3 Windows
```

Поставка атомів: lifecycle → `.desktop` → tray/GUI → package (`#76`→`#79`) → E1.1 P1+QR (`#80`→`#85`).

## 6. Мінімальні Settings (E1)

| Key | Default | Примітка |
|-----|---------|----------|
| `network_profile` | `P0` | `P1` дозволений у E1.1 (`#81`); P2+ fail-closed |
| `open_ui_on_start` | `true` | native UI |
| `autostart_on_login` | `false` | Linux XDG `#78`; macOS LaunchAgent `#87` |
| `http_listen` | `127.0.0.1:8787` | fixed; conflict → fail або attach ([phase-e §2.3](phase-e-plan.md)) |
| `instance_id` | generated once | для attach-семантики |
| `peer_listen` | `null` @ P0; default `127.0.0.1:9797` @ P1 | обов’язковий при P1 |
| auth fields | per `#75`/`#76` | token-ref або IPC mode |

## 7. Посилання

- Plan / acceptance: [`phase-e-plan.md`](phase-e-plan.md)
- Local HTTP: [`local-node.md`](local-node.md)
- Peer (для майбутніх P1+): [`peer-link.md`](peer-link.md)
- Desktop P1 peer supervise: [`desktop-peer.md`](desktop-peer.md)
- PeerInvite file: [`desktop-invite.md`](desktop-invite.md)
- systemd (сервери, не Desktop): [`runbook-systemd.md`](runbook-systemd.md)
- Linux menu launcher: [`desktop-launcher.md`](desktop-launcher.md)
- Desktop GUI: [`desktop-gui.md`](desktop-gui.md)
