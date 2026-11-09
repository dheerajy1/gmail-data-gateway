# Jobs API Gateway

Rust workspace: a Jobs HTTP API for applied-job email ingest and one Kafka subscriber.

## Purpose

- Authenticated HTTP API that reads applied-job mails from Gmail and normalizes them into records
- Publishes the records as domain events to the **Kafka API Gateway** (the API never writes to Azure on the Kafka path)
- **One** permanent subscriber process with **one** WebSocket and **three** handlers

## Architecture

```
Client
  -> Jobs API (POST /api/v1/admin/extractMails)
  -> Kafka API Gateway (HTTP publish, chunks of 100 records)
  -> Kafka
  -> ONE Jobs subscriber (WebSocket)
       ├─ jobs-ingest-write-model  -> Azure SQL
       ├─ jobs-ingest-read-model   -> PostgreSQL
       └─ jobs-status              -> PostgreSQL pipeline state
```

See `proj-flowchart.txt` for the full flow and `docs/SPEC.md` for the contracts.

### Important boundaries

- The HTTP route does **not** call the Azure write procedure `02spinsertRawAppliedJobs`.
- The PostgreSQL read model does **not** query Azure.
- Kafka-originated rows use `raw_applied_jobs.azure_id = NULL`.
- Azure Change Tracking is a **separate** Azure → PostgreSQL sync path (not the Kafka read model).

## HTTP surface

| Method and path | Auth | Purpose |
| --- | --- | --- |
| `GET /`, `/home`, `/about` | Gate | HTML pages |
| `GET /login`, `POST /login` | Public | Docs login page, sets the `auth` cookie |
| `GET /healthz`, `/healthz/app`, `/healthz/db` | Gate | HTML health pages |
| `GET /openapi` | Gate | API documentation |
| `GET /api/v1/health` | Gate | JSON health with `data.app` and `data.db`, 503 while the database is down |
| `POST /api/v1/auth/login` | Client headers | API login, returns a JWT |
| `POST /api/v1/admin/extractMails` | Client headers and JWT | Gmail extraction and publish |
| `GET /global.css`, `/logo.png`, `/favicon.ico` | Public | Embedded assets |

## Kafka topics

| Topic | Consumer | Purpose | DLQ |
| ----- | -------- | ------- | --- |
| `jobs-ingest-write-model` | Jobs subscriber | Azure write model | `jobs-ingest-write-model-dlq` |
| `jobs-ingest-read-model` | Jobs subscriber | PostgreSQL read model | `jobs-ingest-read-model-dlq` |
| `jobs-status` | Jobs subscriber | Pipeline status | None |

The subscriber does **not** consume DLQ topics as normal processors. There is **no** status DLQ.

## Subscriber handlers

1. **Write** (`jobs-ingest-write-model`)
   - Validate event → status RECEIVED → AZURE_WRITE_STARTED
   - Call the Azure write procedure with `@correlationId` and `@input`
   - `rowsInserted = 0` is **SUCCESS**
   - AZURE_WRITE_SUCCEEDED → publish the same event to the read-model topic → ACK
   - Failure → retry (`x-jobs-retry-count`, at most 3) → write DLQ → ACK

2. **Read** (`jobs-ingest-read-model`)
   - READ_MODEL_STARTED → `CALL sp_kafka_apply_committed_records(uuid, jsonb, …)`
   - READ_MODEL_SUCCEEDED → COMPLETED → ACK
   - Failure → retry → read DLQ → ACK

3. **Status** (`jobs-status`)
   - `fn_pipeline_apply_status` with the preserved `eventId`
   - ACK on success; no ACK on failure (redelivery); no status DLQ

The procedure names come from the event, they are not hardcoded.

## Databases

### Azure SQL (`02sjobsData`)

