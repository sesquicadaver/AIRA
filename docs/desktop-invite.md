# Desktop PeerInvite file + QR (QUEUE #83 / #84)

## File export / import

```bash
aira desktop invite-export --out ~/alice.invite.json
# P1: includes peer_listen as addr by default
aira desktop invite-export --out ~/alice.invite.json --addr 127.0.0.1:9797

aira desktop invite-import --file ~/alice.invite.json
```

## QR PNG (no camera)

Encode compact PeerInvite JSON into a PNG; decode from an image file on disk.

```bash
aira desktop invite-export-qr --out ~/alice.invite.png
aira desktop invite-export-qr --out ~/alice.invite.png --addr 127.0.0.1:9797

aira desktop invite-import-qr --file ~/alice.invite.png
```

Import applies `trust add` (identity_ref + public_key_hex). If `addr` is set, also upserts `peers/address_book.json`.

Schema: [`aira:schema:desktop:peer-invite:0.1`](../schemas/desktop/peer-invite.schema.json). RFCs: RFC-0032 (file), RFC-0033 (QR). GUI: [`desktop-gui.md`](desktop-gui.md) / `#85`.
