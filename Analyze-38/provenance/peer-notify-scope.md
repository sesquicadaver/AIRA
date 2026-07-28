# Provenance — peer pubkey notify

Op: trust-delta `rekey` (same identity_id, new pubkey).
Notify: address-book dial **before** rotate cutover (`--notify-peers`).
Out: gossip fanout; remote TrustStore dual-key; notify-after-rotate.
