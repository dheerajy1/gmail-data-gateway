# FEATURES.md: jobs-api-gateway-rust

Owns the **inventory of capabilities**: what exists, what it must do to count as done, and what it depends on. Behavior details live in `SPEC.md`, reasons in `DECISIONS.md`, build order in `PLAN.md`.

Last updated: 2026-11-06

## How to read this file

- Scope is product features plus foundation features (chosen by you).
- Every feature is `PLANNED`: no Rust code exists yet. Nothing here has been compiled or run.
- **Reference** names the file in `recordlog-api-rust` (or another repo) that is the model. These are `FACT` from reading the repos; each phase re-reads the reference before copying a pattern.
- **Done when** lists acceptance criteria that a test or a manual check can decide. Items marked `UNVERIFIED` depend on facts not yet checked (see `SPEC.md` section 14).
- IDs are stable. `PLAN.md` refers to them.

## Feature map

| Group | IDs |
| --- | --- |
| Foundation | F01 to F11 |
| Product (API) | F20 to F26 |
| Subscriber | F30 to F34 |
| Tooling | F40, F41 |
| Delivery | F50 to F53 |

## Foundation

### F01: Workspace and build setup

- **Description:** Cargo workspace with the root crate (library plus binary), the `subscriber` member and the `xtask` member; cargo aliases; shared lint settings.
- **Source:** `SPEC.md` section 3; D8 (`sqlx` 0.9), D9 (`tracing`).
- **Reference:** `recordlog-api-rust/Cargo.toml`, `subscriber/Cargo.toml`, `xtask/Cargo.toml`, `.cargo/config.toml`.
- **Done when:**
  - `cargo check --workspace` passes.
  - The three members build.
  - Dependency versions are pinned in the workspace manifest and `Cargo.lock` is committed.
- **Depends on:** none.

### F02: Configuration and env validation

- **Description:** Load environment variables at start-up and fail with a clear message when one is missing or invalid.
- **Source:** `SPEC.md` section 10; D13 (`APP_ENV`).
- **Reference:** `src/config/env.rs`, `src/schemas/env.rs`.
- **Done when:**
  - Every variable in `SPEC.md` section 10 is validated with the stated rule.
  - A missing or out-of-range value stops start-up and names the variable.
  - `.env.production` and `.env.development` use `APP_ENV`, not `NODE_ENV`.
  - Secrets are never printed in logs or errors.
- **Depends on:** F01.

### F03: Logging

- **Description:** Structured logs with `tracing`, timestamps in IST, one format for API and subscriber.
- **Source:** D9. The format and filter are `UNKNOWN` (`SPEC.md` section 14).
- **Reference:** `kafka-api-gateway-rust/src/main.rs` and its logging setup; recordlog uses `println!` and is not the model.
- **Done when:**
  - API and subscriber log with `tracing`.
  - The log level is configurable through the environment.
  - No secret, token, mail text or password reaches the logs.
- **Depends on:** F01, F02.

### F04: Response envelope and error catalog

- **Description:** Success and error envelopes and the catalog of error codes.
- **Source:** `SPEC.md` sections 4.5 and 11; D19 (only used entries), D22 (`statusCode`).
- **Reference:** `src/schemas/response.rs`, `src/errors/catalog.rs`, `code.rs`, `api_error.rs`.
- **Done when:**
  - Success serializes as `success`, `statusCode`, `message`, `data`.
  - Error serializes as `success`, `statusCode`, `code`, `message`, `error`.
  - The catalog contains exactly the entries listed in `SPEC.md` section 11, with the HTTP code of each.
  - A test checks that every catalog entry maps to the stated HTTP status.
- **Depends on:** F01.

### F05: Global error handling

- **Description:** Unknown path returns 404, wrong method returns 405, a panic returns 500, and validation failures return 400, all in the error envelope.
- **Source:** `SPEC.md` section 4.2 and 4.5.
- **Reference:** `src/errors/fallback.rs`, `panic.rs`; `tests/errors.rs`.
- **Done when:**
  - The four cases return the envelope with the right code.
  - A panic inside a handler does not take the process down.
- **Depends on:** F04.

