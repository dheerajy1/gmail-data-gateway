# SPEC.md: jobs-api-gateway-rust

Owns the **what**: the observable behavior and contracts of the rewritten system. The reasons live in `DECISIONS.md`; the order of work lives in `PLAN.md`.

Last updated: 2026-11-06

## How to read this file

| Tag | Meaning |
| --- | --- |
| `FACT` | Read directly from the old TypeScript code, the reference Rust repos or the database scripts. Not executed. |
| `DECIDED` | You selected it. See `DECISIONS.md` (D-numbers). |
| `PROPOSED` | My proposal that follows the reference pattern. Approved by you on 2026-11-06 (D20), unless a note says otherwise. |
| `UNVERIFIED` | Comes from my general knowledge or an earlier reading, not re-checked against a source. Verify before building. |
| `UNKNOWN` | Not determined. |

Nothing here has been compiled or run.

---

## 1. Purpose and scope

The system reads job-application mails from one Gmail account, publishes them to Kafka through the on-prem Kafka API gateway, and stores them in two databases:

1. **Azure SQL** (write model): table `[02sjobsData].[04trawAppliedJobs]`.
2. **PostgreSQL on-prem** (read model and pipeline status).

It has two deployables (`DECIDED`, D1):

| Component | Runs on | Role |
| --- | --- | --- |
| API (`jobs-api-gateway-rust`) | Vercel | HTTP API, docs pages, Gmail extraction, publish to the gateway. |
| Subscriber | On-prem, Docker | Long-lived WebSocket client of the gateway. Writes Azure, applies the read model, records status. |

Out of scope: any change to `jobs_data_db` or `02sjobsData` scripts (`DECIDED`, D18) and the seven dropped dev scripts (`DECIDED`, D6).

## 2. End-to-end flow

1. Admin calls `POST /api/v1/admin/extractMails`.
2. The API builds a Gmail query, pages through Gmail, reads message metadata, and normalizes each mail into a record (section 5).
3. The records are reversed to oldest first, split into chunks of 100, and each chunk is published as one event to the write-model topic over HTTP (`DECIDED`, D3, D10).
4. The subscriber consumes the write-model topic, calls the Azure procedure, then republishes the same event to the read-model topic.
5. The subscriber consumes the read-model topic and calls the PostgreSQL procedure.
6. At every stage the subscriber publishes a status event to the status topic. A third consumer applies these to PostgreSQL with `fn_pipeline_apply_status`.

## 3. Workspace layout (`PROPOSED`)

Mirrors `recordlog-api-rust` (`FACT`: it has a root crate with a `lib.rs`, a `subscriber` member and an `xtask` member).

```
jobs-api-gateway-rust/
  Cargo.toml                 workspace: root crate, subscriber, xtask
  .cargo/config.toml         cargo aliases
  src/
    main.rs, lib.rs, app.rs
    config/env.rs            env loading
    schemas/                 env, response envelope, auth
    errors/                  catalog, codes, api_error, fallback, panic
    middleware/              auth, no_auth, server_gate, cors
    clients/                 kafka_gateway, gmail, google_oauth
    db/                      azure, api_users, portal_users, google_oauth, jobs
    features/v1/             admin (extractMails), auth, health
    pages/ and components/   HTML pages (home, about, login, healthz)
    openapi/
  subscriber/src/            main, transport, event, handlers/, azure, postgres, retry, failure, status, time
  xtask/src/main.rs          docker helpers and gen-token
  tests/                     integration tests with fakes
  public/                    global.css, logo.png, favicon.ico (committed)
```

Jobs-specific additions over recordlog: `clients/gmail.rs`, `clients/google_oauth.rs`, `db/google_oauth.rs`, `db/jobs.rs`, and the `extractMails` feature. The subscriber is a near copy of the recordlog subscriber with jobs topics and the status handler (section 8).

## 4. HTTP surface

### 4.1 Routes

