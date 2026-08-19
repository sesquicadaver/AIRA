# ULTRAQA — Analyze-65

**Verdict:** PASS  
**Date:** 2026-08-17

## Hostile checks
| Scenario | Result |
|----------|--------|
| JSON-only init + load | OK (`init_writes_json_not_yaml`) |
| YAML ≡ JSON default | OK (`load_config_yaml_matches_json`) |
| Both files present | Fail-closed (`load_config_both_fail_closed` + CLI status error) |
| YAML-only open/status | OK |
| Init on YAML-only | No `config.json` written |
| aira-cli/aira-node compile | OK |

## Commands
```bash
cargo test -p aira-flow --lib
cargo check -p aira-cli -p aira-node
# smoke: init → config.json only; yaml-only status; both → error
```