### F06: Middleware (CORS, access gate, API authentication)

- **Description:** CORS rules, the docs access gate, the client-header check and the JWT check.
- **Source:** `SPEC.md` sections 4.2 to 4.4.
- **Reference:** `src/middleware/*`; `tests/cors.rs`, `gate.rs`, `auth_extractor.rs`.
- **Done when:**
  - CORS allows no-origin requests, `http://localhost:3000` and the four suffix origins, and rejects others.
  - The gate follows the four-step order in `SPEC.md` section 4.2.
  - Each authentication failure in `SPEC.md` section 4.3 returns its listed code and status.
- **Depends on:** F02, F04, F05.

### F07: Azure SQL access and user lookups

- **Description:** Connection handling for Azure SQL and the lookups for portal users and API users.
- **Source:** `SPEC.md` section 9.1; D7 (shared tables, bcrypt only).
- **Reference:** `src/db/azure.rs`, `api_users.rs`, `portal_users.rs`.
- **Done when:**
  - The API connects with the public login and the subscriber with the service login (`SPEC.md` section 10).
  - A user lookup returns the user or a not-found result without leaking whether the user exists.
  - Passwords are checked with `bcrypt` only.
  - Table columns are copied from the recordlog queries (`UNKNOWN` until read).
- **Depends on:** F02.

### F08: Kafka gateway client

- **Description:** Publish one event to the gateway and validate its reply.
- **Source:** `SPEC.md` section 7; D21 (reads `statusCode`).
- **Reference:** `src/clients/kafka_gateway.rs`.
- **Done when:**
  - The request carries the four auth headers and the body `topic`, `correlationId`, `value` only.
  - The reply is accepted only when it parses, `success` is true and `statusCode` equals 202.
  - A non-2xx status, a non-JSON body or a failed check returns a publish error.
  - Tests use a fake gateway. The real gateway now sends `statusCode` (your rename, D29), not yet tested; the live check is in P11.
- **Depends on:** F02, F04.

### F09: OpenAPI documentation

- **Description:** `/openapi` page generated from the code, title "Gmail data gateway API", version `v1`.
- **Source:** `SPEC.md` section 4.1.
- **Reference:** `src/openapi/*`; `tests/openapi.rs`.
- **Done when:**
  - Every route in `SPEC.md` section 4.1 that belongs to the API is documented with request and response examples using `statusCode`.
  - The error responses of each route are listed.
- **Depends on:** F04, F06.

### F10: Static assets and page layout

- **Description:** Embedded `global.css`, `logo.png`, `favicon.ico` and the shared page layout.
- **Source:** `SPEC.md` sections 3 and 4.1; D1 (Vercel does not compile Tailwind).
- **Reference:** `src/components/layout.rs`, `src/tailwind.css`, `public/`.
- **Done when:**
  - The three assets are served without authentication.
  - `public/global.css` is committed and matches the Tailwind source.
- **Depends on:** F01.

### F11: Test harness

- **Description:** Shared test helpers and fakes so each feature is tested without real Azure, Gmail or Kafka.
- **Source:** D15 (tests run before delivery when the toolchain installs).
- **Reference:** `tests/common/mod.rs` and the recordlog test files.
- **Done when:**
  - Helpers build the app with fake configuration and a fake gateway.
  - There is a fake for Gmail and one for the Azure procedures used by the API.
  - `cargo test --workspace` runs offline.
- **Depends on:** F01 to F08.

## Product (API)

### F20: Docs pages and docs login

- **Description:** HTML pages `/`, `/home`, `/about`, `/login` and the cookie login.
- **Source:** `SPEC.md` sections 4.1 and 4.2.
- **Reference:** `src/pages/*`, `src/components/*`; `tests/login.rs`.
- **Done when:**
  - A correct portal login sets the cookie `auth=ok` and redirects.
  - A wrong login shows the error without revealing which part was wrong.
  - Pages behind the gate redirect to `/login` without the cookie.
- **Depends on:** F06, F07, F10.

### F21: Health endpoints and pages