| Method and path | Auth | Purpose |
| --- | --- | --- |
| `GET /`, `/home`, `/about` | Gate | HTML pages (`FACT`: old routes `home`, `about`). |
| `GET /login`, `POST /login` | Public | Docs login page. On success sets cookie `auth=ok` (`FACT`). |
| `GET /healthz`, `/healthz/app`, `/healthz/db` | Gate | HTML health pages (`FACT`). |
| `GET /openapi` | Gate | API documentation. The old title is "Gmail data gateway API", version `v1` (`FACT`). |
| `GET /api/v1/health` | Gate | JSON health with `data.app` and `data.db`; returns 503 `SERVICE_UNAVAILABLE` while the database is down (`DECIDED`, D43; old response `UNKNOWN`). |
| `POST /api/v1/auth/login` | Client headers | API login, returns a JWT (`FACT`). |
| `POST /api/v1/admin/extractMails` | Client headers and JWT | Gmail extraction and publish (section 5). |
| `GET /global.css`, `/logo.png`, `/favicon.ico` | Public | Embedded assets (`FACT` in recordlog Rust). |

### 4.2 Access gate (`FACT` in old code, `PROPOSED` to mirror recordlog Rust)

A request passes when any of these is true, checked in this order:

1. The path ends in `css`, `js`, `png`, `jpg`, `jpeg`, `svg`, `ico` or `txt` (static assets).
2. The `Accept` header does not contain `text/html` and `x-client-id` and `x-client-secret` match `CLIENT_ID` and `CLIENT_SECRET`.
3. The path starts with `/login`.
4. The cookie `auth` equals `ok`.

Otherwise the response is a 302 to `/login`. Unknown paths return the global 404 error shape and wrong methods the 405 shape, without going through the gate (`FACT` in recordlog Rust).

### 4.3 API authentication (`FACT`)

| Header | Rule |
| --- | --- |
| `x-client-id` | Required, 1 to 30 characters, must equal `CLIENT_ID`. |
| `x-client-secret` | Required, 1 to 50 characters, must equal `CLIENT_SECRET`. |
| `authorization` | Required for protected routes, `Bearer <jwt>`. |

Outcomes in the old code:

| Case | Code |
| --- | --- |
| `x-client-id` or `x-client-secret` missing | `UNAUTHORIZED` / `MISSING_AUTH_HEADERS` (401) |
| Headers present but invalid or wrong | `BAD_REQUEST` / `INVALID_HEADER_VALUES` (400) |
| `authorization` not a Bearer value | `UNAUTHORIZED` / `MISSING_AUTHORIZATION_TOKEN` (401) |
| JWT is expired (`DECIDED`, D38) | `UNAUTHORIZED` / `EXPIRED_TOKEN` (401) |
| JWT does not verify for any other reason | `UNAUTHORIZED` / `INVALID_TOKEN` (401) |
| JWT lacks `sub` or `role` | `UNAUTHORIZED` / `MISSING_SUB_OR_ROLE` (401) |

JWT: HS256 is `UNVERIFIED` (the old plugin default); issuer `jobs-api-gateway`, expiry 1 day, claims `sub`, `role` (`admin` or `user`), `iat`, `exp`, optional `iss`. Secret from `JWT_SECRET`. Passwords: bcrypt only (`DECIDED`, D7). Users come from `01tportalUsers` (docs login) and `02tapiUsers` (API login) in Azure (`DECIDED`, D7).

### 4.4 CORS (`FACT`)

