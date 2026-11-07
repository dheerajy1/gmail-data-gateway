# PLAN.md: jobs-api-gateway-rust

Owns the **order of work**: build phases, what each one delivers, and how to know it is finished. What the system does is in `SPEC.md` and `FEATURES.md`; why is in `DECISIONS.md`; how to run, verify and deliver a phase is in `BUILD.md`.

Last updated: 2026-11-07

## Rules for all phases

- **Order** is thin slice first (`DECIDED`, D23), then the rest by dependency.
- **Size** is cut by feature, with no cap on the number of files or changes (`DECIDED`, D17).
- **Delivery** is one full-repo tarball per phase, excluding `target` and `.git`, including all `.env*` files (`DECIDED`, D16).
- **Checks** before delivery: Claude tries to install the Rust toolchain and runs `cargo check` and `cargo test` (`DECIDED`, D15). If the install fails, the phase says so and nothing is described as compiled or tested.
- **Final verify** is run by you before every commit, using your command sequence (`DECIDED`, D24; the exact commands are in `DECISIONS.md` and `BUILD.md`).
- **Commit** happens only after the phase delivered and your verify passed. Commit text follows the format in `BUILD.md`.
- **References** are re-read at the start of each phase. A pattern is copied only after reading the reference file again.
- **Decisions** made during a phase are written to `DECISIONS.md` in the same phase.
- **Status:** P1 is `VERIFIED` (2026-11-07: fmt, check, clippy and 16 tests pass on your machine, and `GET /api/v1/health` works on the Vercel preview). P2 is `VERIFIED` (2026-11-07: fmt, check, clippy and 64 tests pass on your machine; `cargo build-prod` and `cargo start` work; `GET /api/v1/health` answers through the dev tunnel). P3 is `VERIFIED` (2026-11-07: fmt, check, clippy and 90 tests pass on your machine; `cargo build-prod` and `cargo start` work; docs login sets the cookie, API login returns a token, and `GET /api/v1/health` reports the database UP against the real Azure database). P4 to P11 are `NOT STARTED`.

## Phase map

| Phase | Name | Features | Main risk |
| --- | --- | --- | --- |
| P1 | Thin slice: health on Vercel | F01, F02, F03, F04, F05, F10, F11 (minimal), F21 (app part), F41 (core aliases), F52 (minimal) | Vercel Rust runtime, crate versions |
| P2 | Middleware and API docs | F06, F09, F11 (grow) | Gate and CORS rules |
| P3 | Azure access, login and health database check | F07, F20, F21 (database part), F22 | User table columns |
| P4 | Kafka gateway client | F08, F11 (fake gateway) | Shape of the gateway reply `data` |
| P5 | `extractMails`: request and Gmail credentials | F23, F24 | Date format, Google endpoints |
| P6 | `extractMails`: reading and chunked publish | F25, F26 | Time limit, chunk behavior |
| P7 | Subscriber: runtime, models, retry | F30, F31 | WebSocket frames |
| P8 | Subscriber: handlers | F32, F33, F34 | `sqlx` 0.9, procedures |
| P9 | `xtask` | F40, F41 (remaining aliases) | Google OAuth flow |
| P10 | Docker, CI and repo docs | F50, F51, F53 | Bun leftovers |
| P11 | Live verification | none (checks only) | Real gateway and Vercel limits |

Dependencies: P2 needs P1. P3 needs P2. P4 needs P1. P5 needs P3. P6 needs P4 and P5. P7 needs P1 and P3. P8 needs P7. P9 needs P3 and P5. P10 needs P8. P11 needs everything.

## P1: Thin slice: health on Vercel

- **Goal:** A deployed API that answers `GET /api/v1/health` on Vercel, built from the new workspace.
- **Features:** F01 (root crate, `xtask` member and workspace shell; `subscriber` is added in P7), F02 (all variables validated), F03, F04, F05, F10, F11 (minimal harness), F21 (the app check and the JSON route only), F41 (core aliases `css`, `dev`, `start`, `build-prod`, because your verify uses them, D24), F52 (minimal).
- **Includes housekeeping:**
  - Rename `NODE_ENV` to `APP_ENV` in `.env.production` and `.env.development` (D13).
  - Add `.vercelignore` (recordlog has one).
  - Keep `public/global.css` committed.
- **Entry criteria:** `DECISIONS.md`, `SPEC.md`, `FEATURES.md`, `PLAN.md`, `BUILD.md`, `CODING-STANDARDS.md` exist and are approved. `Cargo.toml` does not exist yet.
- **Exit criteria:**
  - `cargo check` and `cargo test` pass for the workspace (or the phase states they could not run).
  - Start-up fails with a clear message when a variable is missing.
  - `GET /api/v1/health` returns the success envelope with `statusCode`.
  - Unknown path returns the 404 envelope; wrong method returns 405.
  - The Vercel deployment answers the health route.