- **Description:** `/healthz`, `/healthz/app`, `/healthz/db` pages and `GET /api/v1/health`.
- **Source:** `SPEC.md` section 4.1.
- **Reference:** `src/features/v1/health/*`, `src/components/healthz*.rs`; `tests/health.rs`.
- **Done when:**
  - The app check reports process status without secrets.
  - The database check runs a trivial query against Azure and reports success or failure.
  - The JSON route returns the success envelope.
- **Depends on:** F04, F07.

### F22: API login

- **Description:** `POST /api/v1/auth/login` returns a JWT.
- **Source:** `SPEC.md` section 4.3.
- **Reference:** `src/features/v1/auth/*`; `tests/auth.rs`.
- **Done when:**
  - A valid user gets a JWT with `sub`, `role`, `iat`, `exp` (1 day) and issuer `jobs-api-gateway`.
  - Wrong user or wrong password returns the same response.
  - The token verifies in F06.
- **Depends on:** F06, F07.

### F23: `extractMails` request and query building

- **Description:** Validate the body and build the Gmail query for each mode.
- **Source:** `SPEC.md` sections 5.1 and 5.2.
- **Reference:** none in recordlog (jobs-specific); the old `extractMails.ts` and `fetch-messages.ts`.
- **Done when:**
  - `Bulk` rejects any date field, `Sync` rejects any date field, `Increment` requires at least one date.
  - `Sync` reads the last stored date and builds `after:<unix>`; an empty table returns 404 `NO_EMAIL_DATA`.
  - `Increment` builds `after` and `before` as given.
  - The date text format returned by the database is confirmed before the parser is written (`UNVERIFIED`).
- **Depends on:** F04, F06, F07.

### F24: Gmail credentials and token refresh

- **Description:** Read the active OAuth credentials from Azure and exchange the refresh token for an access token.
- **Source:** `SPEC.md` sections 5.3 to 5.5; D4 (direct REST).
- **Reference:** old `getAccessToken.ts` and `gmail.ts` (behavior only).
- **Done when:**
  - Credentials come from `05spgetActiveGoogleOAuthCreds` with `Google_APPID`.
  - The access token is cached until it expires and refreshed once on a 401.
  - Gmail 400 or 401, 403 and 429 map to the codes in `SPEC.md` section 5.5.
  - Endpoint details are checked against Google's documentation first (`UNVERIFIED`).
- **Depends on:** F02, F04, F07.

### F25: Gmail reading and record normalization

- **Description:** Page through the list, read the metadata of each mail, clean the text, convert dates and drop incomplete mails.
- **Source:** `SPEC.md` section 5.3.
- **Done when:**
  - Page size 100, pause 300 ms between pages, groups of 20 with a 200 ms pause.
  - With a limit, collection stops early and the list is cut to the limit.
  - A mail with an empty `From`, subject, snippet or date, an unparsable date or a failed read is dropped.
  - Dates look like `2026-11-06T10:15:00+05:30`.
  - Tests use the Gmail fake and cover the cleaning rules.
- **Depends on:** F24, F11.

### F26: Chunked publish and `extractMails` response

- **Description:** Reverse to oldest first, split into chunks of 100, publish each chunk, answer with `correlationIds`.
- **Source:** `SPEC.md` sections 5.6 and 6; D3, D10, D11, D12, D22.
- **Done when:**
  - 250 records publish 3 events of 100, 100 and 50, oldest first, each with its own UUIDv7.
  - The response has `correlationIds`, `recordCount` 250, `chunkCount` 3, `status` `ACCEPTED`.
  - Zero records return 200 with `status` `NO_NEW_RECORDS`, an empty list and nothing published.
  - If chunk 2 of 3 fails, chunk 3 is not sent, the error response says 1 chunk was already published, and the code is `KAFKA_GATEWAY_FAILED`.
- **Depends on:** F08, F23, F25.

## Subscriber

### F30: Subscriber runtime and transport

- **Description:** One process, one authenticated WebSocket to the gateway, three subscribed topics, reconnect and shutdown behavior.
- **Source:** `SPEC.md` section 8.1.
- **Reference:** `subscriber/src/main.rs`, `transport.rs`, `gateway.rs`.
- **Done when:**
  - The subscriber connects with the `x-subscriber-id` identity and subscribes to the three topics.
  - A dropped connection reconnects.
  - Frame types and the acknowledgement frame are read from `transport.rs` first (`UNKNOWN` until read).
