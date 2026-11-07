# DECISIONS.md: jobs-api-gateway-rust

Owns the **why** behind material decisions. The current resulting state lives in `SPEC.md`.

Last updated: 2026-11-07

## Status legend

| Tag | Meaning |
| --- | --- |
| `DECIDED` | You (the project owner) selected it. Build exactly this. |
| `PROPOSED` | Recommended, not yet approved. |
| `ASSUMPTION` | Relied on but not verified. |
| `UNKNOWN` | Unresolved. |
| `CONFLICT` | Evidence or documents disagree. |
| `OBSERVED` | Seen directly in the repositories. |

## Sources of truth

| Source | Role |
| --- | --- |
| `jobs-api-gateway` (ElysiaJS, Bun, TypeScript) | Behavior to replicate. A decision below that deviates from it wins. |
| `jobs-api-gateway-rust` | Target repo. Legacy TypeScript already removed. No Rust code exists yet. |
| `recordlog-api` and `recordlog-api-rust` | Reference pair. Same architecture: API on Vercel, subscriber on-prem in Docker. |
| `kafka-api-gateway` and `kafka-api-gateway-rust` | Reference pair for the Kafka gateway the jobs API publishes to. |
| `jobs_data_db` (PostgreSQL) and `02sjobsData` (Azure SQL) | Database scripts. The Azure scripts define the write-model contract. |

Nothing in this file has been compiled or run. Everything marked as read was read from source files, not executed.

## Decision index

| ID | Title | Status |
| --- | --- | --- |
| D1 | Hosting: API on Vercel, subscriber on-prem Docker | `DECIDED` |
| D2 | `extractMails` stays synchronous | `DECIDED` |
| D3 | `extractMails` publishes in chunks of 100 records | `DECIDED` |
| D4 | Gmail access through direct REST calls with `reqwest` | `DECIDED` |
| D5 | `generate-token` ported as an `xtask` command | `DECIDED` |
| D6 | Seven old dev scripts are dropped | `DECIDED` |
| D7 | Shared Azure database and user tables, bcrypt-only passwords | `DECIDED` |
| D8 | `sqlx` 0.9 for the subscriber | `DECIDED` (against recommendation) |
| D9 | Logging with `tracing` | `DECIDED` (details open) |
| D10 | Chunks are published oldest first | `DECIDED` |
| D11 | A failed chunk stops the request | `DECIDED` |
| D12 | `extractMails` returns a `correlationIds` list | `DECIDED` |
| D13 | The environment variable is `APP_ENV` | `DECIDED` |
| D14 | `generate-token` requests the `gmail.readonly` and `drive` scopes | `DECIDED` (against recommendation) |
| D15 | Code phases try to install the Rust toolchain and run `cargo check` and `cargo test` | `DECIDED` |
| D16 | Each phase is delivered as one full-repo tarball | `DECIDED` |
| D17 | Phases are cut by feature and pattern, with no cap on changes | `DECIDED` |
| D18 | Database scripts stay unchanged unless an error shows a change is needed | `DECIDED` |
| D19 | Only error catalog entries the Rust code uses are ported | `DECIDED` |
| D20 | The `PROPOSED` items in `SPEC.md` are approved, except the `statusCode` topic | `DECIDED` |
| D21 | The jobs gateway client reads `statusCode` | `DECIDED` |
| D22 | The jobs API's own responses use `statusCode` | `DECIDED` |
| D23 | Phase 1 is a thin slice: a health endpoint deployed on Vercel | `DECIDED` |
| D24 | Verify is your own command sequence | `DECIDED` |
| D25 | `release.yml` follows the one in `kafka-api-gateway-rust` | `DECIDED` |
| D26 | Development environment: Linux on an Oracle Cloud ARM server, zsh | `DECIDED` |
| D27 | The subscriber image is built natively for `linux/arm64` | `ASSUMPTION` (from your command output) |
| D28 | The 14 coding rules in `CODING-STANDARDS.md` section 15 are approved | `DECIDED` |
| D29 | `statusCode` rename is done in all five related repos | `DECIDED` (done by you, untested) |
| D30 | Allowed `APP_ENV` values are `development`, `preview` and `production` | `DECIDED` (preview added by you) |
| D31 | Log format and filter follow `kafka-api-gateway-rust` | `DECIDED` |
| D32 | P1 crate set follows the references; `tower-http` stays on 0.6 | `DECIDED` |
| D33 | P1 scope trims: layout, validation 400, `db` health part and `Cargo.lock` | `DECIDED` |
| D34 | `cargo preview` loads `.env.preview` | `DECIDED` |
| D35 | Client secrets and tokens are compared in constant time | `DECIDED` |
| D36 | P2 includes JWT verification; token signing stays in P3 | `DECIDED` |
| D37 | `wants_html` is added in P2 in a helper file outside `pages/` | `DECIDED` |
| D38 | Expired tokens return `UNAUTHORIZED` / `EXPIRED_TOKEN` (401) | `DECIDED` |
| D39 | Constant-time comparison uses the `subtle` crate | `DECIDED` |
| D40 | P2 crates: `jsonwebtoken`, `utoipa`, `utoipa-scalar`; `serde_json` and the `tower-http` `cors` feature | `DECIDED` |
| D41 | Login rejects users with `isActive = 0` | `DECIDED` |
| D42 | Portal and API login give one 401 for unknown, inactive and wrong password, and always run one bcrypt check | `DECIDED` |
| D43 | `GET /api/v1/health` includes `db` and returns 503 while the database is down; `SERVICE_UNAVAILABLE` is the eighth error code | `DECIDED` |
| D44 | Pages show the product name "Gmail data gateway API" | `DECIDED` |
| D45 | P3 crates: `tiberius`, `bb8`, `bb8-tiberius`, `bcrypt`, `dioxus`, `dioxus-ssr` | `DECIDED` |
| D46 | P4: `reqwest` crate, a gateway client that takes its target as an argument, and the gateway reply `data` shape | `DECIDED` |

