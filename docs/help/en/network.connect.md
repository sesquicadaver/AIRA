# Connect to participants

## What is this?

**Connection** is how this computer prepares to work with other AIRA participants. Default profile **P0** is local-only. Address-book rows are saved participants — not live sessions. UNKNOWN is not the same as OFFLINE.

Use this topic for the **setup** path: invite → peer-capable profile → Stop/Start → honest status. That path configures trust and local networking. It does **not** by itself complete an authenticated remote peer session with another computer.

## Boundary (read this)

- **Setup ≠ remote session.** Import invite + P1/P2 + Stop/Start applies local settings and may grow the address book. Live sessions appear only when a session is actually observed — not because the setup steps finished.
- **Loopback ≠ dial address.** Desktop default listen is loopback-bound. An exported listener endpoint on loopback is **not** an address another computer can dial. Do not treat Stop/Start success as “connected to someone else.”
- Public bind and auto-trust stay opt-in advanced/CLI paths — Help never enables them as the default.

A confirmed remote dial path is **opt-in** under **Technical details → Peer dial**: trusted peer identity + **explicit** dial address. The dial attempt runs **off the UI thread** so Help (F1) and navigation stay available while it is in progress. A failed dial **restores** the previous AddressBook entry (trial address does not quietly replace a known-good one). Success stores handshake evidence and may show a **last confirmed handshake** (session closed after the check) — it does **not** invent a live session or DIRECT reachability from setup alone, and Desktop listen stays loopback by default.

## What to do?

Canonical setup scenario (no need to open Technical details for the basic path):

1. **Import an invite** — on **System → Connection**, use **Import invite…** / Import JSON or QR (friend invite). That trusts a known peer; discovery alone is not trust (see help:network.trust).
2. **Enable a peer profile** — choose **P1** or **P2** (private network with peer listen). P0 stays local-only. If the Connection button says **Enable private network**, use it.
3. **Apply with Stop → Start** — if Settings or Connection shows **Restart needed**, open **System → Program**, press **Stop**, then **Start**. Closing the window does not stop AIRA.
4. **Check status honestly** — read the human Connection summary and, if needed, press **Refresh**. Expect Saved ≠ Applied until runtime confirms; book growth without inventing live sessions. For observed reachability details, see help:network.reachability.

## What should happen?

After invite + P1/P2 + restart you should see: Saved ≠ Applied until runtime confirms; restart clears “Restart needed”; book count may grow without inventing live sessions; Connection shows one next-step CTA when setup is still incomplete. You should **not** assume a remote peer session solely because those steps succeeded on loopback.

## If it failed?

- No invite / empty book → import again (help:network.trust).
- Still P0 / “Enable private network” → switch to P1 or P2, then restart.
- Restart needed persists → Stop then Start (help:settings.apply, help:node.lifecycle).
- Banner stays unknown after peer profile → Refresh and help:network.reachability (UNKNOWN ≠ OFFLINE).
- Expected “connected to another PC” but only loopback listen → this Help boundary: setup finished ≠ remote dial.

## Related

- help:network.trust — invites and trust vs discovery
- help:network.reachability — observed mesh status (sessions vs book)
- help:settings.apply — Saved vs Applied / restart
- help:node.lifecycle — Start / Stop