- **Manual checks (yours):** deploy and call the health route; record the real function time limit (feeds D2).
- **Unknowns to settle first:** allowed `APP_ENV` values, `tracing` format, crate versions (check current versions before pinning).
- **Commit scope suggestion:** `feat(api)`.

## P2: Middleware and API docs

- **Goal:** CORS, access gate, client-header check, JWT check and the `/openapi` page.
- **Features:** F06, F09, F11 (more helpers).
- **Reference:** `src/middleware/*`, `src/openapi/*`, `tests/cors.rs`, `gate.rs`, `auth_extractor.rs`, `openapi.rs`.
- **Exit criteria:**
  - Every authentication failure in `SPEC.md` section 4.3 returns its listed code and status.
  - The gate follows the four-step order in `SPEC.md` section 4.2.
  - CORS allows only the origins in `SPEC.md` section 4.4.
  - `/openapi` renders and lists the routes that exist.
- **Scope notes (2026-11-07):** P2 also adds JWT verification (D36), `wants_html` (D37), `EXPIRED_TOKEN` (D38), constant-time comparison (D35, D39). `src/openapi/errors.rs` of the reference moves to P3, because it documents errors per route and P2 has no route with documented errors.
- **Commit scope suggestion:** `feat(middleware)`.

## P3: Azure access, login and health database check

- **Goal:** Azure connection, user lookups, docs login, API login and the database health check.
- **Features:** F07, F20, F21 (database part and pages), F22.
- **Reference:** `src/db/*`, `src/features/v1/auth/*`, `src/pages/*`, `src/components/*`, `tests/login.rs`, `auth.rs`, `health.rs`.
- **Needs from you before the phase:** the columns of `01tportalUsers` and `02tapiUsers` (they are not in the provided tarballs). The recordlog queries show what the code expects; confirm they match.
- **Exit criteria:**
  - Docs login sets the cookie and redirects; wrong login does not reveal which part was wrong.
  - API login returns a JWT that the F06 extractor accepts.
  - `/healthz/db` reports success against a real Azure database.
- **Manual checks (yours):** log in against the real database (the only place user rows exist).
- **Scope notes (2026-11-07):** P3 also adds the per-route error examples (`src/openapi/errors.rs`, moved here from P2), `GET /home`, the `isActive = 1` filter (D41), one 401 and one bcrypt check for every failed login (D42), `data.db` and the 503 in `GET /api/v1/health` (D43), the page name (D44) and the crates (D45). The service database pool and its login are not added yet; they come with the subscriber (F08).
- **Build note:** the new pages use new Tailwind classes, so `cargo build-prod` rewrites `public/global.css`; commit that file with the phase (D1: Vercel does not compile Tailwind).
- **Commit scope suggestion:** `feat(auth)`.

## P4: Kafka gateway client

- **Goal:** Publish one event to the gateway and validate the reply.
- **Features:** F08, F11 (fake gateway).
- **Reference:** `src/clients/kafka_gateway.rs`, `kafka-api-gateway-rust/src/features/v1/http_publish/*`.
- **Exit criteria:**
  - Request body has `topic`, `correlationId`, `value` only.
  - Reply is accepted only when `success` is true and `statusCode` equals 202 (D21).
  - Tests with the fake gateway cover success, non-2xx, non-JSON and a wrong `statusCode`.
- **Live checks:** the gateway now sends `statusCode` (your rename, D29, not tested). Live checks wait for P11.
- **Unknown to settle first:** the shape of the `data` object in the gateway reply.
- **Commit scope suggestion:** `feat(gateway)`.

## P5: `extractMails`: request and Gmail credentials

- **Goal:** Validate the request, build the Gmail query for each mode, read credentials and refresh the access token.
- **Features:** F23, F24.
- **Needs before the phase:** the text format returned by `[02sjobsData].[03spgetLastDateAppliedJobs]` (run it or read its remaining lines), and Google's current documentation for the token and Gmail endpoints (checked by Claude before writing code).
- **Exit criteria:**
  - The three modes validate and build queries as in `SPEC.md` section 5.2.
  - `Sync` on an empty table returns 404 `NO_EMAIL_DATA`.
  - Credentials load with `Google_APPID`; the access token is cached and refreshed on 401.
  - Gmail 400 or 401, 403 and 429 map to the listed codes.
- **Manual checks (yours):** run a token refresh against your real Google app.
- **Commit scope suggestion:** `feat(extract)`.

## P6: `extractMails`: reading and chunked publish

- **Goal:** Page through Gmail, normalize records and publish chunks to the gateway.
- **Features:** F25, F26.
- **Exit criteria:**
  - The reading rules in `SPEC.md` section 5.3 hold (page size, pauses, groups of 20, drop rules, IST dates).
  - 250 records publish 3 events of 100, 100 and 50, oldest first; the response has `correlationIds`, `recordCount`, `chunkCount`, `status`.
  - Zero records return `NO_NEW_RECORDS` and publish nothing.
  - A failure on chunk 2 stops the request, reports the published count and uses `KAFKA_GATEWAY_FAILED`.