- Requests without an `Origin` header are allowed.
- Fixed origin: `http://localhost:3000`.
- Allowed by suffix: `.trycloudflare.com`, `.devtunnels.ms`, `.vercel.app`, `.dheerajy1dev.dpdns.org`.
- Credentials allowed. Allowed headers: `Content-Type`, `Authorization`, `X-Client-Id`, `X-Client-Secret`. Methods: `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `OPTIONS`.

### 4.5 Response envelope (`FACT`)

Success: `{ "success": true, "statusCode": <n>, "message": "<text>", "data": { ... } }`

Error: `{ "success": false, "statusCode": <n>, "code": "<CODE>", "message": "<text>", "error": <detail> }`

Validation errors return 400 with the validation failure in `error`. Anything unexpected returns 500 `INTERNAL_SERVER_ERROR`.

Field name (`DECIDED`, D22): the jobs API's own envelope uses `statusCode`, not `httpCode`. The old jobs code and `recordlog-api-rust` used `httpCode` until your rename on 2026-11-06; you have since renamed it in all five related repos (D29), so no client of those repos should still read `httpCode`.

## 5. `extractMails`

### 5.1 Request (`FACT`)

`POST /api/v1/admin/extractMails`, JSON body:

| Field | Type | Rule |
| --- | --- | --- |
| `type` | `Bulk`, `Increment` or `Sync` | Required. |
| `query` | string | Required, at least 1 character. Gmail search text. |
| `inputMailsCount` | number | Optional. Hard maximum of mails; ignored in `Bulk`. |
| `inputDateAfter` | string | Optional date. |
| `inputDateBefore` | string | Optional date. |

Validation rules:

- `Bulk`: neither date field allowed.
- `Sync`: neither date field allowed (the date comes from the database).
- `Increment`: at least one of `inputDateAfter` or `inputDateBefore` is required.

### 5.2 Query building (`FACT`)

| Mode | Final Gmail query | Limit |
| --- | --- | --- |
| `Bulk` | `query` | None. All pages. |
| `Sync` | `query` plus ` after:<unix seconds of last stored date>` | `inputMailsCount` if given. |
| `Increment` | `query` plus ` after:<unix>` and/or ` before:<unix>` | `inputMailsCount` if given. |

The last stored date for `Sync` comes from `[02sjobsData].[03spgetLastDateAppliedJobs]`. When the table is empty it raises error 50001, mapped to `NOT_FOUND` / `NO_EMAIL_DATA` (404). Other failures map to `INTERNAL_SERVER_ERROR` / `EXTRACT_FAILED`. The exact text format of the returned date is `UNVERIFIED` (earlier reading: latest `dateIST` as text); confirm before writing the parser.

### 5.3 Gmail reading (`FACT` for behavior)

- Page size 100 mails, 300 ms pause between pages, stop when there is no next page token.
- For each page, read the metadata of each mail in groups of 20 concurrently, with a 200 ms pause between groups.
- With a limit, stop as soon as enough mails are collected and cut the list to the limit.
- Each mail uses format `metadata` with headers `Date`, `From`, `Subject`.
- Text cleaning: remove control and invisible characters, collapse whitespace, trim.
- A mail is dropped silently if `From`, `Subject`, snippet or `Date` is empty after cleaning, if the date cannot be formatted, or if reading it fails.
- Dates are converted to IST with the format `YYYY-MM-DDTHH:MM:SS+05:30`. The old code adds a fixed +5:30 offset to the parsed instant.

Gmail request details (`UNVERIFIED`, from my general knowledge; verify against Google's official documentation in the build phase, `DECIDED` D4):

- List: `GET https://gmail.googleapis.com/gmail/v1/users/me/messages` with `q`, `maxResults`, `pageToken`.
- Read: `GET .../users/me/messages/{id}` with `format=metadata` and `metadataHeaders`.
- Token refresh: `POST https://oauth2.googleapis.com/token` with `grant_type=refresh_token`.

### 5.4 Credentials (`FACT`)

Credentials come from `[02sjobsData].[05spgetActiveGoogleOAuthCreds]` using `Google_APPID`. The result holds `clientId`, `clientSecret` and `refreshToken`. The old code caches the Gmail client for the life of the process; the Rust client should cache the access token until it expires (`PROPOSED`).

### 5.5 Gmail error mapping (`FACT`)

| Gmail status | Code |
| --- | --- |
| 400 or 401 | `UNAUTHORIZED` / `GMAIL_SESSION_EXPIRED` (401) |
| 403 | `FORBIDDEN` / `GMAIL_ACCESS_DENIED` (403) |
| 429 | `TOO_MANY_REQUESTS` / `GMAIL_RATE_LIMIT` (429) |
| Other status | `INTERNAL_SERVER_ERROR`, message `Gmail API Error (<status>)` (500) |
| No status (network) | `INTERNAL_SERVER_ERROR` (500) |

### 5.6 Chunked publish (`DECIDED`: D3, D10, D11, D12)

