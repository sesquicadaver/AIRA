# Living Spec Matrix — Analyze-78

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Canonical manifest sign | `CsuManifest::resign_canonical` / `basic_manifest` | sample + C1 manifests |
| Canonical verify on register | `validate_for_registration` | unsigned / TESTSIG / mutation |
| Publisher override re-signs | `apply_publisher` | tenant HTTP register; publisher emit tests |
| Field mutation breaks verify | same | `canonical_verify_fails_when_manifest_fields_change` |
