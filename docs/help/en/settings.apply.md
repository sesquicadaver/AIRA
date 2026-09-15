# Apply settings

## What is this?

Settings are **saved** to disk when you change them. Some values (especially network profile and listen) are **applied** to the running program only after Stop node → Start node. Closing the window does **not** stop AIRA.

**Settings ≠ System:** edit network profile and peer listen under **Settings → Connection**. **System → Connection** shows observed status, invites, and one corrective CTA — not a second settings form.

## What to do?

1. Change **General** options (language, open UI on start, login autostart) anytime — they apply without a node restart badge.
2. Edit **Connection** (profile, peer listen, P3|P4) in Settings. Compare **Saved** vs **Applied**. If Applied shows **not confirmed**, Start node (or wait for status) so values come from the running node — not from disk alone.
3. When you see **Restart needed**, go to **System → Program**, **Stop node**, then **Start node**.

## What should happen?

Immediate prefs take effect now. Network/listen stay marked until a successful Start node syncs applied runtime settings.

## If it failed?

If Start node after a profile change fails, read the problem and help:node.lifecycle. For what the profile means, see help:network.connect. First-run orientation — help:start.

## Related

- help:node.lifecycle — Stop node / Start node to apply
- help:network.connect — connection settings
- help:start — getting started after apply
