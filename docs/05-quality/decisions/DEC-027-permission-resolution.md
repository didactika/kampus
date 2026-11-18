---
id: DEC-027
status: accepted
---

# DEC-027 Permission resolution: allow, deny, forbid, and no bypass

- Date: 2026-11-18
- Context: The check is called by every operation (DEC-017). It must be predictable for people who configure roles, explainable (RF-023), and safe: no account may skip it, and nobody may lock a campus out of its own administration.
- Options considered: allow and deny only; numeric priorities; three values with "forbid wins, then any allow" and administration as a role (chosen).
- Decision:
  - Values are allow, deny and forbid; no row means inherit. For each granted role the first value from the scope up to the campus wins; any forbid denies; otherwise any allow allows; otherwise deny.
  - Administration is a role in the platform, tenant or campus scope, checked like any other; there is no bypass list.
  - A change that would leave a campus without anyone able to manage access is refused.
  - Guests never get write or risky permissions; suspended and deleted accounts fail every check; frozen scopes allow only reads.
  - Every decision can be explained with the grants, values and scopes that produced it.
- Consequences: One rule that people can learn and that the explanation shows; administrators act within audited permissions; lock-out is impossible by construction.
- Affected artifacts: `crates/access`, the permissions screens, the handbook.
