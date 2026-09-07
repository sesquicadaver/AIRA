# Reachability

## What is this?

Reachability is the **last observed** mesh conclusion: direct, relayed, outbound-only, local-only, unknown, or offline when that was actually observed. Stale or unknown must not be painted as a confirmed global link.

## What to do?

1. Open **System → Connection**.
2. Press **Refresh** to request a new observation.
3. Read the banner and observation time — not only the address-book count.

## What should happen?

You see an honest banner. AddressBook rows are not counted as live sessions. UNKNOWN is not OFFLINE.

## If it failed?

- UNKNOWN persists → check peer listen, firewall, and whether the profile even starts peer.
- Program stopped → help:node.lifecycle.
- Profile just changed → help:settings.apply (Restart needed) and help:network.connect.

## Related

- help:network.connect — profiles and participants
- help:network.trust — trust vs discovery
- help:node.lifecycle — Start / Stop / Refresh
