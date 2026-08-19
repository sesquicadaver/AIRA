# Living Spec Matrix — Analyze-68

| ТЗ | Модуль | Тести |
|----|--------|-------|
| FIND/NODES signed UDP | `discv` schemas find/nodes | `iterative_find_a_via_b_stores_c` |
| Listen multiplex | `handle_discv_datagram` | `handle_still_stores_announce` |
| Iterative XOR | `iterative_discv_find` | `iterative_find_a_via_b_stores_c` |
| Untrusted requester | `verify_find` | `find_untrusted_requester_times_out` |
| Untrusted hint skip | `merge_nodes_into_store` | `nodes_skip_untrusted_hint` |
| No apply-book | merge path | roundtrip + iterative assert empty book |