| Object | Role |
|--------|------|
| `04trawAppliedJobs` | Business table (unique `emailId`) |
| `07twriteOperation` | Kafka operation idempotency (`correlationId`) |
| `02spinsertRawAppliedJobs` | Write SP (`@correlationId`, `@input`) |
| `04spupsertGoogleOAuthToken` | Stores the Google refresh token (`cargo gen-token`) |

### PostgreSQL (`jobs_data_db`)

| Object | Role |
|--------|------|
| `raw_applied_jobs` | Raw store; `azure_id` **nullable** for Kafka rows |
| `bronze_applied_jobs` | Cleaned/transformed jobs |
| `kafka_write_operation` | Kafka operation idempotency |
| `pipeline_event` | Immutable status history |
| `pipeline_execution` | Current pipeline state |
| `sp_kafka_apply_committed_records` | Read-model orchestrator SP |
| `fn_pipeline_apply_status` | Status apply function |

Azure → PostgreSQL CT sync reconciles by `azure_id` first, then by `email_id` so Kafka rows can receive their Azure ID later.

## Idempotency

| Key | Layer |
|-----|-------|
| `correlationId` | Azure write operation + PostgreSQL `kafka_write_operation` |
| `emailId` | Business uniqueness on the Azure and PostgreSQL raw tables |
| `eventId` | Status history rows |

## Environment

Copy `.env.example` to `.env.development`, `.env.preview` and `.env.production` and fill the secrets locally. `APP_ENV` is `development`, `preview` or `production`.

Required subscriber variables:

```text
SOURCE_CLIENT_ID=jobs-api-gateway
JOBS_WRITE_TOPIC=jobs-ingest-write-model
JOBS_WRITE_DLQ_TOPIC=jobs-ingest-write-model-dlq
JOBS_READ_TOPIC=jobs-ingest-read-model
JOBS_READ_DLQ_TOPIC=jobs-ingest-read-model-dlq
JOBS_STATUS_TOPIC=jobs-status
JOBS_WRITE_MODEL_PROCEDURE=[02sjobsData].[02spinsertRawAppliedJobs]
JOBS_READ_MODEL_PROCEDURE=sp_kafka_apply_committed_records
```

Also required: Azure SQL, on-prem PostgreSQL, Kafka Gateway auth and JWT/client credentials (see `.env.example`). The process exits at boot when the env is invalid.

## Scripts & Commands

### Development & Build

| Cargo Command | WD | Description |
| --- | --- | --- |
| `cargo fmt` | `~/dev/vs-code/jobs-api-gateway-rust` | Formats all code. |
| `cargo check --workspace --all-targets` | `~/dev/vs-code/jobs-api-gateway-rust` | Type-checks the API, the subscriber and xtask without building. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | `~/dev/vs-code/jobs-api-gateway-rust` | Lints the whole workspace and fails on any warning. |
| `cargo test --workspace` | `~/dev/vs-code/jobs-api-gateway-rust` | Runs all tests. No real database, gateway or Google call is needed. |
| `cargo test --test <name>` | `~/dev/vs-code/jobs-api-gateway-rust` | Runs one test file, for example `cargo test --test gate`. |
| `cargo css` | `~/dev/vs-code/jobs-api-gateway-rust` | Compiles `src/tailwind.css` into `public/global.css` with the Tailwind CLI. |
| `cargo dev` | `~/dev/vs-code/jobs-api-gateway-rust` | Validates `.env.development`, compiles CSS and runs the API on `APP_PORT`. |
| `cargo preview` | `~/dev/vs-code/jobs-api-gateway-rust` | Validates `.env.preview`, compiles CSS and runs the release API. |
| `cargo build-prod` | `~/dev/vs-code/jobs-api-gateway-rust` | Validates `.env.production`, compiles CSS and builds the API in release mode. |
| `cargo start` | `~/dev/vs-code/jobs-api-gateway-rust` | Validates `.env.production`, compiles CSS and runs the release API. |
| `cargo dev-subscriber` | `~/dev/vs-code/jobs-api-gateway-rust` | Validates `.env.development` and runs the subscriber. |
| `cargo build-subscriber` | `~/dev/vs-code/jobs-api-gateway-rust` | Validates `.env.production` and builds the subscriber as one standalone binary: `target/release/subscriber`. |
| `cargo start-subscriber` | `~/dev/vs-code/jobs-api-gateway-rust` | Validates `.env.production` and runs the release subscriber. |
| `cargo gen-token` | `~/dev/vs-code/jobs-api-gateway-rust` | Loads `.env.production`, prints the Google consent URL, reads the pasted code and stores the refresh token. The token is shown masked. |
| `cargo clean` | `~/dev/vs-code/jobs-api-gateway-rust` | Deletes the `target/` folder. |

