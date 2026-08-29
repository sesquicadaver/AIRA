# AIRA-RFC-0080 — Settlement receipt store (append-only JSONL)

## 1. Summary

Phase H `#173`: local `SettlementReceiptStore` in `aira-protocol` persists Book II §15 Settlement Receipts as append-only JSONL (`settlement/receipts.jsonl`) with store schema tag `aira:settlement:receipts-jsonl:v1`. Append and open/get re-verify canonical Ed25519 over the receipt body bound to `provider_identity`. Not a blockchain ledger.

## 5. Non-Goals

B2-011 privacy redaction smoke (`#174`); `run_c4` (`#175`); status PARTIAL (`#176`); federation/remote settlement.

## 10. Behavior

```text
open_or_create(root) → empty JSONL + STORE_SCHEMA
append(receipt) → admit (privacy_class, refs, verify_canonical) → append line
  same receipt_id + identical body → idempotent OK
  same receipt_id + different body → Duplicate
open(root) / get(id) → verify-on-read each receipt; tamper → InvalidSignature
```

## 15. Tests

```text
cargo test -p aira-protocol settlement_receipt_store_
```
