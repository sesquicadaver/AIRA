# Desktop PeerInvite file IO (QUEUE #83)

## Export

```bash
aira desktop invite-export --out ~/alice.invite.json
# P1: includes peer_listen as addr by default
aira desktop invite-export --out ~/alice.invite.json --addr 127.0.0.1:9797
```

## Import

```bash
aira desktop invite-import --file ~/alice.invite.json
```

Applies `trust add` (identity_ref + public_key_hex). If `addr` is set, also upserts `peers/address_book.json`.

Schema: [`aira:schema:desktop:peer-invite:0.1`](../schemas/desktop/peer-invite.schema.json). QR is `#84`.