---

## D1: Hosting: API on Vercel, subscriber on-prem Docker

- **Status:** `DECIDED` (2026-11-06)
- **Context:** The new API needs a runtime target. The Kafka gateway runs on-prem only; recordlog splits API and subscriber.
- **Evidence (`OBSERVED`):**
  - `vercel.json` in `jobs-api-gateway-rust` is set to `framework: rust`.
  - `docker/docker-compose.yml` has the API service commented out and only `jobs-subscriber` active.
  - `recordlog-api-rust` uses the same split.
- **Alternatives:** API on-prem Docker like `kafka-api-gateway` (not chosen).
- **Decision:** The API runs on Vercel. The subscriber runs on-prem in Docker. Same as recordlog.
- **Rationale:** Matches the reference pair and the files you kept.
- **Consequences:**
  - The API crate needs the Vercel runtime entry like recordlog (`vercel_runtime` with `VercelLayer`).
  - The subscriber cannot run on Vercel because it keeps a long-lived WebSocket.
  - `public/global.css` must be committed because Vercel does not compile Tailwind.
- **Affects:** SPEC, PLAN, BUILD.

## D2: `extractMails` stays synchronous

- **Status:** `DECIDED` (2026-11-06)
- **Context:** `POST /api/v1/admin/extractMails` runs the whole Gmail extraction inside the HTTP request. The `Bulk` and `Sync` modes page through every matching mail (100 per page, 300 ms pause between pages, chunks of 20 messages with a 200 ms pause) with no upper limit. A large mailbox may run into the Vercel function time limit (`ASSUMPTION`, limit value not verified).
- **Alternatives:**
  - A: keep it synchronous and unlimited like the old code (chosen).
  - B: synchronous with a hard cap on mails per request (not chosen).
  - C: return 202 at once and extract in the on-prem subscriber (not chosen).
- **Decision:** Option A. Behavior stays as in the old code.
- **Rationale:** Your choice. It keeps the request and response flow identical to the old gateway.
- **Consequences:**
  - The Vercel time limit remains a known risk for very large `Bulk` runs. This is accepted.
  - Open risk to verify in the build phase: the real Vercel limit for the Rust runtime on your plan.
- **Affects:** SPEC, FEATURES, PLAN.

## D3: `extractMails` publishes in chunks of 100 records

- **Status:** `DECIDED` (2026-11-06). Publish order is decided in D10 and failure handling in D11.
- **Context:** The old code sends all records as one Kafka event with one `correlationId`. You asked for an improvement: send the records to the Kafka gateway in chunks over HTTP.
- **Alternatives:**
  - A: fetch all mails first, split into chunks, publish each chunk as its own event (chosen).
  - B: publish each Gmail page as soon as it is fetched (not chosen). Risk: `Sync` resumes from the last date stored in Azure, so a request cut off midway could leave older mails permanently skipped.
- **Decision:** Option A. Chunk size is 100 records. Each chunk is its own event with its own UUIDv7 `correlationId`. The response returns the list of `correlationId`s.
- **Rationale:** Keeps the old fetch-then-publish behavior and avoids the skip risk of option B. 100 matches the Gmail page size and stays far below the 10 MB gateway message limit.
- **Consequences:**
  - This is a deviation from the old response, which returned one `correlationId` and one `recordCount`. The new response shape is not designed yet (open item).
  - One request creates several pipeline runs. Status tracking in PostgreSQL stays per `correlationId`.
  - The Azure write procedure is idempotent per `correlationId` and skips `emailId`s that already exist, so re-running is safe.
- **Affects:** SPEC, FEATURES, PLAN.

## D4: Gmail access through direct REST calls with `reqwest`

- **Status:** `DECIDED` (2026-11-06)
- **Context:** The old code uses the `googleapis` npm package for `messages.list`, `messages.get` (metadata format) and the refresh-token exchange. Rust has no official equivalent.
- **Evidence (`FACT`, docs.rs):** `google-gmail1` exists at version 7.0.0+20251215, and its examples build on `hyper_rustls`, `hyper_util` and `yup_oauth2`. That would add a second HTTP and TLS stack next to the `reqwest` recordlog already uses.
- **Alternatives:**
  - A: the `google-gmail1` crate (not chosen).
  - B: direct REST calls with `reqwest` and our own refresh-token call (chosen).
- **Decision:** Option B.
- **Rationale:** Fewer dependencies and a smaller Vercel build. The code needed is small (list, get, token refresh).
- **Consequences:**
  - We own the OAuth refresh code and the response parsing.
  - `UNKNOWN` until the build phase: the exact endpoints, parameters and error codes must be checked against Google's official documentation before any code is written. Nothing here is verified yet.
- **Affects:** SPEC, FEATURES, BUILD, CODING-STANDARDS.

## D5: `generate-token` ported as an `xtask` command

- **Status:** `DECIDED` (2026-11-06)
- **Context:** `generate-token.ts` prints a Google consent URL, asks for the pasted code and saves the new refresh token to Azure through `[02sjobsData].[04spupsertGoogleOAuthToken]`. Scopes in the old script: `gmail.readonly` and `drive`.
- **Decision:** Port it as an `xtask` command (for example `cargo gen-token`).
- **Rationale:** Without it nothing renews the stored Gmail refresh token.
- **Consequences:**
  - `xtask` needs an HTTP client and Azure access for this command.
  - The requested scopes are decided in D14.
- **Affects:** SPEC, FEATURES, BUILD.

## D6: Seven old dev scripts are dropped

- **Status:** `DECIDED` (2026-11-06)
- **Context:** The old repo has dev scripts that are not part of the product.
- **Decision:** Drop `count-bulk`, `count-sync`, `db-select-test`, `http-publish-test`, `run-ingest`, `get-token-expiry` and `getAccessToken`.
- **Rationale:** `getAccessToken` is unused. `get-token-expiry` has a real JWT hardcoded in the file and must not be copied. The others are covered by Rust tests and the health endpoints.
- **Consequences:** No Rust equivalents are built.
- **Affects:** SPEC, PLAN.

