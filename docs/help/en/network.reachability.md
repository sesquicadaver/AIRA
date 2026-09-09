# Reachability

## What is this?

Reachability is the **last observed** mesh conclusion: direct, relayed, outbound-only, local-only, unknown, or offline when that was actually observed. Stale or unknown must not be painted as a confirmed global link.

## What to do?

1. Open **System → Connection**.
2. Follow the connect scenario first if you have not yet imported an invite or left P0 — help:network.connect (invite → P1/P2 → Stop/Start).
3. Press **Refresh** (or the Connection next-step when it says so) to request a new observation.
4. Read the human summary and observation time — not only the address-book count. Raw enums live under Technical details.

## What should happen?

You see an honest conclusion. AddressBook rows are not counted as live sessions. UNKNOWN is not OFFLINE. Strip Network language agrees with Connection. Finishing the connect **setup** steps does not invent a remote session; loopback listen is not a peer dial target (help:network.connect).

## If it failed?

- UNKNOWN persists after a peer profile → check peer listen, firewall, and help:network.connect / help:node.lifecycle.
- Program stopped → help:node.lifecycle.
- Profile just changed → help:settings.apply (Restart needed).
- Empty book / no trust → help:network.trust.

## Related

- help:network.connect — invite → profile → restart **setup** (setup ≠ remote; loopback ≠ dial)
- help:network.trust — trust vs discovery
- help:node.lifecycle — Start / Stop / Refresh