1. If no records were collected, answer 200 with the no-records response (below). Nothing is published.
2. Convert each mail to a record `{ id, date, from, subject, snippet }`, all strings.
3. Reverse the list so the oldest mail is first.
4. Split into chunks of 100 (the last chunk may be smaller).
5. For each chunk, in order: create a UUIDv7 `correlationId`, build the event (section 6), publish it to `JOBS_WRITE_TOPIC` through the gateway (section 7).
6. If a publish fails, stop. Chunks already published stay published. Return the error response; the message states how many chunks were already published.

Success response (200):

| Field | Value |
| --- | --- |
| `message` | Text stating the mode was accepted and that Azure and PostgreSQL processing is asynchronous. |
| `data.correlationIds` | List of UUIDv7, oldest chunk first. |
| `data.recordCount` | Total records across all chunks. |
| `data.chunkCount` | Number of chunks published. |
| `data.status` | `ACCEPTED` |

No-records response (200), `PROPOSED` to keep the old meaning: message "No messages found for the given query.", `data.correlationIds` `[]`, `data.recordCount` `0`, `data.chunkCount` `0`, `data.status` `NO_NEW_RECORDS`.

Publish-failure error code: `PROPOSED` `INTERNAL_SERVER_ERROR` / `KAFKA_GATEWAY_FAILED` (as in recordlog Rust). The old jobs code used the `DATABASE_ERROR` text for gateway failures, which misdescribes the cause.

## 6. Event contract (`FACT`)

Kafka message value for the write-model and read-model topics:

| Field | Rule |
| --- | --- |
| `correlationId` | UUID version 7. |
| `job.writeModel` | Non-empty. Azure procedure name, default `[02sjobsData].[02spinsertRawAppliedJobs]`. |
| `job.readModel` | Non-empty. PostgreSQL procedure name, default `sp_kafka_apply_committed_records`. |
| `sourceClientId` | Non-empty. From `SOURCE_CLIENT_ID`. |
| `records` | At least 1 item. Each item has non-empty `id`, `date`, `from`, `subject`, `snippet`. |

Status event (topic `JOBS_STATUS_TOPIC`):

| Field | Rule |
| --- | --- |
| `eventId` | UUID version 7. Generated once at publish time and kept across redelivery. |
| `correlationId` | UUID version 7. |
| `stage` | `RECEIVED`, `AZURE_WRITE_STARTED`, `AZURE_WRITE_SUCCEEDED`, `READ_MODEL_STARTED`, `READ_MODEL_SUCCEEDED`, `COMPLETED`, `FAILED`. |
| `status` | `STARTED`, `SUCCEEDED`, `FAILED`. |
| `timestamp` | ISO string, at least 10 characters. |
| `sequence` | Number or null. Assigned by the database. |
| `details` | Optional object. |

## 7. Publishing to the gateway (`FACT` for old behavior)

`POST {ONPREM_SERVER_URL}/api/v1/http-publish` with headers `x-client-id`, `x-client-secret`, `x-api-key` (`KAFKA_API_KEY`), `x-api-secret` (`KAFKA_API_KEY_SECRET`).

Body: `{ "topic", "correlationId", "value" }`. The gateway accepts only these three fields and uses `correlationId` as the Kafka message key (`FACT`, kafka gateway code).

Success reply: `202` with `success`, `statusCode: 202`, a `message` and `data` of `{ topic, correlationId }` (`OBSERVED`, D46: `kafka-api-gateway-rust` `SuccessResponse::accepted` and `HttpPublishData`; the `statusCode` rename of D29 is in that code).

Gateway reply field (`DECIDED`, D21): the jobs client reads `statusCode` and requires it to equal 202, as `recordlog-api-rust` does. Your rename in `kafka-api-gateway-rust` (D29) makes the real gateway send that field. The client is tested against a fake gateway in P4; a live publish check waits for P11.

Failure handling: any non-2xx status, a non-JSON body, or a reply that fails validation is a publish failure (section 5.6).

## 8. Subscriber

### 8.1 Connection (`FACT` for the old behavior, `PROPOSED` to copy the recordlog Rust transport)

