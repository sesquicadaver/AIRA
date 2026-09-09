# Connect to participants

## What is this?

**Connection** is how this computer joins other AIRA participants. Default profile **P0** is local-only. Address-book rows are saved participants — not live sessions. UNKNOWN is not the same as OFFLINE.

Use this topic when you want a working path to another person: invite → peer-capable profile → Stop/Start → honest status.

## What to do?

Canonical scenario (no need to open Technical details for the basic path):

1. **Import an invite** — on **System → Connection**, use **Import invite…** / Import JSON or QR (friend invite). That trusts a known peer; discovery alone is not trust (see help:network.trust).
2. **Enable a peer profile** — choose **P1** or **P2** (private network with peer listen). P0 stays local-only. If the Connection button says **Enable private network**, use it.
3. **Apply with Stop → Start** — if Settings or Connection shows **Restart needed**, open **System → Program**, press **Stop**, then **Start**. Closing the window does not stop AIRA.
4. **Check status** — read the human Connection summary and, if needed, press **Refresh**. For observed reachability details, see help:network.reachability.

Never enable public bind or auto-trust strangers from Help — those remain opt-in CLI / advanced paths under Technical details.

## What should happen?

After invite + P1/P2 + restart you should see: Saved ≠ Applied until runtime confirms; restart clears “Restart needed”; book count may grow without inventing live sessions; Connection shows one next-step CTA when something is still missing.

## If it failed?

- No invite / empty book → import again (help:network.trust).
- Still P0 / “Enable private network” → switch to P1 or P2, then restart.
- Restart needed persists → Stop then Start (help:settings.apply, help:node.lifecycle).
- Banner stays unknown after peer profile → Refresh and help:network.reachability (UNKNOWN ≠ OFFLINE).

## Related

- help:network.trust — invites and trust vs discovery
- help:network.reachability — observed mesh status
- help:settings.apply — Saved vs Applied / restart
- help:node.lifecycle — Start / Stop
