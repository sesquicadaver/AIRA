# Initial Issue Set (аналіз)

**Джерело:** `Manifesto etc/AIRA Initial Issue Set v0.1.md` (1985 рядків)  
**Type:** Engineering / Backlog  
**Confidence:** High

## Evidence

- **§1–3:** GitHub-ready backlog; labels (type/priority/profile/status/risk); milestones.
- **Epics 0–11 / Issues #1–#80:**
  - E0 Bootstrap #1–5
  - E1 Spec snapshot #6–8 (Books + governance + terminology)
  - E2 Schema #9–21
  - E3 C0 Core #22–26
  - E4 Artifact/Event/Policy #27–34
  - E5 CSU Runtime #35–40
  - E6 Basic CSU #41–46
  - E7 Operational flow #47–56 (incl. demos)
  - E8 CLI/Node #57–62
  - E9 Conformance C0/C1 #63–70
  - E10 Partial local C2 #71–75
  - E11 Alpha #76–80
- **§16 PR sequence:** PR-001…016 mapping issues.
- **§17 Critical path:** #1→#3→#20→#25→#28+#31+#33+#34→#36+#38+#39→#41…#45→#47…#53→#64→#65→#80.
- **§18–19:** Issue writing rules; **MVP Backlog Freeze** до #80 (no federation/GPU/LLM/chain/PHM/Research/UI/cloud).
- **§20 Status next:** MVP PRD (ordering note: PRD already exists in corpus).

## Inference

Канонічний execution backlog для runtime. Analyze-7 **не** створює GitHub issues — лише карту. Freeze rule = soft gate для наступного autopilot impl.
