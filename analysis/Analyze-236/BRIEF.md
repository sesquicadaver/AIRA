# BRIEF — Analyze-236

Seal `object_store_access::mint` so it is not on the default `aira-object` prelude. Store backends only in `aira-core` via feature `store-backend`. Do not change VRA payload (`#202`).