- **Manual checks (yours):** a small `Increment` run against the real mailbox and gateway, after P11's gateway prerequisite.
- **Risk to record:** the measured time of a `Bulk` run on Vercel (D2).
- **Commit scope suggestion:** `feat(extract)`.

## P7: Subscriber: runtime, models, retry

- **Goal:** The subscriber process connects to the gateway and handles events through validation, retry and dead letter.
- **Features:** F30, F31.
- **Reference:** `subscriber/src/*` in `recordlog-api-rust`. Read `transport.rs` and `gateway.rs` first for the frame details (`UNKNOWN` until read).
- **Includes:** add the `subscriber` member to the workspace.
- **Exit criteria:**
  - Connect, subscribe to three topics, reconnect on a dropped connection.
  - Event and status validation match `SPEC.md` section 6.
  - Retry increments `x-jobs-retry-count`; at 3 the message goes to the dead-letter topic with the listed headers.
- **Commit scope suggestion:** `feat(subscriber)`.

## P8: Subscriber: handlers

- **Goal:** Write-model, read-model and status handlers.
- **Features:** F32, F33, F34.
- **Needs first:** compare the recordlog subscriber's `sqlx` 0.8 code with the kafka gateway's 0.9 use (D8).
- **Exit criteria:**
  - Status sequences match `SPEC.md` section 8.2.
  - A replayed `correlationId` is treated as success.
  - A result other than `SUCCESS` goes through retry and dead letter.
  - Status handler does not dead-letter.
- **Manual checks (yours):** run the subscriber against the real databases with one chunk published by hand.
- **Commit scope suggestion:** `feat(subscriber)`.

## P9: `xtask`

- **Goal:** `gen-token` and the helper commands.
- **Features:** F40, F41 (the subscriber aliases and any alias not yet added).
- **Exit criteria:**
  - `cargo gen-token` (alias to be added) prints the consent URL with scopes `gmail.readonly` and `drive`, exchanges the code and stores the token through `04spupsertGoogleOAuthToken`.
  - The remaining helper aliases listed in `FEATURES.md` F41 work (`docker-*` aliases arrive with their images in P10).
- **Manual checks (yours):** one real consent run.
- **Commit scope suggestion:** `feat(xtask)`.

## P10: Docker, CI and repository docs

- **Goal:** Rust images, compose, CI that fits the Rust repo, and updated repository docs.
- **Features:** F50, F51, F53.
- **Known work (`OBSERVED`):**
  - Rewrite `docker/Dockerfile`, `docker/Dockerfile.subscriber` and `docker/docker-compose.yml` (still Bun and `NODE_ENV`).
  - Add `.dockerignore`.
  - Replace `release.yml` with the `kafka-api-gateway-rust` version (D25); `.releaserc.json` is already identical to kafka's. The kept file still uses Bun and `bunx semantic-release`, which cannot work without a `package.json`.
  - Update `README.md`, `structure.txt`, `proj-flowchart.txt` and `CHANGELOG.md` (their current content has not been read).
- **Exit criteria:**
  - The subscriber image builds and starts with the env file.
  - CI test workflow passes on a pull request.
  - Repo docs describe the Rust project.
- **Commit scope suggestion:** `chore(docker)` and `docs`.

## P11: Live verification

- **Goal:** Check what tests with fakes cannot.
- **Prerequisite (yours):** the `statusCode` rename in `kafka-api-gateway-rust` and `recordlog-api-rust` is done (D29) and still needs a first real run.
- **Checks:**
  - A real publish through the gateway returns `statusCode` 202 and reaches the subscriber.
  - A full run: `extractMails` to Azure and PostgreSQL, with status events recorded.
  - Vercel function time limit measured against a `Bulk` run (D2).
  - Kafka topic partition count recorded (D10 residual risk).
- **Output:** results recorded in `DECISIONS.md` as decisions or open items.

## Standing checks at the end of every phase

1. `FEATURES.md` entries touched are updated with what changed.
2. New decisions are in `DECISIONS.md`.
3. `SPEC.md` is changed only if the behavior changed.
4. The tarball contains `.env*` files and excludes `target` and `.git` (D16).
5. The commit text uses the heading counts from your rule.

## Open items that affect the plan

| Item | Affects |
| --- | --- |
| Your test of the kafka-style `release.yml` in GitHub Actions containers (already in recordlog) | P10 |
| Allowed `APP_ENV` values and `tracing` format | P1 |
| User table columns | P3 |
| `data` shape of the gateway reply | P4 |
| Date format from `03spgetLastDateAppliedJobs` | P5 |
| WebSocket frame details | P7 |
| Your untested `statusCode` rename across the repos (D29) | P4 live checks, P11 |