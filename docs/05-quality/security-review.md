# Security review of 0.1.0

Date: 2026-11-23. Scope: the whole core of 0.1.0 (every crate under `crates/`, the web client under `web/`, the deployment under `deploy/`), against OWASP ASVS 4.0.3 level 2 (RNF-001).

Result: **no open finding of high or medium severity**. Every finding below is fixed in 0.1.0; the residual risks are accepted and listed at the end.

## Method

| Part | How |
| --- | --- |
| Dependencies | `cargo audit` (RustSec) and `cargo deny` (advisories, licences, bans, sources) on the workspace; `npm audit --omit=dev` on the web client: no advisory |
| Code | Review of authentication, sessions, access control, input handling, files, the API, configuration and the deployment, chapter by chapter of ASVS; a static rule for connections held across service calls (`tools/repo-check`) |
| Running system | The production Compose deployment behind Caddy: headers of pages and API answers, cookies, the content security policy in a browser (no violation on any page), sign-in throttling behind proxies |
| Load | The load test of RNF-004, which found the starvation of the connection pool (S-02); see [performance](./performance.md) |
| Accessibility | axe on every page in both schemes, in the end-to-end tests |

## Findings

| Id | Severity | ASVS | Finding | Resolution |
| --- | --- | --- | --- | --- |
| S-01 | High | V2.2.1, V7.1 | The API took the address of a request from its connection, which behind the web client or a load balancer is always the proxy. Fifty failed sign-ins from anyone locked the address of everyone, and sessions and the audit log recorded the proxy | `server.trusted_proxies`: only a peer in those networks may tell the address with `X-Forwarded-For`, read from the right, skipping trusted hops; none are trusted by default. The deployments set it to their internal networks. Unit tests cover spoofed and chained headers |
| S-02 | High | V11.1.4 (availability) | Reading the layout of the home, the menu of a new campus and the second-factor check at sign-in held a database connection while the permission check took another. Under load every connection ended up waiting for a second one, and requests failed after 30 s. The same pattern was in 54 changes, inside their transactions | Checks and reads on the held connection (`require_in`, `check_in`, `course_in`, `section_in`, `policy_in`, `any_in`, `graded_accounts_in`), which also makes them see what the transaction locked. `tools/repo-check` refuses the pattern, and a server test serves the pages of the client with a pool of one connection |
| S-03 | Medium | V8.2.1 | Answers of the API, most with personal data, carried no caching rule, so browsers and shared caches could keep them | `Cache-Control: no-store` on every answer of the API that does not set its own |
| S-04 | Medium | V14.4.3 | Pages had no content security policy | A policy per request with a nonce: scripts and style sheets only from the client or with the nonce (`strict-dynamic`), no plugins, no framing, forms and connections to the same origin |
| S-05 | Low | V9.1.1 | Pages sent no `Strict-Transport-Security` | Two years, sent on every page |
| S-06 | Low | V2.1 | A password could hold spaces or other blank characters, which cannot be seen and are mistyped | Rejected with the rule `whitespace` on every change; the client checks it as it is typed |

## Controls verified

| Area | What holds |
| --- | --- |
| Passwords | Argon2id with a decoy hash for unknown accounts, so timing tells nothing; minimum length and the names of the account refused; at most 1024 characters |
| Sign-in | The same answer whatever failed; throttling by name, account and address, with locks; second factors (authenticator app, code by email, recovery codes) and roles that require them |
| Sessions | Random tokens stored as SHA-256 hashes; cookie `HttpOnly`, `Secure` on HTTPS, `SameSite=Lax`; changes with the cookie require the `X-Kampus` header; idle and absolute lifetimes; changing the password ends every session and token; sensitive changes need a recent sign-in |
| Links by email | One-time tokens, hashed, with lifetimes; in the fragment of the address, so never in a server log or a `Referer`; recovery and sign-up answer the same whether the email has an account; a new email is confirmed with a button, not on opening |
| Access control | Every route checks its permission in the scope of the record; lists filter by the scopes allowed before paging; view-as is read-only; frozen scopes refuse changes; decisions can be explained |
| Input and output | Every query parameterised (sqlx); validation errors name fields and rules; problem details without internals, with a request id; JSON only |
| Files | Read only by who sees what owns them; links signed for five minutes; content that could run is downloaded, never rendered, with `nosniff` and a sandboxing policy; optional malware scanning; quotas |
| Secrets | Second-factor secrets sealed with XChaCha20-Poly1305 under the secret key of the installation; secrets only in variables or a file readable by Kampus; never logged |
| Headers of pages | CSP, HSTS, `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`, `Permissions-Policy` |
| Deployment | Images run as an unprivileged user; the Helm chart drops every capability, mounts a read-only root and runs with the default seccomp profile; only the proxy is published |

## Residual risks

| Risk | Why accepted |
| --- | --- |
| `style-src-attr 'unsafe-inline'` | Components of the design system set style attributes; attributes cannot run code |
| Images from any HTTPS address | A theme can point its icons and brand images elsewhere; images cannot run code |
| `/metrics` without authentication | It holds counts and route templates, no personal data; it is not served through the web client and the guides tell operators to keep it inside their network |
| No general rate limit on the API | Sign-in, recovery and codes by email are throttled; a general limit belongs to the proxy or the load balancer in 0.1.0 |
