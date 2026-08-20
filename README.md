# Jobs API Gateway

Jobs pipeline API and Kafka subscriber for applied-job email ingest.

## Purpose

- Authenticated HTTP API for Gmail applied-job extraction
- Publish domain events to **Kafka API Gateway** (not direct Azure writes on the Kafka path)
- **One** permanent Jobs subscriber process with **one** WebSocket and **three** handlers

## Architecture

```
Client
  -> Jobs API (/api/v1/admin/extractMails)
  -> Kafka API Gateway (HTTP publish)
  -> Kafka
  -> ONE Jobs subscriber (WebSocket)
       ├─ jobs-ingest-write-model  -> Azure SQL
       ├─ jobs-ingest-read-model   -> PostgreSQL
       └─ jobs-status              -> PostgreSQL pipeline state
```

See `proj-flowchart.txt` and `docs/ARCHITECTURE-SUBSCRIBER.md`.

### Important boundaries

- HTTP route does **not** call Azure `02spinsertRawAppliedJobs` for the Kafka pipeline.
- PostgreSQL read-model does **not** query Azure.
- Kafka-originated rows use `raw_applied_jobs.azure_id = NULL`.
- Azure Change Tracking is a **separate** Azure → PostgreSQL sync path (not the Kafka read-model).

## Kafka topics

| Topic | Consumer | Purpose | DLQ |
|-------|----------|---------|-----|
| `jobs-ingest-write-model` | Jobs subscriber | Azure write model | `jobs-ingest-write-model-dlq` |
| `jobs-ingest-read-model` | Jobs subscriber | PostgreSQL read model | `jobs-ingest-read-model-dlq` |
| `jobs-status` | Jobs subscriber | Pipeline status | None |

The subscriber does **not** consume DLQ topics as normal processors. There is **no** status DLQ.

## Subscriber handlers

1. **Write** (`jobs-ingest-write-model`)
   - Validate event → status RECEIVED → AZURE_WRITE_STARTED
   - Call Azure `02spinsertRawAppliedJobs(@correlationId, @input)`
   - `rowsInserted = 0` is **SUCCESS**
   - AZURE_WRITE_SUCCEEDED → publish same event to read-model topic → ACK
   - Failure → retry (`x-jobs-retry-count`) → write DLQ → ACK

2. **Read** (`jobs-ingest-read-model`)
   - `CALL sp_kafka_apply_committed_records(uuid, jsonb, …)`
   - READ_MODEL_SUCCEEDED → COMPLETED → ACK
   - Failure → retry → read DLQ → ACK

3. **Status** (`jobs-status`)
   - `fn_pipeline_apply_status` with preserved `eventId`
   - ACK on success; no ACK on failure (redelivery); no status DLQ

## Databases

### Azure SQL (`02sjobsData`)

| Object | Role |
|--------|------|
| `04trawAppliedJobs` | Business table (unique `emailId`) |
| `07twriteOperation` | Kafka operation idempotency (`correlationId`) |
| `02spinsertRawAppliedJobs` | Write SP (`@correlationId`, `@input`) |

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

## Environment

Copy `.env.example` to `.env.development` / `.env.production` and fill secrets locally.

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

Also required: Azure SQL, On-prem PostgreSQL, Kafka Gateway auth, JWT/client credentials (see `.env.example`).

### Development & Build