## D7: Shared Azure database and user tables, bcrypt-only passwords

- **Status:** `DECIDED` (2026-11-06)
- **Context:** Jobs portal login and API login query `01tportalUsers` and `02tapiUsers` by unqualified name, exactly like recordlog. The old TS code verified passwords with `Bun.password.verify`, which also accepts argon2id. Recordlog Rust verifies with `bcrypt` only.
- **Evidence:** You confirmed jobs uses the same Azure database and the same user rows as recordlog, and that the recordlog Rust login already works against them (`DECIDED` by your statement). The user table scripts are not in the `02sjobsData` tarball.
- **Decision:** Reuse the recordlog approach: same tables, `bcrypt` verification only.
- **Consequences:** An argon2id hash would fail to log in. If hashes are ever created through the old TypeScript code again, re-check them.
- **Affects:** SPEC, FEATURES.

## D8: `sqlx` 0.9 for the subscriber

- **Status:** `DECIDED` (2026-11-06), against the recommendation of 0.8.
- **Context:** The jobs subscriber is a near copy of the recordlog subscriber, which uses `sqlx` 0.8. The kafka gateway uses `sqlx` 0.9.
- **Alternatives:** 0.8 (recommended, not chosen).
- **Decision:** `sqlx` 0.9.
- **Rationale:** Your choice, consistent with the kafka gateway.
- **Consequences:**
  - Risk: the recordlog subscriber code was written for 0.8. Any API difference only shows up when you run `cargo check`, because the toolchain cannot be run here.
  - Build phases that touch the subscriber's PostgreSQL code must compare against the kafka gateway's 0.9 usage first.
- **Affects:** BUILD, PLAN, CODING-STANDARDS.

## D9: Logging with `tracing`

- **Status:** `DECIDED` (2026-11-06). Format details are open (see Open items).
- **Context:** Recordlog prints plain `println!` lines in the old IST format. The kafka gateway uses `tracing` and `tracing-subscriber`.
- **Alternatives:** Plain `println!` like recordlog (recommended, not chosen).
- **Decision:** Use `tracing`.
- **Consequences:**
  - Adds `tracing` and `tracing-subscriber` (env-filter) to the crates that log.
  - The recordlog subscriber's `println!` lines must be converted when copied.
- **Affects:** SPEC, BUILD, CODING-STANDARDS.

## D10: Chunks are published oldest first

