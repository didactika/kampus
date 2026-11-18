---
id: DEC-041
status: accepted
---

# DEC-041 Self-enrolment with limits in the transaction, hashed keys, approvals and payments

- Date: 2026-11-18
- Context: Self-enrolment is exposed to everyone: capacities are crossed by concurrent requests when they are counted first, and keys stored in clear and hinted are easy to guess.
- Options considered: count then insert; a queue in front of self-enrolment; a lock on the instance in the creating transaction with hashed keys and throttling (chosen).
- Decision:
  - Capacity, opening period and restrictions are checked with a lock on the instance in the transaction that creates the enrolment.
  - Keys are stored hashed, compared in constant time, never hinted; attempts are throttled per person and per address; a group key places the person in that group.
  - Approval creates a request decided by people with `enrol.request.decide`; payment-gated methods enrol only after the payment port confirms.
- Consequences: Capacities hold under load; keys resist guessing; paid and approved courses use the same model.
- Affected artifacts: `crates/enrol`, the payment port, the course page.