### Docker Operations

| Cargo Command | WD | Description |
| --- | --- | --- |
| `cargo docker-build` | `~/dev/vs-code/jobs-api-gateway-rust` | Builds all images defined in the compose file. |
| `cargo docker-up` | `~/dev/vs-code/jobs-api-gateway-rust` | Creates and starts all containers in detached mode. |
| `cargo docker-down` | `~/dev/vs-code/jobs-api-gateway-rust` | Stops and removes all containers and networks. |
| `cargo docker-start` | `~/dev/vs-code/jobs-api-gateway-rust` | Starts existing stopped containers. |
| `cargo docker-stop` | `~/dev/vs-code/jobs-api-gateway-rust` | Stops running containers without removing them. |
| `cargo docker-restart` | `~/dev/vs-code/jobs-api-gateway-rust` | Restarts all running services. |
| `cargo docker-logs` | `~/dev/vs-code/jobs-api-gateway-rust` | Streams the logs of all services. |
| `cargo docker-logs-clear` | `~/dev/vs-code/jobs-api-gateway-rust` | Empties the log file of every container in the project (asks for the sudo password). |
| `cargo docker-rmi` | `~/dev/vs-code/jobs-api-gateway-rust` | Stops the containers and removes the locally built images. |
| `cargo docker-clean` | `~/dev/vs-code/jobs-api-gateway-rust` | Full reset: stops the containers and removes the local images and volumes. |

Service, image and container name of the subscriber: **`jobs-subscriber`**. The API service in `docker/docker-compose.yml` stays disabled because the API runs on Vercel.

## Local development

```bash
cp .env.example .env.development
# fill secrets
cargo test --workspace
cargo dev-subscriber   # terminal 1
cargo dev              # terminal 2
```

## Production subscriber

```bash
cargo build-subscriber
cargo start-subscriber
# or Docker
cargo docker-up
```

## Testing

```bash
cargo fmt
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace
```

## Project layout

```text
Cargo.toml               workspace: API crate, subscriber, xtask
.cargo/config.toml       cargo aliases
src/                     API: config, schemas, errors, middleware, clients, db, features/v1, pages, components, openapi
subscriber/src/          transport, gateway, event, handlers, azure, postgres, retry, failure, status, topics
xtask/src/               dev, build, docker and gen-token commands
tests/                   API integration tests with fakes
public/                  global.css, logo.png, favicon.ico (committed)
docker/                  Dockerfile, Dockerfile.subscriber, docker-compose.yml
docs/                    SPEC, PLAN, BUILD, DECISIONS, FEATURES, CODING-STANDARDS
```

## Troubleshooting

- Subscriber exits on startup: ensure all `JOBS_*` and `SOURCE_CLIENT_ID` variables are set.
- Write succeeds with `rowsInserted=0`: all `emailId`s already existed, this is success.
- Read path errors: confirm `sp_kafka_apply_committed_records` is installed and the service role has EXECUTE.
- Status not updating: confirm the `jobs-status` topic and `fn_pipeline_apply_status` exist.
- `cargo css` fails: confirm `tailwindcss` is on the PATH.
