# TODO_FIXME — Analyze-62

- [ ] QUEUE #28 tenant rotate / revoke ceremony (audit + CLI; not raw `save_*`)
- [ ] Ceremony: revoke = `unregister_csu_tenant` + delete `identity/tenants/<hex>/` (trust CRL alone insufficient)
- [ ] Rename order: secret before meta; optional `--force` on register overwrite
- [ ] Per-tenant load errors: log-and-continue (avoid silent partial + fail-fast clash with `let _`)
- [ ] Optional: `--secret-hex-file` / stdin; dir mode `0700`