One process, one authenticated WebSocket to the gateway, three subscribed topics, identity sent as `x-subscriber-id` from `SOURCE_CLIENT_ID`. The gateway-WebSocket files in the old repos for recordlog and jobs are identical (`FACT`, file comparison), so the recordlog Rust `transport.rs` is the model. The exact WebSocket path, frame types and acknowledgement frame are `UNKNOWN` here and must be read from `subscriber/src/transport.rs` in the build phase.

### 8.2 Handlers (`FACT`)

**Write-model** (topic `JOBS_WRITE_TOPIC`):

1. Validate the event.
2. Publish status `RECEIVED` / `SUCCEEDED`.
3. Publish status `AZURE_WRITE_STARTED` / `STARTED`.
4. Call the Azure procedure (section 9.1).
5. Publish status `AZURE_WRITE_SUCCEEDED` / `SUCCEEDED` (with `rowsInserted` and `replayed`).
6. Publish the same event to `JOBS_READ_TOPIC`, key `correlationId`.
7. Acknowledge the message.

**Read-model** (topic `JOBS_READ_TOPIC`):

1. Validate the event.
2. Publish `READ_MODEL_STARTED` / `STARTED`.
3. Call the PostgreSQL procedure (section 9.2).
4. Publish `READ_MODEL_SUCCEEDED` / `SUCCEEDED`, then `COMPLETED` / `SUCCEEDED`.
5. Acknowledge.

**Status** (topic `JOBS_STATUS_TOPIC`): validate, call `fn_pipeline_apply_status` with the original `eventId`, acknowledge. On failure there is no acknowledgement and no dead-letter topic; redelivery is left to the gateway and Kafka.

**Failure of write-model or read-model:** publish a best-effort `FAILED` status (never throws), then retry or dead-letter (section 8.3), then acknowledge. If the retry or dead-letter publish itself fails, do not acknowledge. An event without a readable `correlationId` is logged and not acknowledged.

### 8.3 Retry and dead letter (`FACT`)

- Maximum 3 retries per stage. The first delivery has count 0.
- Count below 3: republish the original payload to the same stage topic with the retry header incremented.
- Count at 3 or more: publish to the stage dead-letter topic.

| Header | Meaning |
| --- | --- |
| `x-jobs-retry-count` | Current retry attempt. A missing, negative or non-numeric value counts as 0. |
| `x-original-topic`, `x-original-partition`, `x-original-offset` | Where the message came from (dead-letter only; partition and offset only when known). |
| `x-correlation-id` | The correlation id. |
| `x-failure-reason` | Failure text cut to 500 characters. |

Note: recordlog uses the header `x-record-log-retry-count`; jobs keeps `x-jobs-retry-count`.

## 9. Data contracts

### 9.1 Azure SQL (service login, `FACT`)

| Object | Contract |
| --- | --- |
| `[02sjobsData].[02spinsertRawAppliedJobs]` | Inputs `@correlationId` (UNIQUEIDENTIFIER) and `@input` (NVARCHAR(MAX), JSON array of records; `$.id` maps to `emailId`). Returns one row `result = 'SUCCESS'` and `notes` (JSON with `rowsInserted`, `message`, `replayed`). The same `correlationId` replays the stored result. Existing `emailId`s are skipped, so zero inserted rows is still success. Raises 50001, 50002, 50003 and 50010 for input and state errors. |
| `[02sjobsData].[03spgetLastDateAppliedJobs]` | No inputs. Returns `result = 'SUCCESS'` and `date`. Raises 50001 when there are no rows. |
| `[02sjobsData].[05spgetActiveGoogleOAuthCreds]` | Input app id. Returns client id, client secret and refresh token. Raises 50001, 50002, 50003 (meanings `UNVERIFIED`; re-read the procedure). |
| `[02sjobsData].[04spupsertGoogleOAuthToken]` | Used by `gen-token` to store the refresh token. Raises 50001, 50002, 50003 (meanings `UNVERIFIED`). |
| `01tportalUsers`, `02tapiUsers` | User lookup. Table scripts are not in the provided tarballs (`UNKNOWN` columns; copy from the recordlog Rust queries). |

