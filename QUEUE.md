# AIRA — оптимізована черга задач

**Оновлено:** 2026-07-28  
**База:** Analyze-20…33 CLOSED (APPROVE/CLEAR); `main` @ peer CLI  
**Правило:** один активний Analyze-N slice за цикл; не чіпати `Manifesto etc/**`, `Meditation_About/**`.

## Поточний стан

| Гілка | Стан |
|-------|------|
| MVP Epic 0–11 + Alpha.2 / Keyring / Plane / Trust / Peer P0+CLI | **done** |
| Відкритий Analyze-34+ | **Analyze-42 CLOSED**; наступний = #10 NAT / gossip / DHT |
| Канон наступних milestone | `specs/mvp-roadmap.md` M12–M13 + post-MVP |

Дубльовані пункти з `Analyze-*/todo/TODO_FIXME.md` зведено тут. Старі TODO у Analyze-N лишаються як provenance; **черга виконання = цей файл**.

## Stale (не брати в роботу)

| Пункт | Чому |
|-------|------|
| A-31 «Blocking: restore_previous after trust» | Виправлено в A-31 CODE_REVIEW + `rotate_node_signing_secret` |
| A-30 «Optional durable backup of previous secret» | Зроблено в A-31 (`--backup` / `.prev`) |
| A-32 «CLI peer add/dial/send» | Зроблено в A-33 |

## Активна черга (оптимізована)

Порядок = залежності + demo value. Один рядок = один Analyze-цикл (або явно «micro»).

| # | Pri | Slice (пропозиція) | Scope | Залежить від | Не в scope |
|---|-----|--------------------|-------|--------------|------------|
| 1 | **P0** | ~~Analyze-34 — Peer listen daemon~~ **DONE** | Persistent multi-accept listen + CLI; dial smoke без recv | A-33 CLI | Noise, NAT, gossip |
| 2 | **P0** | ~~Analyze-35 — Noise XX~~ **DONE** | Noise XX під тим самим frame/envelope + CLI `peer *` | #1 (стабільний listen для smoke) | Trust-delta, DHT |
| 3 | **P1** | ~~Analyze-36 — Trust-delta over peer~~ **DONE** | CRL / trust-delta message по peer link | #2 (краще шифр. payload) або мінімум #1 | Auto-notify UX |
| 4 | **P1** | ~~Analyze-37 — Dual-key node grace~~ **DONE** | Multi-pubkey Keyring для того ж node `key_ref` | crypto в `aira-object` (паралельно до #2–3) | Peer notify |
| 5 | **P1** | ~~Analyze-38 — Peer pubkey notify~~ **DONE** | Auto-notify peers про новий node pubkey | #3 + #4 | Gossip fanout |
| 6 | **P1** | ~~Analyze-39 — CSU emit_failed publisher~~ **DONE** | Довести publisher lifecycle (A-29 deferred) | незалежно | Peer |
| 7 | **P2** | ~~CRL / ceremony audit log~~ **DONE** (Analyze-40) | rotate/revoke/unrevoke → durable audit | після #4 | — |
| 8 | **P2** | ~~Timestamped `.prev` history~~ **DONE** (Analyze-41) | історія бекапів, не один слот | після #4 | — |
| 9 | **P2** | ~~Multi-tenant per-CSU keyring~~ **DONE** (Analyze-42) | ізоляція keyring | після #6 | — |
| 10 | **P2** | NAT / gossip / DHT | федерація-lite | після #2–3 | не MVP |
| 11 | **P2** | HTTP TLS / persist discovery | A-19 deferred | окремий трек | peer P0 |
| 12 | **P2** | Roadmap M12–M13 / v0.2 C2 | conformance harness + partial C2 protocols | окремий трек | peer crypto |

### Рекомендований наступний цикл

**Analyze-43 / #10 — NAT / gossip / DHT** (федерація-lite; не MVP-критично).

### Залежності (скорочено)

```text
34 daemon+dial-smoke
  └─► 35 Noise XX
        └─► 36 trust-delta
              └─► 38 peer notify
37 dual-key ── (паралельно з 35/36) ──► 38
39 CSU publisher ── незалежно
7–12 ── після стабілізації trust/peer або окремі треки
```

## Anti-patterns черги

- Не змішувати Noise + NAT + DHT в одному Analyze.
- Не стартувати dual-key і Noise в одному PR без жорсткого ADR split.
- Не піднімати P2 (gossip/DHT) до закриття P0 peer encrypt.
- Не відкривати новий Analyze, поки попередній без APPROVE/CLEAR.

## Джерела

- `Analyze-30…33/todo/TODO_FIXME.md`, README Follow-up
- `docs/peer-link.md`, `docs/crypto.md` (Out of scope → черга)
- `specs/mvp-roadmap.md` §18–20, post-MVP