- **Status:** `DECIDED` (2026-11-06)
- **Context:** `Sync` mode resumes from the last date stored in Azure. As far as I know Gmail lists mails newest first (`ASSUMPTION`, to verify against Google's documentation in the build phase). If the newest chunk were published first and a later chunk failed, the next `Sync` would start after the newest stored date and skip the older, unpublished mails for good.
- **Alternatives:** Newest chunk first, in Gmail order (recommendation was oldest first; newest first not chosen).
- **Decision:** Publish chunks oldest first. The fetched list is reversed before it is split into chunks of 100 (see D3).
- **Rationale:** After a partial failure the stored data is a contiguous older part, so the next `Sync` fetches only the remainder.
- **Consequences:**
  - Residual risk (`ASSUMPTION`): if the Kafka topic has several partitions, a newer chunk could be processed before an older one. If the older chunk then fails, the newer date is stored and the older mails could still be skipped. Whether the topic has one or several partitions is `UNKNOWN`.
  - The order inside one chunk does not matter, because a chunk is written to Azure as one unit.
- **Affects:** SPEC, FEATURES, PLAN.

## D11: A failed chunk stops the request

- **Status:** `DECIDED` (2026-11-06)
- **Context:** With oldest-first publishing (D10), chunk 3 of 5 may fail after chunks 1 and 2 were already published. Re-running is safe because the Azure write procedure skips `emailId`s that already exist.
- **Alternatives:** B: try the remaining chunks anyway and report which failed (recommendation was A; B not chosen).
- **Decision:** Stop at the first failed chunk. Return the standard error response (`success: false`, `statusCode`, `code`, `message`). The message states how many chunks were already published.
- **Rationale:** Continuing after a failure could store newer mails while an older chunk is missing, which is the skip risk D10 avoids.
- **Consequences:**
  - Already published chunks are not rolled back. They continue through the pipeline.
  - The exact error `code` and `statusCode` for this case are not chosen yet (`UNKNOWN`, to define in SPEC following the recordlog error code pattern).
- **Affects:** SPEC, FEATURES, PLAN.

## D12: `extractMails` returns a `correlationIds` list

- **Status:** `DECIDED` (2026-11-06)
- **Context:** The old success response carries one `correlationId` and one `recordCount`. With chunking (D3) there are several `correlationId`s.
- **Alternatives:** B: keep `correlationId` as the oldest chunk's id and add `correlationIds` and `chunkCount` beside it (recommendation was A; B not chosen).
- **Decision:** Replace `correlationId` with a `correlationIds` list in publish order (oldest chunk first). Keep `recordCount` as the total across all chunks. Add `chunkCount`.
- **Rationale:** Clean contract with no half-meaningful legacy field.
- **Consequences:**
  - This is a breaking change to the response of `extractMails`. Any client that reads `correlationId` must change. Whether such a client exists is `UNKNOWN`.
  - Exact field types and the `NO_NEW_RECORDS` response shape are defined in SPEC.
- **Affects:** SPEC, FEATURES.

## D13: The environment variable is `APP_ENV`

- **Status:** `DECIDED` (2026-11-06)
- **Context:** In `jobs-api-gateway-rust`, `.env.example` uses `APP_ENV`, while `.env.production` and `.env.development` use `NODE_ENV`. `kafka-api-gateway-rust` reads `APP_ENV` in code. `recordlog-api-rust` does not read either name in Rust.
- **Alternatives:** `NODE_ENV` (not chosen; a leftover from the Node setup).
- **Decision:** The Rust code reads `APP_ENV`. `.env.production` and `.env.development` are renamed from `NODE_ENV` to `APP_ENV` (values unchanged).
- **Consequences:** The env files are changed in the first build phase that touches configuration. Allowed values for `APP_ENV` are defined in SPEC.
- **Affects:** SPEC, BUILD.

## D14: `generate-token` requests the `gmail.readonly` and `drive` scopes

- **Status:** `DECIDED` (2026-11-06), against the recommendation of `gmail.readonly` only.
- **Context:** The old `generate-token.ts` requests `gmail.readonly` and `drive`. The only mention of Drive in the whole old source is that scope line; the ingest code never calls Drive (`OBSERVED`).
- **Alternatives:** `gmail.readonly` only (recommended, not chosen).
- **Decision:** The ported `xtask` command requests both scopes, as the old script did.
- **Rationale:** Your choice. It keeps the token compatible with the old setup.
- **Consequences:**
  - The stored refresh token grants access to Google Drive as well as Gmail. It must be handled as a high-sensitivity secret.
  - No Rust code uses Drive. The scope is only requested, not used.
- **Affects:** SPEC, BUILD.

## D15: Code phases try to install the Rust toolchain and run `cargo check` and `cargo test`

- **Status:** `DECIDED` (2026-11-06)
- **Context:** Your earlier saved preference said not to install the Rust toolchain and that you run `cargo fmt`, `check`, `test` and `build`. This project's first message said to install it if possible.
- **Alternatives:** B: never install; you run all cargo commands and send errors (not chosen).
- **Decision:** In code-writing chats, Claude tries to install the Rust toolchain and runs `cargo check` and `cargo test` before delivering each phase. You still run the final verify before committing.
- **Rationale:** Catches compile errors before delivery, which matters because D8 (`sqlx` 0.9) and D4 (hand-written Gmail client) carry compile risk.
- **Consequences:**
  - `UNKNOWN` until tried: whether the toolchain can be installed in Claude's environment. Its network allowlist includes `crates.io`, `index.crates.io`, `static.crates.io`, `conda.anaconda.org` and `archive.ubuntu.com`, but not the official Rust installer hosts (`OBSERVED` from the environment configuration).
  - If the install fails, the phase says so explicitly and nothing is described as compiled or tested. Claude then sends the tarball unverified, you run the cargo commands, and errors come back to Claude.
  - This replaces the old saved preference for this project only.
- **Affects:** BUILD, PLAN, CODING-STANDARDS.

## D16: Each phase is delivered as one full-repo tarball

- **Status:** `DECIDED` (2026-11-06)
- **Context:** Your first message asked for a tarball excluding `target` and `.git` but including env files. An older saved preference said to send a zip of new files plus `changes.md` when four or more files are new, and to include `.git`.
- **Alternatives:** B: a zip of only the new and changed files plus `changes.md` listing every modified existing file (recommendation was A; B not chosen).
- **Decision:** After each successful phase Claude delivers one full-repo tarball. It excludes `target` and `.git` and includes all `.env*` files.
- **Rationale:** The whole repo drops in at once, so nothing depends on remembering which files changed.
- **Consequences:**
  - The tarball contains real env files (`.env*`). Treat it as sensitive and do not share it.
  - Because `.git` is excluded, extract it over your working copy so your own `.git` stays untouched.
  - This replaces the older saved preferences about zip delivery and about including `.git`, for this project only.
- **Affects:** BUILD, PLAN.

## D17: Phases are cut by feature and pattern, with no cap on changes

- **Status:** `DECIDED` (2026-11-06)
- **Context:** Phase size was asked as a number of changes (under 20, 20 to 30, over 30). You said the change counts in your commit rule are descriptions of commit size, not limits, and asked not to hard-code them.
- **Alternatives:** Capping each phase at under 20 changes (recommended, not chosen).
- **Decision:** Each phase contains whatever its feature or functionality needs, following the project patterns and `CODING-STANDARDS.md`. No fixed limit on the number of files or changes per phase.
- **Consequences:**
  - `PLAN.md` cuts phases by feature and dependency order, not by size.
  - The commit message heading tiers (4, 8 or 12 headings by actual change count) still apply to the commit text. This is my reading of your rule (`ASSUMPTION`). Tell me if the tiers should go away too.
  - A large phase carries more uncompiled risk if D15's toolchain install does not work. `PLAN.md` flags such phases.
- **Affects:** PLAN, BUILD.

## D18: Database scripts stay unchanged unless an error shows a change is needed

- **Status:** `DECIDED` (2026-11-06)
- **Context:** With chunking (D3), each chunk gets its own `correlationId`. From reading the scripts, the Azure write procedure and the PostgreSQL status and read-model procedures already work per `correlationId` and skip `emailId`s that already exist. Nothing was run (`UNKNOWN` whether any hidden problem exists).
- **Alternatives:** Changing scripts up front (not chosen; you cannot judge what to change before something fails).
- **Decision:** Keep `jobs_data_db` and `02sjobsData` unchanged in this rewrite. If a real error appears during a phase, the cause is reported and a change is decided then.
- **Consequences:**
  - No database migration work is planned in `PLAN.md`.
  - Any database change that comes later is added to this file as its own decision before it is made.
- **Affects:** SPEC, PLAN.

## D19: Only error catalog entries the Rust code uses are ported

- **Status:** `DECIDED` (2026-11-06)
- **Context:** The old error catalog has about 70 entries across many HTTP codes, and most are never used by the jobs code.
- **Alternatives:** B: port the whole old catalog (not chosen).
- **Decision:** The Rust catalog contains only entries the code actually uses (listed in `SPEC.md` section 11). New entries are added when a feature needs them.
- **Consequences:** Less code to maintain and test. A later feature that needs a new code must add its entry in the same phase.
- **Affects:** SPEC, FEATURES, CODING-STANDARDS.

## D20: The `PROPOSED` items in `SPEC.md` are approved, except the `statusCode` topic

- **Status:** `DECIDED` (2026-11-06)
- **Context:** `SPEC.md` marks several items `PROPOSED` (workspace layout, access gate, `KAFKA_GATEWAY_FAILED`, environment variable usage, startup validation and others). You answered "rest all i agree" while asking for `statusCode` to be used.
- **Decision:** All `PROPOSED` items in `SPEC.md` are approved as written. The one exception was the field name `statusCode` versus `httpCode`, settled in D21.
- **Consequences:** `SPEC.md` keeps the `PROPOSED` tag, and its legend now says those items are approved by D20.
- **Affects:** SPEC.

## D21: The jobs gateway client reads `statusCode`

- **Status:** `DECIDED` (2026-11-06)
- **Context:** `recordlog-api-rust` reads `statusCode` from the Kafka gateway reply. The current `kafka-api-gateway-rust` sends `httpCode`. You asked for `statusCode` in the jobs Rust code.
- **Alternatives:** A: rename the jobs API's own response field to `statusCode` and keep reading `httpCode` from the gateway (recommended, not chosen). C: both A and B (not chosen).
- **Decision:** Option B. The jobs gateway client reads `statusCode` from the gateway reply and requires it to equal 202. You will change `recordlog-api-rust` and `kafka-api-gateway-rust` to use `statusCode` only (your TODO).
- **Rationale:** Your choice. One field name across the gateway and its clients.
- **Consequences:**
  - Your rename in the gateway is done (see D29). A live publish check still waits for P11 because nothing has been tested yet. Tests with a fake gateway are not affected.
  - The jobs API's own response field is settled separately in D22.
- **Affects:** SPEC, PLAN, BUILD.

## D22: The jobs API's own responses use `statusCode`

- **Status:** `DECIDED` (2026-11-06)
- **Context:** The old jobs API and `recordlog-api-rust` use `httpCode` in their success and error responses. You want `statusCode` as the field name and said you will align any other project, Rust or TypeScript, that needs it.
- **Alternatives:** Keep `httpCode` in the jobs API's own responses (not chosen).
- **Decision:** The success and error envelopes of the jobs Rust API use `statusCode`. Together with D21, `statusCode` is the only name used in the jobs Rust code, for its own responses and for the gateway reply.
- **Rationale:** Your choice. One field name across all your services.
- **Consequences:**
  - This changes the public jobs response for any client that reads `httpCode`. You aligned the other repos yourself (D29).
  - Docs, tests and the OpenAPI schema in the jobs repo use `statusCode`.
- **Affects:** SPEC, FEATURES, PLAN.

## D23: Phase 1 is a thin slice: a health endpoint deployed on Vercel

- **Status:** `DECIDED` (2026-11-06)
- **Context:** The Vercel Rust runtime is the least-proven part of the plan, and D2 already carries a time-limit risk.
- **Alternatives:** A: build all foundation features first, then API, subscriber and deployment (recommendation was B; A not chosen).
- **Decision:** Phase 1 builds only enough to deploy a working health endpoint on Vercel: workspace, configuration, response envelope, error handling, static assets and health. The foundation, API and subscriber phases follow in the order set in `PLAN.md`.
- **Rationale:** Proves that the build and the Vercel deployment work before larger features depend on them.
- **Consequences:**
  - Phase 1 includes F52 (Vercel deployment) in a minimal form.
  - Some foundation features (middleware, Azure access) are built after the first deployment, so phase 1 uses only the parts it needs.
- **Affects:** PLAN, BUILD.

## D24: Verify is your own command sequence

- **Status:** `DECIDED` (2026-11-06)
- **Context:** No verify command exists in the repos or in `.agents`. I proposed the standard Rust set; you answered A and gave your own notes, which are the real sequence.
- **Alternatives:** The plain set `cargo fmt --all -- --check`, clippy, test, build (not used as written).
- **Decision:** Verify is this sequence, in order (your notes):

'''+verify_block+'''

  Meaning: format (this rewrites files), check, clippy with warnings as errors and the lockfile enforced, test with the filtered output, a production build, then a start of the app for a manual look.
- **Consequences:**
  - `cargo build-prod` and `cargo start` (and `cargo dev`, `cargo fmt`) are cargo aliases defined in `.cargo/config.toml` and backed by `xtask` (`FACT`: recordlog defines `css`, `dev`, `start`, `build-prod`). The core aliases must exist in phase 1, so `PLAN.md` moves them from P9 to P1.
  - `clippy --locked` requires `Cargo.lock` to be committed.
  - `cargo fmt` changes files, so a verify run can leave edits that belong in the same commit.
  - The output filters use `grep -P` and `grep -Ev`. Which shell you run them in is not stated and not assumed here.
  - When Claude runs checks itself (D15) it runs the commands without the filters, in its own environment.
  - The `SIGTERM` exclusion in the test filter suggests a test that starts and stops a server; the cause is `UNKNOWN`.
- **Affects:** PLAN, BUILD.

## D25: `release.yml` follows the one in `kafka-api-gateway-rust`

- **Status:** `DECIDED` (2026-11-06)
- **Context:** The kept `release.yml` in the jobs repo still sets up Bun, runs `bun install --frozen-lockfile` and ends with `bunx semantic-release`, but the Rust repo has no `package.json` or `bun.lock`. The version in `kafka-api-gateway-rust` instead uses `cycjimmy/semantic-release-action@v6.0.0` with `extra_plugins` (changelog, git, cargo) and no Bun (`FACT`: file comparison). The `.releaserc.json` files of the two repos are identical.
- **Alternatives:** Keep the recordlog-style file (also Bun; not chosen).
- **Decision:** Use the kafka-api-gateway-rust `release.yml` as the reference for jobs. Update 2026-11-06: you have already applied it to `recordlog-api-rust` (your commit replaced the Bun `release.yml` with the cycjimmy action and added `.releaserc.json`). Testing it in GitHub Actions containers is still pending.
- **Consequences:**
  - My earlier note that `release.yml` needed no change was wrong; it does need replacing.
  - Until your test passes, the file is untested (`UNVERIFIED`).
  - Phase P10 copies the file and keeps it in sync with whatever you settle on.
- **Affects:** PLAN, BUILD.

## D26: Development environment: Linux on an Oracle Cloud ARM server, zsh

- **Status:** `DECIDED` (2026-11-06)
- **Context:** `BUILD.md` needs to know where the `cargo` commands run.
- **Decision:** You run the commands on Linux, on an Oracle Cloud ARM server (aarch64), in `zsh` (bash underneath).
- **Consequences:**
  - The verify filters in D24 use GNU `grep -P`, which exists on Linux, so they run as written. I have not run them.
  - Builds on this server produce `aarch64` binaries. Vercel builds for its own platform (`UNKNOWN`, not checked).
  - The Docker image platform for the subscriber depends on where the subscriber container runs (open question).
  - Commands in `BUILD.md` are written for `zsh` and `bash` syntax.
- **Affects:** BUILD, PLAN.

## D27: The subscriber image is built natively for `linux/arm64`

- **Status:** `ASSUMPTION`, discerned from evidence at your request (2026-11-06). Tell me if it is wrong.
- **Context:** The subscriber runs on-prem in Docker (D1). The image platform must match the machine that runs the container.
- **Evidence (`OBSERVED`):**
  - Your `uname -m` printed `aarch64`.
  - `docker ps` on that host shows an `apache/kafka:4.1.0` container publishing port 9092, so the Kafka broker runs there.
  - `docker/docker-compose.yml` has no `platform` setting, uses `network_mode: bridge` and `extra_hosts: host.docker.internal:host-gateway`, which means the container talks to services on its own Docker host.
  - `docker images` shows no jobs or recordlog subscriber image on that host (nothing built there yet).
- **Decision:** Build the subscriber image on the aarch64 host with no platform override, so it is `linux/arm64`.
- **Consequences:**
  - If the subscriber ever runs on an x86_64 machine, the image must be rebuilt there or built with an explicit platform. This is not planned.
  - `docker/Dockerfile.subscriber` must use base images that exist for arm64 (checked in P10).
- **Affects:** BUILD, PLAN (P10).

## D28: The 14 coding rules in `CODING-STANDARDS.md` section 15 are approved

- **Status:** `DECIDED` (2026-11-06)
- **Context:** `CODING-STANDARDS.md` ended with 14 proposed rules.
- **Decision:** All 14 rules are approved as written.
- **Consequences:**
  - They apply from phase P1 on.
  - Other `PROPOSED` items in the body of `CODING-STANDARDS.md` that are not in the list, such as constant-time secret comparison, were not covered by D28 (the constant-time rule was approved later in D35).
- **Affects:** CODING-STANDARDS, PLAN.

## D29: The `statusCode` rename is done in all five related repos

- **Status:** `DECIDED` (2026-11-06). Done by you; reported, not tested and not re-read by me.
- **Context:** D21 and D22 chose `statusCode` and left the rename of the other repos to you.
- **Evidence (your report):** In each repo you ran `grep -rnE "httpCode|http_code"` over the source, docs and config files (excluding `target`, `.git` and `CHANGELOG.md`) and it returned nothing. You committed in each:
  - `recordlog-api-rust`: `refactor(api): rename httpCode to statusCode and add Rust release workflow`
  - `kafka-api-gateway-rust`: `refactor(api): rename httpCode to statusCode in all gateway responses` (includes the websocket auth error frame and the `xtask` ws test)
  - `recordlog-api` (TypeScript), `jobs-api-gateway` (TypeScript) and `kafka-api-gateway` (TypeScript): `refactor: rename httpCode to statusCode ...`
- **Decision:** `statusCode` is the single response field name across the jobs Rust API, the gateway and the sibling repos.
- **Consequences:**
  - The "old code" facts about `httpCode` in `SPEC.md` describe the code before this rename. The old TypeScript jobs gateway now uses `statusCode` as well.
  - A live publish from the jobs API to the gateway is first checked in P11. Until then only fake-gateway tests cover the reply check.
  - One name in your commit log shows `getstatusCode()` in `kafka-api-gateway` (TypeScript) as the renamed method; that is your repo, outside this project.
  - The `recordlog-api-rust` commit also replaced its Bun `release.yml` (see D25).
- **Affects:** SPEC, PLAN, FEATURES.

## D30: Allowed `APP_ENV` values are `development`, `preview` and `production`

- **Status:** `DECIDED` (2026-11-07). Closes `SPEC.md` section 14 item 5, first half. The first proposal had two values; you kept `APP_ENV=preview` in `.env.preview`, so `preview` is added.
- **Evidence (`OBSERVED`):** `kafka-api-gateway-rust/src/schemas/env.rs` accepts only these two lowercase values; `.env.development` and `.env.production` here already use them.
- **Decision:** `AppEnv` is an enum with these three values; any other value stops start-up with "APP_ENV must be development, preview or production".
- **Affects:** SPEC, `src/schemas/env.rs`.

## D31: Log format and filter follow `kafka-api-gateway-rust`

- **Status:** `DECIDED` (2026-11-07). Closes `SPEC.md` section 14 item 5, second half.
- **Evidence (`OBSERVED`):** `kafka-api-gateway-rust/src/logging.rs`.
- **Decision:** Each line is `<IST time>\t[Target:LEVEL]\t<message>`. The filter is `RUST_LOG` when set, otherwise `info`. The kafka-specific `kafkit_client=error` part is not copied. The `expect` on the IST offset is replaced by a fallback, because of the no-`expect` rule.
- **Affects:** `src/logging.rs`.

## D32: P1 crate set follows the references; `tower-http` stays on 0.6

- **Status:** `DECIDED` (2026-11-07).
- **Evidence:** The crates.io index on 2026-11-07 lists `tower-http` 0.7.1, `axum` 0.8.9, `tokio` 1.54.1, `vercel_runtime` 2.4.1, `validator` 0.21.0, `envy` 0.4.2, `tracing` 0.1.44, `tracing-subscriber` 0.3.23. Only `tower-http` is ahead of the version in both references.
- **Decision:** Same version lines as the references. `tower-http` stays on 0.6 because both references use it with `axum` 0.8 and 0.7 is not proven with them. `Cargo.lock` pins the exact versions. No crate outside the references is added.
- **Affects:** `Cargo.toml`.

## D33: P1 scope trims

- **Status:** `DECIDED` (2026-11-07).
- **Decision:**
  - F10: assets are served; the `Layout` component moves to P3 with the first page that uses it (no `dioxus` in P1).
  - F05: 404, 405 and panic 500 are done; the validation 400 moves to P2, where the validated-input extractor lives with the middleware.
  - F21: `GET /api/v1/health` returns `data.app` only; `data.db` is added in P3 (additive).
  - `Cargo.lock` is not in this tarball because it cannot be generated here; your first `cargo check` creates it and it is committed with P1.
- **Affects:** FEATURES, PLAN.

## D34: `cargo preview` loads `.env.preview`

- **Status:** `DECIDED` (2026-11-07), on your request.
- **Context:** Vercel builds non-main branches as preview deployments that use the Preview environment. `.env.preview` holds those values locally.
- **Decision:** `cargo preview` is an alias for `xtask preview`: it loads `.env.preview`, validates it, builds Tailwind CSS and runs the API in release mode. `xtask` prints the file it loaded for `dev`, `start` and `preview`, and `build-prod` prints that it validated `.env.production`. `APP_ENV` in `.env.preview` is `preview` (D30).
- **Affects:** BUILD, `.cargo/config.toml`, `xtask`.

## D35: Client secrets and tokens are compared in constant time

- **Status:** `DECIDED` (2026-11-07). Closes O12.
- **Context:** A normal comparison stops at the first wrong character, so response time can leak how much of a guess was right.
- **Decision:** Every comparison of a client secret or token uses a constant-time comparison helper, introduced in P2 with the gate and the API authentication code. Errors still do not leak length or content.
- **Affects:** `CODING-STANDARDS.md` section 13, SPEC section 4.2, PLAN (P2), `src/middleware/*`.

## D36: P2 includes JWT verification; token signing stays in P3

- **Status:** `DECIDED` (2026-11-07).
- **Context:** The `Auth` extractor (F06) calls `verify_token`, which does not exist in jobs yet. In `recordlog-api-rust` it lives in `src/features/v1/auth/jwt.rs` next to `sign_token`.
- **Decision:** P2 adds `verify_token` and the token claims type so the `Auth` extractor works. `sign_token` and the login handler come in P3 (API login). Issuer and claims follow `SPEC.md` section 4.3.
- **Affects:** PLAN (P2, P3), `src/features/v1/auth/jwt.rs`, `src/middleware/auth.rs`.

## D37: `wants_html` is added in P2 in a helper file outside `pages/`

- **Status:** `DECIDED` (2026-11-07).
- **Context:** The gate (SPEC section 4.2, step 2) needs `wants_html`, which checks whether the `Accept` header contains `text/html`. In `recordlog-api-rust` it lives in `src/pages/render.rs`; jobs has no `pages/` until P3.
- **Decision:** P2 adds `wants_html` in a small helper file outside `pages/`, so the gate and the P3 pages both use it.
- **Affects:** PLAN (P2, P3), `src/middleware/server_gate.rs`.

## D38: Expired tokens return `UNAUTHORIZED` / `EXPIRED_TOKEN` (401)

- **Status:** `DECIDED` (2026-11-07).
- **Context:** The old jobs code returned `INVALID_TOKEN` for every token that failed verification. `recordlog-api-rust` returns a separate `EXPIRED_TOKEN` for expired tokens (`OBSERVED`: `src/errors/catalog.rs`, `src/features/v1/auth/jwt.rs`).
- **Decision:** An expired token returns `EXPIRED_TOKEN` (401). Any other verification failure returns `INVALID_TOKEN` (401). `EXPIRED_TOKEN` is added to the jobs error catalog in P2.
- **Affects:** SPEC sections 4.3 and 13, `src/errors/catalog.rs`, `src/features/v1/auth/jwt.rs`.

## D39: Constant-time comparison uses the `subtle` crate

- **Status:** `DECIDED` (2026-11-07). Implements D35.
- **Context:** Neither `recordlog-api-rust` nor `kafka-api-gateway-rust` has constant-time comparison code or the `subtle` crate (`OBSERVED`). D32 says a crate outside the references needs your approval.
- **Decision:** P2 adds the `subtle` crate and compares secrets and tokens with it.
- **Affects:** `Cargo.toml`, `src/middleware/*`, `CODING-STANDARDS.md` section 13.

## D40: P2 crates and features

- **Status:** `DECIDED` (2026-11-07), under D32 (same version lines as the references). This is the `DECISIONS.md` line that `CODING-STANDARDS.md` section 15, rule 13 asks for.
- **Evidence (`OBSERVED`, `recordlog-api-rust/Cargo.toml`):** `jsonwebtoken` 11 with `rust_crypto`, `utoipa` 6, `utoipa-scalar` 0.4 with `axum`, `serde_json` 1 as a normal dependency, `tower-http` 0.6 with `catch-panic` and `cors`.
- **Decision:** P2 adds exactly these. `serde_json` moves from dev-dependencies to dependencies because the `utoipa` path examples use it. `subtle` 2 comes from D39.
- **Affects:** `Cargo.toml`.

## D41: Login rejects users with `isActive = 0`

- **Status:** `DECIDED` (2026-11-07).
- **Context:** `01tportalUsers` and `02tapiUsers` both have an `isActive` bit column (`FACT`, your query result). The recordlog queries ignore it.
- **Decision:** The portal and API user lookups add `AND isActive = 1`, so an inactive user is treated like an unknown user.
- **Affects:** `src/db/api_users.rs`, `src/db/portal_users.rs`.

## D42: One 401 for every failed login, and one bcrypt check every time

- **Status:** `DECIDED` (2026-11-07).
- **Context:** `recordlog-api-rust` returns `USER_NOT_FOUND` for an unknown API user and `INVALID_CREDENTIALS` for a wrong password, and skips bcrypt for an unknown user. `FEATURES.md` F22 says wrong user and wrong password return the same response. An unknown user answered faster than a wrong password, so the response time showed which users exist.
- **Decision:** Portal login and API login return `UNAUTHORIZED` / `INVALID_CREDENTIALS` (401) for an unknown user, an inactive user and a wrong password. Every attempt runs one bcrypt check; a missing user is checked against a fixed cost-12 dummy hash and never matches. `USER_NOT_FOUND` is not ported. The cost 12 matches the existing hashes (`OBSERVED`: `kafka-api-gateway-rust/src/auth/password.rs`).
- **Affects:** `src/security.rs`, `src/features/v1/auth/service.rs`, `src/pages/login.rs`, SPEC section 11.

## D43: Health includes the database and returns 503 while it is down

- **Status:** `DECIDED` (2026-11-07). Amends D19.
- **Context:** The P1 response had only `data.app` and said the database part comes in P3. `recordlog-api-rust` returns `data.app` and `data.db` and answers 503 when the probe fails.
- **Decision:** `GET /api/v1/health` returns `data.app` and `data.db` and returns 503 `SERVICE_UNAVAILABLE` / `DB_PROBE_FAILED` while `SELECT 1` fails within 5 seconds. The pages `/healthz`, `/healthz/app` and `/healthz/db` answer 200 and show the database as DOWN. `SERVICE_UNAVAILABLE` (503) is added as the eighth error code.
- **Affects:** `src/features/v1/health/*`, `src/pages/healthz.rs`, `src/errors/code.rs`, `src/errors/catalog.rs`, SPEC sections 4.1 and 11.

## D44: Product name on the pages

- **Status:** `DECIDED` (2026-11-07).
- **Decision:** The login, home, about and health pages show "Gmail data gateway API", the same name as the OpenAPI title. The login heading reads "Gateway Login". The home and about texts are written for jobs.
- **Affects:** `src/components/*`, `src/pages/*`.

## D45: P3 crates

- **Status:** `DECIDED` (2026-11-07), under D32 (same version lines as the references). This is the `DECISIONS.md` line that `CODING-STANDARDS.md` section 15, rule 13 asks for.
- **Evidence (`OBSERVED`, `recordlog-api-rust/Cargo.toml`):** `tiberius` 0.12 (`tds73`, `rustls`), `bb8` 0.9, `bb8-tiberius` 0.16 (`with-tokio`, `tds73`, `tls`), `bcrypt` 0.19, `dioxus` 0.7 (`macro`, `html`, `hooks`, `signals`), `dioxus-ssr` 0.7.
- **Affects:** `Cargo.toml`.

## D46: P4 gateway client

- **Status:** `DECIDED` (2026-11-07), under D32 for the crate.
- **Evidence (`OBSERVED`):** `recordlog-api-rust/Cargo.toml` has `reqwest` 0.12 with `json` and `rustls-tls`. `kafka-api-gateway-rust` returns `statusCode` 202 and `data` of `{ topic, correlationId }` (`SuccessResponse::accepted`, `HttpPublishData`), so the D29 rename is in that code and the `data` shape is settled.
- **Decision:** P4 adds `reqwest` as in the reference. `post_to_gateway` reads the env and calls `post_to_gateway_at`, which takes the gateway url and the four credentials as a `GatewayTarget`. Tests start a fake gateway on a free local port and call `post_to_gateway_at`, so they need no env variables and no change to the shared test env. `KAFKA_GATEWAY_FAILED` (500) is added to the error catalog.
- **Affects:** `Cargo.toml`, `src/clients/kafka_gateway.rs`, `src/errors/catalog.rs`, `tests/gateway.rs`, SPEC section 7.

---



## Open items (not decisions)

| ID | Item | Class |
| --- | --- | --- |
| O1 | Chunk publish order. | Resolved by D10 |
| O2 | Failure of one chunk after earlier chunks succeeded. | Resolved by D11 |
| O3 | New `extractMails` response shape for several `correlationId`s. | Resolved by D12 |
| O4 | `CONFLICT`: `recordlog-api-rust` expects `statusCode` in the gateway reply, but `kafka-api-gateway-rust` returned `httpCode`. | Resolved by D21 and D29 (rename done by you) |
| O11 | Field name in the jobs API's own responses. | Resolved by D22 |
| O12 | Whether client secrets and tokens are compared in constant time (`CODING-STANDARDS.md` section 13, not part of D28). | Resolved by D35 |
| O5 | `OBSERVED`: the kafka gateway `http-publish` request struct has only `topic`, `correlationId` and `value`, with no `deny_unknown_fields`, and uses `correlationId` as the Kafka message key. The old jobs schema allowed optional `key` and `headers`. The Rust jobs client sends only the three fields. | Resolved by evidence, to confirm in SPEC |
| O6 | Environment variable name (`APP_ENV` vs `NODE_ENV`). | Resolved by D13 |
| O7 | `OBSERVED`: `.agents/state/checkpoint.md` holds sample content (an `acme-widgets` project), not real project state. | Acceptable, to replace on first save |
| O8 | Whether the `drive` OAuth scope is still needed. | Resolved by D14 |
| O9 | Rust toolchain use. | Resolved by D15 |
| O10 | Error catalog scope. | Resolved by D19 |