### 9.2 PostgreSQL on-prem (service login, `FACT`)

| Object | Contract |
| --- | --- |
| `CALL sp_kafka_apply_committed_records($1::uuid, $2::jsonb, NULL, NULL, NULL)` | Run inside one transaction. Returns one row with `result`, `replayed`, `rows_inserted`. `result` must be `SUCCESS`. The procedure name comes from the event and must match `^[a-zA-Z0-9_."]+$` and be at most 200 characters. |
| `fn_pipeline_apply_status(eventId uuid, correlationId uuid, stage varchar, status varchar, timestamp timestamptz, details jsonb)` | Run inside one transaction. Returns `result`, `replayed`, `assigned_sequence`, `current_stage`, `current_status`. `result` must be `SUCCESS`. |

The Azure procedure name from the event must match `^[a-zA-Z0-9_."\[\]]+$` and be at most 200 characters (`FACT`). The Rust version keeps both name checks (`PROPOSED`).

## 10. Environment variables

Loaded and validated at start-up (`PROPOSED` to follow recordlog Rust: serde plus `validator`). Names are case-sensitive and kept exactly as in `.env.example`.

| Name | Used by | Rule (from old schema) |
| --- | --- | --- |
| `APP_NAME` | API | 1 to 50 characters. |
| `APP_ENV` | API, subscriber | `DECIDED` D13, D30. Allowed values `development`, `preview`, `production`. |
| `APP_PORT` | API | Positive integer. |
| `CLIENT_ID` | API | 1 to 50 characters (header limit is 30). |
| `CLIENT_SECRET` | API | 1 to 50 characters. |
| `JWT_SECRET` | API | 1 to 50 characters. |
| `AZURE_DB_SERVER`, `AZURE_DB_DATABASE` | API, subscriber | 1 to 50 characters. |
| `AZURE_DB_USER_PUBLIC`, `AZURE_DB_PASSWORD_PUBLIC` | API | 1 to 50 characters. |
| `AZURE_DB_USER_SERVICE`, `AZURE_DB_PASSWORD_SERVICE` | API, subscriber | 1 to 50 characters. |
| `ONPREM_SERVER_URL` | API, subscriber | Valid URL. |
| `KAFKA_API_KEY` | API, subscriber | 1 to 50 characters. |
| `KAFKA_API_KEY_SECRET` | API, subscriber | 1 to 300 characters. |
| `SOURCE_CLIENT_ID` | API, subscriber | 1 to 100 characters. |
| `JOBS_WRITE_TOPIC` | API, subscriber | 1 to 100 characters. |
| `JOBS_WRITE_DLQ_TOPIC` | Subscriber | 1 to 100 characters. |
| `JOBS_READ_TOPIC` | Subscriber | 1 to 100 characters. |
| `JOBS_READ_DLQ_TOPIC` | Subscriber | 1 to 100 characters. |
| `JOBS_STATUS_TOPIC` | Subscriber | 1 to 100 characters. |
| `JOBS_WRITE_MODEL_PROCEDURE` | API | Default `[02sjobsData].[02spinsertRawAppliedJobs]`. |
| `JOBS_READ_MODEL_PROCEDURE` | API | Default `sp_kafka_apply_committed_records`. |
| `ONPREM_DB_SERVER` | Subscriber | 1 to 50 characters. |
| `ONPREM_DB_PORT` | Subscriber | Positive integer. |
| `ONPREM_DB_DATABASE` | Subscriber | 1 to 50 characters. |
| `ONPREM_DB_USER_SERVICE`, `ONPREM_DB_PASSWORD_SERVICE` | Subscriber | 1 to 50 characters. |
| `Google_APPID` | API, `xtask gen-token` | Number between 1 and 300. |

The "Used by" column is `PROPOSED`, derived from where the old code reads each value. The old schema labels `APP_NAME` as "App environment", which is a labeling slip; `APP_NAME` is the application name.

`.env.production` and `.env.development` still use `NODE_ENV`; they are renamed to `APP_ENV` (`DECIDED`, D13). Real values never appear in this file.

## 11. Error catalog

