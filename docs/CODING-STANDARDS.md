# CODING-STANDARDS.md: jobs-api-gateway-rust

Owns **how code is written** in this repo: structure, naming, errors, comments, dependencies, testing and what counts as done. What to build is in `SPEC.md` and `FEATURES.md`; reasons are in `DECISIONS.md`.

Last updated: 2026-11-06

## How to read this file

| Tag | Meaning |
| --- | --- |
| `FACT` | Observed in `recordlog-api-rust` or `kafka-api-gateway-rust` (files read, not run). |
| `DECIDED` | You decided it. The D-number or your standing rule is named. |
| `PROPOSED` | My proposal. Not approved, except the 14 numbered rules in section 15, which you approved on 2026-11-06 (D28). |

A rule tagged `FACT` is the default pattern: copy it unless a `DECIDED` rule says otherwise. Nothing here has been compiled.

---

## 1. Principles

1. Follow the reference repos before inventing a pattern (`FACT`: your instruction to model the rewrite on recordlog and kafka).
2. Match the behavior in `SPEC.md`, not the structure of the old TypeScript code.
3. Fail loudly at start-up, never silently at run time.
4. Keep each file about one thing (`FACT`: handler, service and model are separate files).
5. No unproven claims: a comment, doc or commit never says something compiled or passed unless it did.

## 2. Workspace and crates

- Root crate `jobs-api-gateway` is a library plus a binary named `main` at `src/main.rs`; members are `xtask` and later `subscriber` (`FACT`: recordlog layout; crate name is `PROPOSED`).
- The subscriber depends on the root crate by path and reuses its config, env schema, clients and database code (`FACT`).
- Edition 2024 (`FACT`), `publish = false` (`FACT`).
- No `rustfmt.toml`, `clippy.toml` or `rust-toolchain.toml` (`FACT`: none in the references). Default `rustfmt` rules apply.
- `Cargo.lock` is committed because verify uses `clippy --locked` (`DECIDED`, D24).
- Lints: `cargo clippy --workspace --all-targets --locked -- -D warnings` must pass with no `allow` added to hide a warning (`DECIDED`, D24 for the command; the no-`allow` rule is `PROPOSED`).

## 3. Module layout

`FACT` from `recordlog-api-rust/src/lib.rs`: `app`, `clients`, `components`, `config`, `db`, `errors`, `features`, `middleware`, `openapi`, `pages`, `schemas`.

| Module | Holds |
| --- | --- |
| `app` | Builds the router and wires middleware |
| `clients` | Outbound HTTP (gateway, Gmail, Google OAuth) |
| `config` | The global env accessor |
| `db` | Azure SQL access and queries |
| `errors` | `ErrorCode`, catalog, `ApiError`, fallback and panic handling |
| `features/v1/<feature>/` | One folder per API feature |
| `middleware` | CORS, gate, authentication |
| `openapi` | OpenAPI document and error examples |
| `pages`, `components` | HTML pages and shared layout |
| `schemas` | Env schema, response envelope, shared request and response types |

Rules:
- A feature folder has `mod.rs`, `handler.rs`, `service.rs`, `model.rs` (`FACT`; `auth` also has `jwt.rs`).
- `handler.rs` exposes `router()` and the route functions and does only HTTP work: extract, validate the shape, call the service, build the response (`FACT`).
- `service.rs` holds the logic and returns `Result<_, ApiError>` (`FACT`).
- `model.rs` holds request, response and event types (`FACT`).
- Handlers never talk to a database or a client directly; they call a service (`FACT`).
- Jobs-specific code goes in the places listed in `SPEC.md` section 3: `clients/gmail.rs`, `clients/google_oauth.rs`, `db/google_oauth.rs`, `db/jobs.rs`, `features/v1/admin`.

## 4. Naming and style

- Rust standard naming: `snake_case` functions and modules, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants (`FACT`).
- JSON field names are camelCase through `#[serde(rename_all = "camelCase")]` on the type (`FACT`). The status field is `statusCode` (`DECIDED`, D22).
- Route paths and operation ids: path `/api/v1/admin/extractMails` as in the old API; operation id in `snake_case` such as `admin_extract_mails` (`FACT` pattern from `admin_post_logs`).
- One statement of intent per function name: `post_to_gateway`, `post_logs` style (`FACT`).
- Keep imports grouped: external crates first, then `super`, then `crate` (`FACT` pattern); `rustfmt` enforces the layout.

## 5. Comments

- **Every comment goes on its own line above the code it describes. No trailing comments after code.** (`DECIDED`: your standing rule.)
- Comments use plain `//` lines in short plain English (`FACT`: the references do this, for example "Builds the error from a catalogue entry").
- A comment says why or what a function is for. It does not repeat the code line.
- Where a rule comes from `SPEC.md` or a decision, the comment may name it (`PROPOSED`, example: `// Oldest chunk first, see DECISIONS D10`).
- No commented-out code is committed (`PROPOSED`).

## 6. Errors

