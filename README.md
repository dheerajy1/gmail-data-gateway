# Gmail-data-gateway

Gmail data gateway

# API Documentation Examples

## Bulk

Behavior:

1. Pull everything

2. Full pagination

3. Ignores DB

```JSON
{
  "type": "Bulk",
  "query": "label:job's-mails-applied-jobs"
}
```

## Sync (Production Cron Job)

Behavior:

1. Gets MAX(date) from DB

2. Builds after:lastDate

3. Fetches only new mails

4. Safe to call repeatedly

5. Idempotent

Note:

1. Uses DB max(date).

```JSON
{
  "type": "Sync",
  "query": "label:job's-mails-applied-jobs"
}
```

## Increment (Manual Window)

```JSON
{
  "type": "Increment",
  "query": "label:job's-mails-applied-jobs",
  "inputDateAfter": "2026-03-01T00:00:00+05:30"
}
```

## Increment With Range

```JSON
{
  "type": "Increment",
  "query": "label:job's-mails-applied-jobs",
  "inputDateAfter": "2026-03-01T00:00:00+05:30",
  "inputDateBefore": "2026-03-02T00:00:00+05:30"
}
```

---

## Scripts & Commands

All commands are managed via `bun`.

### Development & Build

| Bun Command                        | WD                                 | Bun Command with CWD                                                      | With Absolute Path                                                                  | Description                                                                                |
|------------------------------------|------------------------------------|---------------------------------------------------------------------------|-------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------  |
| `bun run dev`                      | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway dev`                        | NA                                                                                  | Starts the dev server with hot reloading and builds Tailwind CSS on change.                |
| `bun run build`                    | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway build`                      | NA                                                                                  | Cleans `dist/`, builds CSS, compiles TypeScript, resolves paths, and copies static assets. |
| `bun run start`                    | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway start`                      | NA                                                                                  | Runs the compiled production server from `dist/server.js` with `NODE_ENV=production`.      |
| `bun run lint`                     | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway lint`                       | NA                                                                                  | Runs ESLint using local cache.                                                             |
| `bun run clean`                    | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway clean`                      | NA                                                                                  | Removes the contents of the `dist/` directory.                                             |
| `bun run test:db`                  | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway test:db`                    | NA                                                                                  | Executes database select test script.                                                      |
| `bun run test:ingest`              | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway test:ingest`                | NA                                                                                  | Executes the ingest test runner.                                                           |
| `bun run token:gen`                | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway token:gen`                  | NA                                                                                  | Generates a token.                                                                         |
| `bun run token:getexpat`           | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway token:getexpat`             | NA                                                                                  | Executes token expiration check script.                                                    |
| `bun run dev:subscriber`           | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway dev:subscriber`             | `bun --cwd ~/dev/vs-code/jobs-api-gateway src/lib/azure-sync-job-subscriber.ts`     | Runs the subscriber directly with Bun (development mode).                                  |
| `bun run compile:subscriber`       | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway compile:subscriber`         | NA                                                                                  | Compiles the subscriber into a standalone Bun binary.                                      |
| `bun run start:subscriber:binary`  | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway start:subscriber:binary`    | NA                                                                                  | Runs the standalone compiled subscriber binary.                                            |
| `bun run madge:subscriber`         | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway madge:subscriber`           | NA                                                                                  | Shows the dependency tree of the subscriber.                                               |
| `bun run madge:subscriber:json`    | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway madge:subscriber:json`      | NA                                                                                  | Outputs the subscriber dependency tree as JSON.                                            |
| `bun run madge:subscriber:circular`| `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway madge:subscriber:circular`  | NA                                                                                  | Checks the subscriber for circular dependencies.                                           |
| `bun run build:subscriber`         | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway build:subscriber`           | NA                                                                                  | Builds the subscriber into a regular JavaScript file.                                      |
| `bun run start:subscriber`         | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway start:subscriber`           | `bun --cwd ~/dev/vs-code/jobs-api-gateway dist/azure-sync-job-subscriber.js`        | Runs the compiled subscriber JavaScript file.                                              |

### Docker Operations

These scripts target the Compose configuration located at `docker/docker-compose.yml`.

## Docker Commands

## Docker Commands

| Bun Command                        | WD                                 | Bun Command with CWD                                                        | With Absolute Path                                                                                                                                                    | Description                                                                 |
|------------------------------------|------------------------------------|-----------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------|
| `bun run docker:build`             | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:build`                 | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml build`                                                                                    | Builds all services defined in the Docker Compose file.                     |
| `bun run docker:build:subscriber`  | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:build:subscriber`      | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml build jobsdata-subscriber`                                                                | Builds only the `jobsdata-subscriber` image.                                |
| `bun run docker:up`                | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:up`                    | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml up -d`                                                                                    | Starts all services in detached mode.                                       |
| `bun run docker:up:subscriber`     | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:up:subscriber`         | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml up -d jobsdata-subscriber`                                                                | Starts only the `jobsdata-subscriber` container in detached mode.           |
| `bun run docker:stop`              | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:stop`                  | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml stop`                                                                                     | Stops all containers and networks.                              |
| `bun run docker:stop:subscriber`   | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:stop:subscriber`       | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml stop jobsdata-subscriber`                                                                 | Stops only the `jobsdata-subscriber` container.                             |
| `bun run docker:down`              | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:down`                  | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml down`                                                                                     | Stops and removes all containers and networks.                              |
| `bun run docker:down:subscriber`   | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:down:subscriber`       | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml down jobsdata-subscriber`                                                                 | Stops only the `jobsdata-subscriber` container.                             |
| `bun run docker:restart`           | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:restart`               | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml down && docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml up -d` | Restarts all services.                                                      |
| `bun run docker:restart:subscriber`| `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:restart:subscriber`    | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml restart jobsdata-subscriber`                                                              | Restarts only the `jobsdata-subscriber` container.                          |
| `bun run docker:logs`              | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:logs`                  | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml logs -f`                                                                                  | Streams live logs for all services.                                         |
| `bun run docker:logs:subscriber`   | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:logs:subscriber`       | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml logs -f jobsdata-subscriber`                                                              | Streams live logs for the `jobsdata-subscriber` container.                  |
| `bun run docker:rmi`               | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:rmi`                   | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml down --rmi local`                                                                         | Stops containers and removes locally built images.                          |
| `bun run docker:rmi:subscriber`    | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:rmi:subscriber`        | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml rm -sf jobsdata-subscriber && docker image rm -f jobsdata-subscriber:latest`              | Force removes the subscriber container and its image.                       |
| `bun run docker:clean`             | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:clean`                 | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml down --rmi local --volumes --remove-orphans`                                              | Full cleanup: stops containers, removes images, volumes and orphans.        |
| `bun run docker:clean:subscriber`  | `~/dev/vs-code/jobs-api-gateway`   | `bun run --cwd ~/dev/vs-code/jobs-api-gateway docker:clean:subscriber`      | `docker compose -f ~/dev/vs-code/jobs-api-gateway/docker/docker-compose.yml rm -sf jobsdata-subscriber && docker image rm -f jobsdata-subscriber:latest`              | Force removes the subscriber container and its image.                       |