- **Depends on:** F01, F02, F03.

### F31: Event models, retry and dead letter

- **Description:** Event and status models with validation, the retry counter and the dead-letter publish.
- **Source:** `SPEC.md` sections 6 and 8.3.
- **Reference:** `subscriber/src/event.rs`, `retry.rs`, `failure.rs`, `status.rs`.
- **Done when:**
  - Validation matches the field rules in `SPEC.md` section 6, including UUIDv7 and at least one record.
  - A failure at count 0, 1 or 2 republishes to the same topic with the incremented `x-jobs-retry-count`.
  - A failure at count 3 or more publishes to the dead-letter topic with the five headers.
  - The best-effort `FAILED` status never throws.
- **Depends on:** F30.

### F32: Write-model handler (Azure)

- **Description:** Validate, publish status, call the Azure procedure, republish to the read topic, acknowledge.
- **Source:** `SPEC.md` sections 8.2 and 9.1.
- **Reference:** `subscriber/src/handlers/write_model.rs`, `azure.rs`.
- **Done when:**
  - The status sequence is `RECEIVED`, `AZURE_WRITE_STARTED`, `AZURE_WRITE_SUCCEEDED`.
  - A replay of the same `correlationId` is success.
  - The procedure name check rejects names outside the allowed pattern.
- **Depends on:** F31.

### F33: Read-model handler (PostgreSQL)

- **Description:** Validate, publish status, call the PostgreSQL procedure inside a transaction, acknowledge.
- **Source:** `SPEC.md` sections 8.2 and 9.2; D8 (`sqlx` 0.9).
- **Reference:** `subscriber/src/handlers/read_model.rs`, `postgres.rs`.
- **Done when:**
  - The status sequence is `READ_MODEL_STARTED`, `READ_MODEL_SUCCEEDED`, `COMPLETED`.
  - A result other than `SUCCESS` is a failure and goes through F31.
  - The 0.8-to-0.9 differences are checked against the kafka gateway's use (D8).
- **Depends on:** F31.

### F34: Status handler

- **Description:** Apply each status event with `fn_pipeline_apply_status`.
- **Source:** `SPEC.md` sections 8.2 and 9.2.
- **Reference:** `subscriber/src/handlers/status.rs`.
- **Done when:**
  - The original `eventId` is passed unchanged so redelivery is a replay.
  - On failure the message is not acknowledged and no dead-letter message is sent.
- **Depends on:** F31, F33.

## Tooling

### F40: `xtask gen-token`

- **Description:** Interactive Google consent flow that stores the refresh token in Azure.
- **Source:** D5, D14; `SPEC.md` section 12.
- **Reference:** old `generate-token.ts`.
- **Done when:**
  - The command prints the consent URL with scopes `gmail.readonly` and `drive`.
  - It exchanges the pasted code and calls `04spupsertGoogleOAuthToken` for `Google_APPID`.
  - Tokens are never printed in full.
- **Depends on:** F02, F07, F24.

### F41: `xtask` helpers

- **Description:** Docker and project helper commands.
- **Source:** `SPEC.md` section 12 (`PROPOSED`, exact list `UNKNOWN`).
- **Reference:** `recordlog-api-rust/xtask/src/main.rs`, `.cargo/config.toml`.
- **Done when:** The same cargo aliases exist as in recordlog (`FACT`): `xtask`, `css`, `dev`, `start`, `build-prod`, `dev-subscriber`, `build-subscriber`, `start-subscriber`, and `docker-build`, `docker-up`, `docker-down`, `docker-start`, `docker-stop`, `docker-restart`, `docker-logs`, `docker-logs-clear`, `docker-rmi`, `docker-clean`, adjusted to the jobs image and service names. The core aliases (`css`, `dev`, `start`, `build-prod`) are built in P1 because your verify uses them (D24).
- **Depends on:** F01.

## Delivery

### F50: Docker images and compose

