---
id: DEC-064
status: accepted
---

# DEC-064 Webhooks as event deliveries, signed, permission-filtered and protected against internal addresses

- Date: 2026-11-18
- Context: Webhooks fired inside the request slow it and lose events; unsigned payloads can be forged; URLs pointing to internal addresses expose the network.
- Options considered: calls in the request; a separate queue; webhooks as listeners of the event system (DEC-055) (chosen).
- Decision:
  - A webhook is a listener of its client: deliveries are at least once, ordered per object, with retries and back-off.
  - Only event types and payload fields the client may see are sent; payloads are signed with an HMAC of the body, a timestamp and the event id.
  - URLs must be HTTPS and must not resolve to private or loopback addresses, checked again at delivery time.
  - Repeated failures disable the webhook and notify its owner.
- Consequences: Reliable, verifiable notifications to external systems without new infrastructure.
- Affected artifacts: `crates/api`, `crates/events`.
