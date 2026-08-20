# UltraQA — Analyze-102 / QUEUE #67

| ID | Scenario | Result | Notes |
|----|----------|--------|-------|
| U1 | deny no policy | pass | no share-offer pointer |
| U2 | allow no activate | pass | `NoActivated` |
| U3 | visibility=global | pass | `BadVisibility` |
| U4 | allow + activate → publish | pass | ModelArtifact + ShareOffer CAS; Event |
| U5 | gate-only no offer | pass | `request_publish` unchanged |
| U6 | clippy -D warnings | pass | acquisition + cli |
