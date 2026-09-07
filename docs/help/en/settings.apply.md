# Apply settings

## What is this?

Settings are **saved** to disk when you change them. Some values (especially network profile and listen) are **applied** to the running program only after Stop → Start. Closing the window does **not** stop AIRA.

## What to do?

1. Change **General** options (language, open UI on start, login autostart) anytime — they apply without a node restart badge.
2. For **Connection** / listen, compare **Saved** vs **Applied** on Settings. If Applied shows **not confirmed**, Start (or wait for status) so values come from the running node — not from disk alone.
3. When you see **Restart needed**, go to **System → Program**, **Stop**, then **Start**.

## What should happen?

Immediate prefs take effect now. Network/listen stay marked until a successful Start syncs applied runtime settings.

## If it failed?

If Start after a profile change fails, read the problem and help:node.lifecycle. For what the profile means, see help:network.connect. First-run orientation — help:start.

## Related

- help:node.lifecycle — Stop / Start to apply
- help:network.connect — connection settings
- help:start — getting started after apply