| Bun Command                                | WD                                   | Bun Command with CWD                                                         | With Absolute Path     | Description                                                                                               |
| ------------------------------------------ | ------------------------------------ | ---------------------------------------------------------------------------- | ---------------------- | --------------------------------------------------------------------------------------------------------- |
| `bun run dev`                              | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway dev`                            | NA                     | Compiles Tailwind CSS and starts the development server with hot reloading using `.env.development`.      |
| `bun run type-check`                       | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway type-check`                     | NA                     | Runs TypeScript type checking without emitting build artifacts.                                           |
| `bun run lint`                             | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway lint`                           | NA                     | Runs ESLint on the codebase using a local cache file.                                                     |
| `bun run test`                             | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway test`                           | NA                     | Runs all test suites via Bun test runner using `.env.development`.                                        |
| `bun run test:db`                          | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway test:db`                        | NA                     | Executes database select test script using `.env.development`.                                            |
| `bun run test:ingest`                      | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway test:ingest`                    | NA                     | Executes the ingest test runner using `.env.development`.                                                 |
| `bun run token:gen`                        | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway token:gen`                      | NA                     | Generates authentication tokens using `.env.development`.                                                 |
| `bun run token:getexpat`                   | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway token:getexpat`                 | NA                     | Runs script to inspect token expiration details using `.env.development`.                                 |
| `bun run test:count-bulk`                  | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway test:count-bulk`                | NA                     | Runs Bulk mode for label:jobs-mails-applied-jobs and prints the total message count.                      |
| `bun run test:count-sync`                  | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway test:count-sync`                | NA                     | Runs sync mode for label:jobs-mails-applied-jobs and prints the total message count.                      |
| `bun run build`                            | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway build`                          | NA                     | Cleans dist, compiles CSS, runs tsc, resolves path aliases, and copies static public assets.              |
| `bun run start`                            | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway start`                          | NA                     | Starts the compiled production server (`dist/server.js`) using `.env.production`.                         |
| `bun run clean`                            | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway clean`                          | NA                     | Deletes all build artifacts and hidden files in the `dist` directory.                                     |
| `bun run dev:jobs-subscriber`              | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway dev:jobs-subscriber`            | NA                     | Runs the jobs subscriber entrypoint in development mode using `.env.development`.                         |
| `bun run test:jobs-subscriber`             | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway test:jobs-subscriber`           | NA                     | Runs test suite specifically targeting the jobs subscriber module.                                        |
| `bun run build:jobs-subscriber`            | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway build:jobs-subscriber`          | NA                     | Bundles the jobs subscriber into a single JavaScript artifact (`dist/jobs-subscriber.js`).                 |
| `bun run start:jobs-subscriber`            | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway start:jobs-subscriber`          | NA                     | Starts the bundled jobs subscriber script using `.env.production`.                                        |
| `bun run compile:jobs-subscriber`          | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway compile:jobs-subscriber`        | NA                     | Compiles the jobs subscriber into a standalone binary in `dist/`.                                         |
| `bun run start:jobs-subscriber:binary`     | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway start:jobs-subscriber:binary`   | NA                     | Executes the compiled standalone jobs subscriber binary using `.env.production`.                          |
| `bun run madge:jobs-subscriber`            | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway madge:jobs-subscriber`          | NA                     | Generates a visual dependency graph image (`graph-subscriber.svg`) for the jobs subscriber.               |
| `bun run madge:jobs-subscriber:json`       | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway madge:jobs-subscriber:json`     | NA                     | Outputs module dependencies in JSON format for the jobs subscriber.                                       |
| `bun run madge:jobs-subscriber:circular`   | `~/dev/vs-code/jobs-api-gateway`     | `bun run --cwd ~/dev/vs-code/jobs-api-gateway madge:jobs-subscriber:circular` | NA                     | Analyzes the jobs subscriber codebase and reports any circular dependencies.                              |

### Docker Operations

| Bun Command                                            | WD                                         | Bun Command with CWD                                                                     | With Absolute Path                                                                                                                                                            | Description                                                                                     |
| ------------------------------------------------------ | ------------------------------------------ | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------| ----------------------------------------------------------------------------------------------- |
| `bun run docker:build`                                 | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:build`                               | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml build`                                                                                           | Builds all images defined in docker-compose.                                                    |
| `bun run docker:build:jobs-subscriber`                 | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:build:jobs-subscriber`               | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml build jobs-subscriber`                                                                           | Builds only the jobs-subscriber image.                                                          |
| `bun run docker:up`                                    | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:up`                                  | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml up -d`                                                                                           | Creates and starts all containers in detached mode.                                             |
| `bun run docker:up:jobs-subscriber`                    | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:up:jobs-subscriber`                  | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml up -d jobs-subscriber`                                                                           | Creates and starts only the jobs-subscriber container in detached mode.                         |
| `bun run docker:down`                                  | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:down`                                | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml down`                                                                                            | Stops and removes all containers and networks.                                                  |
| `bun run docker:down:jobs-subscriber`                  | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:down:jobs-subscriber`                | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml stop jobs-subscriber`                                                                            | Stops the jobs-subscriber container.                                                            |
| `bun run docker:start`                                 | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:start`                               | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml start`                                                                                           | Starts existing stopped containers for all services.                                            |
| `bun run docker:start:jobs-subscriber`                 | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:start:jobs-subscriber`               | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml start jobs-subscriber`                                                                           | Starts the stopped jobs-subscriber container.                                                   |
| `bun run docker:stop`                                  | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:stop`                                | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml stop`                                                                                            | Stops running containers for all services without removing them.                                |
| `bun run docker:stop:jobs-subscriber`                  | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:stop:jobs-subscriber`                | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml stop jobs-subscriber`                                                                            | Stops the running jobs-subscriber container.                                                    |
| `bun run docker:restart`                               | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:restart`                             | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml restart`                                                                                         | Restarts all running services.                                                                  |
| `bun run docker:restart:jobs-subscriber`               | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:restart:jobs-subscriber`             | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml restart jobs-subscriber`                                                                         | Restarts the jobs-subscriber service.                                                           |
| `bun run docker:logs`                                  | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:logs`                                | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml logs -f`                                                                                         | Streams real-time logs for all services.                                                        |
| `bun run docker:logs:jobs-subscriber`                  | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:logs:jobs-subscriber`                | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml logs -f jobs-subscriber`                                                                         | Streams real-time logs for the jobs-subscriber service.                                         |
| `bun run docker:logs:clear`                            | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:logs:clear`                          | `for c in $(docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml ps -q); do sudo truncate -s 0 $(docker inspect --format='{{.LogPath}}' $c); done`     | Truncates/clears container log files for all running compose services.                          |
| `bun run docker:logs:clear:jobs-subscriber`            | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:logs:clear:jobs-subscriber`          | `sudo truncate -s 0 $(docker inspect --format='{{.LogPath}}' jobs-subscriber) 2>/dev/null; true`                                                                             | Truncates/clears the log file for the jobs-subscriber container.                                |
| `bun run docker:rmi`                                   | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:rmi`                                 | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml down --rmi local`                                                                                | Stops containers and removes locally built images.                                              |
| `bun run docker:rmi:jobs-subscriber`                   | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:rmi:jobs-subscriber`                 | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml rm -sf jobs-subscriber && docker image rm -f jobs-subscriber:latest || true`                     | Force-removes container and local image for jobs-subscriber.                                    |
| `bun run docker:clean`                                 | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:clean`                               | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml down --rmi local --volumes --remove-orphans`                                                     | Full reset: stops containers, removes local images, attached volumes, and orphans.              |
| `bun run docker:clean:jobs-subscriber`                 | `~/dev/vs-code/jobs-api-gateway`           | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:clean:jobs-subscriber`               | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml rm -sf jobs-subscriber && docker image rm -f jobs-subscriber:latest || true`                     | Full reset specifically for the jobs-subscriber container and image.                            |

Service/image/container name: **`jobs-subscriber`**.

## Local development

```bash
cp .env.example .env.development
# fill secrets
bun install
bun run type-check
bun run test
bun run subscriber:dev   # terminal 1
bun run dev              # terminal 2
```

## Production subscriber

```bash
bun run compile:subscriber
# or Docker
bun run docker:up:subscriber
```

## Testing

```bash
bun run test
bun run type-check
bun run lint
```

## Troubleshooting

- Subscriber exits on startup: ensure all `JOBS_*` and `SOURCE_CLIENT_ID` vars are set.
- Write succeeds with `rowsInserted=0`: all `emailId`s already existed — this is success.
- Read path errors: confirm Phase 1 `sp_kafka_apply_committed_records` is installed and role `gdg_proj_service` has EXECUTE.
- Status not updating: confirm `jobs-status` topic and `fn_pipeline_apply_status` exist.

# Jobs Subscriber Architecture

## Final shape (Phase 2B)

- **One** process: `src/lib/jobs-subscriber/main.ts`
- **One** authenticated WebSocket to Kafka API Gateway
- **Three** handlers: write → Azure, read → PostgreSQL, status → PostgreSQL

## Boundaries

| Layer | Responsibility |
|-------|----------------|
| HTTP API | Gmail ingest + publish write-model event |
| Kafka API Gateway | Transport only (DB-backed topic registry) |
| Jobs subscriber | Topic routing, ACK, retry, DLQ |
| Azure | Write SP + `07twriteOperation` idempotency |
| PostgreSQL | Read-model SP, status function, pipeline tables |

## Idempotency

| Key | Layer |
|-----|-------|
| `correlationId` | Azure write operation + PG kafka_write_operation |
| `emailId` | Business uniqueness on Azure/PG raw tables |
| `eventId` | Status history rows |

## Kafka-first read path

Kafka → `raw_applied_jobs` with `azure_id = NULL`.  
Azure CT sync is separate and reconciles by `azure_id` or `email_id`.

## DLQ

- Write DLQ / Read DLQ after max retries
- **No** status DLQ