- One error type for handlers: `ApiError` (`FACT`).
- Entries live in a catalog of `const` `CatalogEntry` values grouped by HTTP class in modules such as `unauthorized`, `bad_request`, `not_found`, `internal` (`FACT`).
- Create an error with `ApiError::from_entry(bad_request::INVALID_INPUT)` or by `?` on a `CatalogEntry` (`FACT`).
- `ApiError::new(code, text)` only when the text is dynamic, for example the Gmail status (`FACT`).
- The catalog holds only entries the code uses (`DECIDED`, D19). A new entry is added in the same change that needs it.
- Messages shown to clients never contain secrets, SQL text, tokens or mail content (`PROPOSED`).
- The error body is `success`, `statusCode`, `code`, `message`, `error` (`DECIDED`, D22).
- No `unwrap` and no `expect` in code that runs after start-up. In the recordlog source the only `expect` builds the HTTP client once (`FACT`); the same single exception is allowed (`PROPOSED`).
- Use `?` to propagate. Map a foreign error to `ApiError` at the boundary where it happens, once (`PROPOSED`).
- Panics inside handlers are caught and turned into the 500 envelope (`FACT`: `catch-panic` in `tower-http`).

## 7. Configuration

- Environment is read once into a global `Env` through `LazyLock` and `env()`; invalid or missing values stop the process at boot (`FACT`: `config/env.rs`).
- The schema is a serde struct loaded with `envy` and checked with `validator` rules matching `SPEC.md` section 10 (`FACT` approach).
- Variable names stay exactly as in `.env.example`, including `Google_APPID` (`DECIDED`, D20 approves the `SPEC.md` names).
- `APP_ENV` selects the environment (`DECIDED`, D13).
- Never print or log the full env or any secret field. Do not derive `Debug` on a struct that holds secrets without redacting them (`PROPOSED`).
- Tests set the variables once in a shared helper before the env is first read (`FACT`: `tests/common/mod.rs`).

## 8. HTTP, async and external calls

- `axum` 0.8 router, `tokio` runtime, `tower-http` for CORS and panic catching (`FACT`).
- One shared `reqwest` client per process, built once with `rustls` (`FACT`: recordlog builds it once). Request timeouts are set explicitly (`PROPOSED`, values `UNKNOWN`).
- Never block the async runtime: no blocking file or network calls inside `async` code (`PROPOSED`).
- Gmail pacing follows `SPEC.md` section 5.3: page size 100, 300 ms between pages, groups of 20 with 200 ms between groups.
- Outbound JSON replies are parsed into typed structs and checked (`statusCode` equals 202 for the gateway, `DECIDED`, D21). An unparsable reply is an error, never a guess.
- The access gate and authentication run in middleware or extractors, not inside feature code (`FACT`: `middleware/auth.rs` with an `Auth` extractor).

## 9. Database access

- Azure SQL: `tiberius` through `bb8` and `bb8-tiberius` with `rustls` (`FACT`). The API uses the public login and the subscriber the service login (`SPEC.md` section 10).
- PostgreSQL in the subscriber: `sqlx` 0.9 (`DECIDED`, D8). The recordlog subscriber is written for 0.8, so every copied query call is compared with the kafka gateway's 0.9 use first.
- Values go in as parameters, never built into SQL text (`FACT` pattern for procedure inputs; `PROPOSED` as a hard rule).
- A procedure name from an event must match the allowed pattern and length in `SPEC.md` section 9 before it is used (`FACT` in the old code, kept by D20).
- Passwords are checked with `bcrypt` only (`DECIDED`, D7). Never log a hash or a password.
- Database scripts stay unchanged unless an error shows a change is needed (`DECIDED`, D18).

## 10. Logging, time and data

- Logging uses `tracing` with timestamps in IST (`DECIDED`, D9). Format and level filter are `UNKNOWN` until P1. The kafka gateway is the model (`FACT`).
- Env errors at boot may be printed to stderr before logging starts (`FACT`: recordlog prints `[env] Validation failed`).
- Never log tokens, passwords, hashes, the refresh token, secrets, or mail text (`PROPOSED`).
- Times use `chrono` without default features (`FACT`). IST is the fixed offset +05:30 and dates are written `YYYY-MM-DDTHH:MM:SS+05:30` (`SPEC.md` section 5.3).
- Identifiers use `uuid` version 7 (`FACT`).
- Types state what they hold: `String` for text, `Uuid` internally and `String` on the wire only where the contract says so (`PROPOSED`).

## 11. Testing

