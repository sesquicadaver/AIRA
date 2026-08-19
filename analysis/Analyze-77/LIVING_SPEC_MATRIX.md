# Living Spec Matrix — Analyze-77

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Canonical Object sign | `ObjectDescriptor::attach_canonical_signature` | `example_problem` + plane submit |
| Canonical Object verify | `verify_canonical` / `admit_object` | `canonical_verify_fails_when_object_fields_change` |
| Store admission | Memory + SQLite `create` | `create_rejects_unsigned_and_mutated_object_signature` |
| Plane production sign | `OperationalPlane::submit_problem` | `local_session_submit_signs_with_node_identity` |
