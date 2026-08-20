# AIRA Desktop UX — канон рішень

**Статус:** зафіксовано 2026-08-20 (рішення розробника).  
**План виконання:** [`phase-e-plan.md`](phase-e-plan.md). Канон черги: [`QUEUE.md`](../QUEUE.md).  
**Не змінює** Book 0–IV / C0–C1 / Core.

## 1. Персони

| Персона | Статус |
|---------|--------|
| **End-user** | основна — продукт **AIRA Desktop** |
| **Developer** | окрема редакція **AIRA Dev** (той самий runtime + повний CLI) |
| Оператор вузла | **поза scope** |

## 2. Запуск і UI

- Запуск node для end-user — **лише через GUI** (іконка / tray).
- Після старту відкривати UI — **налаштування** (`open_ui_on_start`, default **on** для Desktop).
- Автостарт після логіну — **налаштування** (`autostart_on_login`, default **off**).
- Local HTTP loopback (`aira-node --http`) у Desktop **завжди** увімкнений (API для GUI).

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

**Онбординг P1:** обмін файлом/QR (pubkey + опційно addr) — **не** в E1.  
**Рішення 2026-08-20:** варіант **C** — E1 = лише **P0 + GUI**; P1 відкладено на **E1.1**.

Заборонено в default Desktop: `--allow-public-bind`, публічний STUN default, авто-trust невідомих peers.

## 4. Редакції

| | AIRA Desktop | AIRA Dev |
|--|--------------|----------|
| Вхід | іконка / tray | GUI + повний CLI |
| Root | OS app-data path (див. phase-e) | `--root` / кілька профілів |
| First run | wizard → init + identity | CLI без змін |
| Мережа | профілі з §3 | усі CLI-прапорці |

## 5. Послідовність ОС

```text
E0 (код, усі ОС) → E1 Linux (P0) → E1.1 P1+QR → E2 macOS → E3 Windows
```

Один UI-шар (кандидати: Tauri або egui tray); не три окремі GUI.

## 6. Мінімальні Settings (E1)

| Key | Default | Примітка |
|-----|---------|----------|
| `network_profile` | `P0` | інші значення — після E1.1 |
| `open_ui_on_start` | `true` | |
| `autostart_on_login` | `false` | |
| `http_listen` | `127.0.0.1:8787` | |
| `peer_listen` | n/a у P0 | |

## 7. Посилання

- Local HTTP: [`local-node.md`](local-node.md)
- Peer (для майбутніх P1+): [`peer-link.md`](peer-link.md)
- systemd (сервери, не Desktop): [`runbook-systemd.md`](runbook-systemd.md)