The Rust catalog follows `recordlog-api-rust/src/errors/catalog.rs` (`FACT`: entries of code, message and error text). Jobs needs these codes in addition to the common ones (`FACT`, from the old catalog; `USER_NOT_FOUND` is not used, D42):

| Code (HTTP) | Entries used by jobs |
| --- | --- |
| `UNAUTHORIZED` (401) | `MISSING_AUTH_HEADERS`, `MISSING_AUTHORIZATION_TOKEN`, `INVALID_TOKEN`, `EXPIRED_TOKEN`, `MISSING_SUB_OR_ROLE`, `INVALID_CREDENTIALS`, `GMAIL_SESSION_EXPIRED`, `NO_ACTIVE_CREDENTIALS` |
| `BAD_REQUEST` (400) | `INVALID_HEADER_VALUES`, `INVALID_INPUT`, validation messages |
| `FORBIDDEN` (403) | `ACCESS_DENIED` (portal login posted by a non-browser), `GMAIL_ACCESS_DENIED` |
| `NOT_FOUND` (404) | `NO_EMAIL_DATA`, `CLIENT_ID_NOT_FOUND`, `APPS_TABLE_MISSING`, `TOKENS_TABLE_MISSING` |
| `TOO_MANY_REQUESTS` (429) | `GMAIL_RATE_LIMIT` |
| `INTERNAL_SERVER_ERROR` (500) | `SERVER_ERROR`, `DATABASE_ERROR`, `TOKEN_SIGNING_FAILED`, `EXTRACT_FAILED`, `KAFKA_GATEWAY_FAILED` (`PROPOSED`) |
| `SERVICE_UNAVAILABLE` (503) | `DB_PROBE_FAILED` (`DECIDED`, D43) |

The old catalog has many more entries (412, 413, 415, 422, 428, 499, 501, 502, 503, 504). Whether to port all of them or only the used ones is `UNKNOWN`; `PLAN.md` proposes porting only the entries the code uses (`PROPOSED`).

## 12. `xtask`

- `gen-token` (`DECIDED`, D5, D14): prints the Google consent URL with scopes `gmail.readonly` and `drive`, asks for the pasted code, exchanges it for tokens, and saves the refresh token through `[02sjobsData].[04spupsertGoogleOAuthToken]` for the app id in `Google_APPID`. The old redirect URI is `http://localhost:3000`. The exact prompts and flow follow the old `generate-token.ts`.
- Helper commands (`PROPOSED`, approved by D20): the same cargo aliases as `recordlog-api-rust` (`FACT`): `css`, `dev`, `start`, `build-prod`, `dev-subscriber`, `build-subscriber`, `start-subscriber` and the `docker-*` family (`build`, `up`, `down`, `start`, `stop`, `restart`, `logs`, `logs-clear`, `rmi`, `clean`).

## 13. Behavior changes from the old gateway

| Change | Source |
| --- | --- |
| `extractMails` sends several events and returns `correlationIds`, `recordCount`, `chunkCount`, `status`. | D3, D12 |
| Chunks are published oldest first; a failure stops the request. | D10, D11 |
| Gateway failures use `KAFKA_GATEWAY_FAILED`, not the `DATABASE_ERROR` text. | `PROPOSED` |
| Environment variable `APP_ENV` replaces `NODE_ENV`. | D13 |
| Logging uses `tracing`. | D9 |
| The gateway client reads `statusCode` (gateway renamed by you, D29). | D21 |
| The jobs API's own responses use `statusCode` instead of `httpCode`. | D22 |
| Seven dev scripts are not ported. | D6 |

## 14. Unknowns to resolve before or during the build

1. Exact gateway reply `data` shape and WebSocket frame details (read the gateway and `transport.rs`).
2. Gmail and Google OAuth endpoint details (verify with official docs).
3. Text format returned by `03spgetLastDateAppliedJobs`.
4. Columns of `01tportalUsers` and `02tapiUsers`.
5. Allowed `APP_ENV` values and the `tracing` format and filter (D9).
6. Real Vercel function time limit for the Rust runtime (D2 risk).
7. Whether the Kafka topic has one partition or several (D10 residual risk).