- **Description:** Subscriber image and compose file for on-prem; the API image if needed.
- **Source:** D1.
- **Kept files (`OBSERVED`, read):** `docker/Dockerfile` and `docker/Dockerfile.subscriber` still build with Bun (`oven/bun` image, `bun install`, `bun build --compile`). `docker/docker-compose.yml` still sets `NODE_ENV` and has the API service commented out. All three must be rewritten for Rust. `recordlog-api-rust` has the same three files plus a `.dockerignore`, which jobs does not have yet (`OBSERVED`).
- **Done when:**
  - The subscriber image builds from the workspace and starts with the env file.
  - The compose file runs the subscriber, keeps the API service disabled and uses `APP_ENV`.
  - A `.dockerignore` exists.
- **Depends on:** F30 to F34.

### F51: CI and release

- **Description:** GitHub workflows for tests, pull-request checks and releases, plus release config.
- **Kept files (`OBSERVED`, read):** `.github/workflows/test.yml` calls an external reusable workflow `test-rust.yml` in your `.github` repository (its content is not in the tarballs, `UNKNOWN`). `release.yml` still sets up Bun and ends with `bunx semantic-release`, which cannot work because the Rust repo has no `package.json` or `bun.lock` (`FACT`: file comparison). It is replaced by the `kafka-api-gateway-rust` version (D25, untested until you run it in GitHub Actions containers). `.releaserc.json` is identical to kafka's and already uses `semantic-release-cargo`. `pr-checks.yml`, `CODEOWNERS`, `labeler.yml` and the PR template need no Rust change that I have seen.
- **Done when:**
  - The test workflow runs fmt, check, clippy and test for the workspace.
  - Release config produces versions and `CHANGELOG.md`.
- **Depends on:** F01, F11.

### F52: Vercel deployment

- **Description:** Deploy the API on Vercel with the Rust runtime.
- **Source:** D1.
- **Kept file (`OBSERVED`, read):** `vercel.json` with `framework: rust` and no build overrides. `recordlog-api-rust` also has a `.vercelignore`, which jobs does not have yet (`OBSERVED`).
- **Done when:**
  - The API crate builds for Vercel and answers `GET /api/v1/health`.
  - The real function time limit for the Rust runtime is checked and recorded (D2 risk).
- **Depends on:** F01 to F10, F21.

### F53: Repository docs refresh

- **Description:** Update the kept `README.md`, `structure.txt`, `proj-flowchart.txt` and `CHANGELOG.md` so they describe the Rust project.
- **Kept files (`OBSERVED`):** all four exist; whether they describe the TypeScript version is `UNVERIFIED`.
- **Done when:** The files describe the Rust workspace, the flow in `SPEC.md` section 2 and the build commands in `BUILD.md`.
- **Depends on:** all other features.

## Dropped or unchanged

| Item | Status | Source |
| --- | --- | --- |
| Seven old dev scripts | Not built | D6 |
| `jobs_data_db` and `02sjobsData` scripts | Unchanged unless an error shows a change is needed | D18 |
| Upload-style or other new product features | Not planned | none |

## Dependency summary

- Foundation first: F01, F02, F04, F05, F06, F07, F08, F10, F11.
- API features: F20 to F22 need F06 and F07; F23 to F26 are a chain (F23, F24, F25, F26).
- Subscriber features are independent of the API after F01 to F03 and F07: F30, F31, then F32 to F34.
- Delivery features follow the code they package.

## Risk flags (`UNVERIFIED` items that can change a feature)

| Feature | Open fact |
| --- | --- |
| F08, F26 | The gateway rename to `statusCode` is done (D29) but untested; the live publish check is in P11. |
| F23 | Text format of the last stored date. |
| F24 | Google endpoint and error details. |
| F26 | Vercel time limit and Kafka partition count (D2, D10). |
| F30 | WebSocket path and frame details. |
| F33 | `sqlx` 0.9 API differences (D8). |
| F50 | The three Docker files are still Bun builds and must be rewritten. |
| F51 | The external reusable workflow `test-rust.yml` is not visible, and the kafka `release.yml` is untested (D25). |
