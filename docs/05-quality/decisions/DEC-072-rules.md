---
id: DEC-072
status: accepted
---

# DEC-072 Rules and workflows from declared triggers, conditions and actions, run as jobs

- Date: 2026-11-18
- Context: Automations written as plugins for every small need are costly; free scripting inside the platform is a security and stability risk.
- Options considered: plugins only; a scripting language; typed rules built from declarations (chosen).
- Decision:
  - Modules declare rule triggers (event types), conditions and actions with schemas.
  - A rule combines them; it runs after commit as a job with the permissions of its owner.
  - Chains are limited in depth and a rule cannot trigger itself; every run is recorded with its outcome.
- Consequences: Common workflows (enrol on completion, notify on a late submission, add to a group) need no code and are auditable.
- Affected artifacts: `crates/customise`, every module that declares triggers, conditions or actions.