- Integration tests live in `tests/` and use the real router from `app::router()` with test env values and `tower::ServiceExt` (`FACT`). The helper module is `tests/common/mod.rs` (`FACT`).
- Existing test files in recordlog: `auth`, `auth_extractor`, `cors`, `errors`, `gate`, `health`, `login`, `openapi`, `post_logs` (`FACT`). Jobs follows the same one-file-per-area habit.
- Recordlog tests do not start a local server: they point outbound URLs at an unreachable address and test the failure paths (`FACT`: no `TcpListener` in `tests/` or `src/`). Success paths of outbound calls therefore have no reference test.
- For success paths jobs adds small fake servers inside tests, built with `axum` on port 0 so no new dependency is needed (`PROPOSED`). Gmail, Google OAuth and the gateway get one fake each (matches `FEATURES.md` F11).
- Pure logic (text cleaning, chunking, query building, date formatting) gets unit tests next to the code in `#[cfg(test)]` modules (`PROPOSED`).
- Tests never use real credentials or real Azure, Gmail or Kafka (`PROPOSED`).
- A phase that adds behavior adds tests for it in the same phase, using the "Done when" lines in `FEATURES.md` as the checklist (`PROPOSED`).

## 12. Dependencies

Versions below are the ones in `recordlog-api-rust/Cargo.toml` (`FACT`). They are a starting point: current versions are checked before pinning in P1, because I have not verified that they are still the latest.

| Crate | Recordlog version and features | Used for |
| --- | --- | --- |
| `axum` | 0.8 | HTTP |
| `tokio` | 1, `full` | Runtime |
| `tower` | 0.5 (`util` in dev) | Service helpers, tests |
| `tower-http` | 0.6, `catch-panic`, `cors` | CORS, panic catching |
| `vercel_runtime` | 2.4, `axum` | Vercel entry (D1) |
| `dioxus` and `dioxus-ssr` | 0.7, minimal features | Server-rendered HTML pages |
| `dotenvy`, `envy` | 0.15, 0.4 | Env loading |
| `serde`, `serde_json` | 1 | JSON |
| `validator` | 0.21, `derive` | Env and input rules |
| `chrono` | 0.4, `clock` only | Time |
| `tiberius`, `bb8`, `bb8-tiberius` | 0.12, 0.9, 0.16, `rustls` | Azure SQL |
| `bcrypt` | 0.19 | Password check |
| `jsonwebtoken` | 11, `rust_crypto` | JWT |
| `reqwest` | 0.12, `json`, `rustls-tls`, no default features | Outbound HTTP |
| `uuid` | 1, `v7` | Correlation ids |
| `utoipa`, `utoipa-scalar` | 6, 0.4 | OpenAPI and its page |
| `http-body-util` | 0.1 (dev) | Test bodies |

Subscriber (`FACT`): `tokio-tungstenite` 0.28 (`connect`, `rustls-tls-webpki-roots`), `futures-util` 0.3, and `sqlx` (jobs uses 0.9, `DECIDED`, D8).

Jobs additions (`DECIDED` D9, rest `PROPOSED`): `tracing` and `tracing-subscriber` with the env filter.

Rules:
- Prefer `default-features = false` and list only the features used (`FACT` habit).
- A new crate needs a line in `DECISIONS.md` saying why (`PROPOSED`).
- Gmail and OAuth use `reqwest` and our own code, not a Google crate (`DECIDED`, D4).

## 13. Security

- Secrets live only in env files and in Azure; never in source, docs, tests, logs or chat. `.env.*` is ignored by git except `.env.example` (`FACT`).
- The delivered tarball contains real env files and must not be shared (`DECIDED`, D16).
- The stored Google refresh token also grants Drive access (`DECIDED`, D14), so handle it as high sensitivity.
- Compare client secrets and tokens without leaking length or content in errors. Whether to use a constant-time comparison is `UNKNOWN` and `PROPOSED` for the gate.

## 14. Definition of done for a change

A change is done when:
1. Behavior matches `SPEC.md`, or `SPEC.md` is updated in the same change.
2. Tests exist for the new behavior (section 11).
3. Your verify passes: format, check, clippy, test, then `cargo build-prod` (`DECIDED`, D24).
4. `FEATURES.md` and `DECISIONS.md` reflect what changed.
5. The commit message follows `BUILD.md` section 9.

## 15. Approved rules (D28)

You approved these 14 rules on 2026-11-06 (D28). They are tagged `PROPOSED` in the body above because that is where they were first written; this list is the approved set. Any `PROPOSED` item in the body that is not in this list is still not approved.

1. No `allow` attributes added to silence a clippy warning.
2. No commented-out code committed.
3. Comments may name the `SPEC.md` rule or decision they implement.
4. No secrets, SQL text or mail content in client-facing errors or in logs; no `Debug` derive on structs holding secrets without redaction.
5. No `unwrap` or `expect` after start-up, with the single exception of building the shared HTTP client.
6. Errors from other crates are mapped to `ApiError` once, at the boundary.
7. Explicit timeouts on every outbound HTTP call (values decided in the phase).
8. No blocking calls inside async code.
9. All SQL values passed as parameters, never built into SQL text.
10. Fake servers built with `axum` on port 0 for Gmail, Google OAuth and the gateway in tests; unit tests next to pure logic.
11. Tests never touch real credentials or real external services.
12. Each phase adds the tests for its behavior in the same phase.
13. A new crate needs a line in `DECISIONS.md`.
14. Root crate name `jobs-api-gateway`.