# Apply settings

## What is this?

Settings go through an honest lifecycle: **Changed** (draft in the UI) → **Saved** (on disk) → **Applied** (confirmed by a running node) → or **Restart needed** when Saved differs from Applied for network/listen. Closing the window does **not** stop AIRA.

**Settings ≠ System:** edit network profile and peer listen under **Settings → Connection**. **System → Connection** shows observed status, invites, and one corrective CTA — not a second settings form.

## What to do?

1. Change **General** options (language, open UI on start, login autostart) anytime — they apply without a node restart badge.
2. Edit **Connection** (profile, peer listen, P3|P4) in Settings. If the badge says **Changed**, use **Save draft** or **Cancel draft** — Cancel restores disk values.
3. Compare **Saved** vs **Applied**. If Applied shows **not confirmed**, Start node (or wait for status) so values come from the running node — not from disk alone.
4. When you see **Restart needed**, go to **System → Program**, **Stop node**, then **Start node**.
5. If **Apply error** appears under the Settings badge, fix the value (for example peer listen) and Save again — the message is the concrete failure, not a silent success.

## What should happen?

Immediate prefs take effect now. Unsaved Connection edits stay **Changed** until Save or Cancel. Network/listen stay marked until a successful Start node syncs applied runtime settings.

## If it failed?

- Apply/persist failed → read the **Apply error** line in Settings; do not assume the disk was updated.
- If Start node after a profile change fails, read the problem and help:node.lifecycle.
- For what the profile means, see help:network.connect. First-run orientation — help:start.

## Related

- help:node.lifecycle — Stop node / Start node to apply
- help:network.connect — connection settings
- help:start — getting started after apply
