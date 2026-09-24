# specs/

Working copies and additive engineering RFCs for this repository.

**`#386` rule:** there is **one** content canon. This tree is **not** a second Manifesto and **not** a competing standard. Do not rewrite `Manifesto etc/` from routine QUEUE atoms.

## Immutable baseline

| Layer | Location | Role |
|-------|----------|------|
| Manifesto corpus | `Manifesto etc/` | **Immutable** source of truth for Books / Schema Pack prose / Conformance / SCD. Engineering PRs do not edit it in place. |
| Machine schemas | `schemas/**/*.json` | Canonical `$id` bodies (one `$id` ↔ one file; see `#385`). |
| Working Book / pack copies | `specs/book-*.md`, `specs/schema-pack.md`, `specs/conformance.md`, `specs/specification-control.md`, … | Implementation convenience copies. Update by re-copying from Manifesto after **approved** corpus changes — not by inventing divergent prose here. |

What this tree implements versus Books / schemas / C0–C1: [`docs/implementation-status.md`](../docs/implementation-status.md).

## Amendments (RFCs)

Additive, versioned changes live under [`specs/rfc/`](rfc/). Process: [`rfc-process.md`](rfc-process.md).

- RFCs **amend** the baseline for engineering; they do **not** replace Manifesto as the corpus root.
- Accepted RFCs bind QUEUE atoms and Living Spec rows; rejected / withdrawn / superseded RFCs do not.
- Schema-bearing RFCs must land the machine body under `schemas/` with the same `$id` (pack excerpt must match — `#385`).

## Priority (conflict resolution)

When texts disagree, apply in this order (highest first):

1. **`Manifesto etc/`** normative Books 0–III + SCD + Conformance (immutable baseline).
2. **`schemas/**/*.json`** for any schema `$id` (machine authority over pack prose copies).
3. **Accepted RFCs** in `specs/rfc/` that explicitly amend the affected surface (later accepted RFC supersedes earlier on the same scope when `supersedes` / consolidating close says so).
4. **Working copies** under `specs/*.md` (Book / schema-pack / conformance mirrors) — convenience only; must not invent a second canon.
5. **`docs/`** operational guides and plans — describe the repo; they do not outrank Manifesto / schemas / accepted RFCs.

Book IV remains reference (non-normative). Book V / Research stay non-operational except via promotion ([`docs/rfc-p-promotion.md`](../docs/rfc-p-promotion.md)).

## RFC metadata (`class` / `status`)

Every RFC **MUST** declare process metadata per [`rfc-process.md`](rfc-process.md) §15:

| Field | Meaning |
|-------|---------|
| `class` | Primary change class: `A` \| `B` \| `C` \| `D` \| `S` \| `T` \| `E` \| `R` \| `P` \| `X` (exactly one primary class). |
| `status` | Lifecycle: `draft` \| `review` \| `accepted` \| `rejected` \| `superseded` \| `withdrawn`. |

Engineering convention in this repo: many RFCs open with a Summary section and state class in the title (e.g. `RFC-D`) or body; new RFCs should include the YAML metadata block. Living Spec / QUEUE tip authority is still [`QUEUE.md`](../QUEUE.md) — RFCs do not invent a parallel “first OPEN.”

## Non-goals for `#386`

```text
Rewriting Manifesto etc/
Minting a second schema canon under specs/
Changing federation / settlement runtime
Mass RFC renumbering or status backfill
```
