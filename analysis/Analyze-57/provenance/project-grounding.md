# Project grounding — Analyze-57 / QUEUE #22

**Date:** 2026-08-01  
**Why:** User flagged context loss; re-read QUEUE, A-47, peer-link, Book II/III, code before any further interview.

## TOR / ТЗ у цьому репо

Файлів `TOR_*` у дереві AIRA **немає**. Еквівалент технічного завдання для peer-контуру:

| Джерело | Роль |
|---------|------|
| `QUEUE.md` | Канон Phase B; #22 = атомарний scope |
| `Analyze-47/README.md` | DHT-lite closed; явно відклав `--apply-book` |
| `docs/peer-link.md` | Контракт: book = dial authority; DHT = advisory |
| `Analyze-43/.../ADR-connectivity-relay-first.md` | Порядок: gossip → relay → DHT |
| `specs/book-ii.md` / Book III | Discovery MAY: DHT / address book; exact DHT design Out of Book II |
| `Manifesto etc/**` | Читати можна; **не редагувати** |

## Архітектурна модель (зафіксовано)

```
TrustStore  →  хто допущений
AddressBook →  куди dial (єдине джерело dial)
DHT (dht.json) →  advisory lookup memory (XOR closest)
Discovery journal →  observational only
```

Announce (`peer.dht.announce`) уже: пише в `dht.json`, fan-out по **існуючому** book.  
Find уже: exact + closest **лише друк**.  
Жоден шлях **не** мутує book без оператора — це навмисно (A-47 Out: auto address-book mutation).

## Що саме є #22

QUEUE: *Opt-in: результат DHT find/announce → upsert у `address_book.json`*  
Done when: *CLI flag + тест dial після upsert*  
Не в рядку: *discv5; auto без flag*

A-47 RALPLAN уже назвав deferred UX:

> find prints; optional `--apply-book` later Out

Тобто #22 — **не новий винахід**, а закриття Out з A-47 під іменем `--apply-book`, у межах QUEUE (find **і** announce).

## Висновок для імплементації (proposed crystallize)

Без повторного «A/B/C про trigger» — витягнуто з канону:

1. **Прапор:** `--apply-book` (як у A-47).
2. **`peer dht find --apply-book`:** upsert **лише exact** hit `identity→addr` у book; closest лишаються print-only (не промоутити здогад XOR).
3. **`peer listen --dht --apply-book`:** після успішного `apply_dht_announce` додатково upsert того ж identity/addr у book.
4. **Overwrite** addr для існуючого peer; `via` **зберігати**, якщо запис уже був (relay-first A-44 не ламати).
5. **Без flag** — поведінка A-47 без змін.
6. **Тест:** announce/find з flag → запис у book → `dial` успішний.
7. **Out:** discv5, auto без flag, dial напряму з DHT, iterative FIND_NODE (#33).

## Єдина залишкова неоднозначність

Чи `--apply-book` на listen (inbound announce) входить у #22 разом із find, чи QUEUE «find/announce» означає лише CLI `dht find` + CLI `dht announce` (локальний self у book уже є)?

Рекомендація з QUEUE-літералу + A-47: **find + inbound apply** (п.2–3). Потрібне лише підтвердження / заперечення.
