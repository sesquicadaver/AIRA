# Trust

## What is this?

Trust decides whom you accept as a participant. Discovery alone is not trust. Desktop must never auto-trust strangers.

## What to do?

1. On **System → Connection** (primary surface — not only Technical details), import a friend invite (JSON or QR) when you intend to trust that peer.
2. Follow the connect scenario: after import, enable **P1/P2** if still on P0, then **Stop → Start** when restart is needed — see help:network.connect.
3. Keep higher profiles and advanced discovery fail-closed until you configure them deliberately.

## What should happen?

**DISCOVERED** is not **TRUSTED**. Imported invites appear in trust / address-book tooling without inventing live sessions. Connection may show **Import invite…** as the next step when the book is empty. Completing invite import is setup of trust — not proof of a live remote session (see help:network.connect boundary).

## If it failed?

If import fails, check the invite file or QR and retry. Reachability problems are separate — see help:network.reachability. Profile / restart path — help:network.connect and help:settings.apply.

## Related

- help:network.connect — invite → profile → restart **setup** (setup ≠ remote session; loopback ≠ dial)
- help:network.reachability — observed sessions vs book
- help:settings.apply — when settings need restart
