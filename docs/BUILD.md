# BUILD.md: jobs-api-gateway-rust

Owns **how to build, verify, run, deliver and commit**. What to build is in `SPEC.md` and `FEATURES.md`, the order is in `PLAN.md`, reasons are in `DECISIONS.md`.

Last updated: 2026-11-06

## How to read this file

- Every command has a heading that states exactly what it does, placed right before it.
- `FACT` was read from files or from your terminal output. `UNKNOWN` was not determined. Nothing in this file has been run by me.
- Where a command or tool is not known to work yet, the section says so.

## 1. Environment

| Item | Value | Source |
| --- | --- | --- |
| Operating system | Linux, Oracle Cloud ARM server | D26 |
| CPU architecture | `aarch64` | your `uname -m` output |
| Shell | `zsh` (bash underneath) | D26 |
| Docker | Installed, runs the Kafka broker (`apache/kafka:4.1.0`, port 9092) | your `docker ps` output |
| Rust edition | 2024 (needs Rust 1.85 or newer) | `Cargo.toml` of `recordlog-api-rust` and `kafka-api-gateway-rust` |
| Pinned toolchain file | None (no `rust-toolchain.toml`, `rustfmt.toml` or `clippy.toml` in the reference repos) | file listing |
| Git branch in use | `feat/migrate-to-axum` | `git branch` of the jobs repo |

Rust edition 2024 is my reading of the manifests; the installed `rustc` version on your server is `UNKNOWN`.

### Check the installed Rust compiler version

```
rustc --version
```

### Check the installed Cargo version

```
cargo --version
```

### Check that the standalone Tailwind CSS binary is on the path

```
which tailwindcss
```

`cargo css` calls `tailwindcss` (`FACT`: `recordlog-api-rust/xtask/src/main.rs`). Where you installed it from is `UNKNOWN`.

## 2. Cargo aliases

Defined in `.cargo/config.toml` and backed by `xtask`. The recordlog set is the model (`FACT`). The jobs repo gets the core four in P1 and the rest in P7 and P10 (`PLAN.md`).

| Alias | What it does (`FACT` in recordlog) | Added in |
| --- | --- | --- |
| `cargo css` | Builds the Tailwind CSS file | P1 |
| `cargo dev` | Runs the API with the development environment | P1 |
| `cargo start` | Runs the API with the production environment | P1 |
| `cargo preview` | Runs the API in release mode with `.env.preview` (the values of Vercel preview deployments) | P1 |
| `cargo build-prod` | Release build of the API package | P1 |
| `cargo dev-subscriber` | Runs the subscriber with the development environment | P7 |
| `cargo start-subscriber` | Runs the subscriber with the production environment | P7 |
| `cargo build-subscriber` | Release build of the subscriber package | P7 |
| `cargo docker-build`, `docker-up`, `docker-down`, `docker-start`, `docker-stop`, `docker-restart`, `docker-logs`, `docker-logs-clear`, `docker-rmi`, `docker-clean` | Docker Compose helpers | P10 |
| `cargo gen-token` | Google consent flow that stores the refresh token (D5, D14) | P9 |

## 3. Verify (your command sequence, D24)

Run these in order from the repository root. `cargo fmt` rewrites files, so a run can leave edits that belong in the same commit.

### Format, check, lint and test the whole workspace

```
cargo fmt && cargo check --workspace --all-targets 2>&1 | grep -Ev '^( *Compiling| *Checking [^ ]+ v[^ ]+$)' && cargo clippy --workspace --all-targets --locked -- -D warnings && cargo test --workspace 2>&1 | grep -P '^(?!.*SIGTERM).*(Finished|Running|test result|FAILED|panicked|Doc-tests|^error(\[E\d+\])?:)'
```

### The same sequence in one command

```
cargo verify
```

It runs `cargo fmt`, `cargo check`, `cargo clippy` and `cargo test` in that order and stops at the first failure. The compiler progress shows as one line that is overwritten, and warnings and errors still print.

### Build the production binary with filtered output

```
cargo build-prod 2>&1 | grep -Ev '^( *Compiling| *Checking [^ ]+ v[^ ]+$)'
```

### Start the application for a manual look

```
cargo start
```

Notes:
- `--locked` needs `Cargo.lock` to be committed.
- The filters hide compiler progress lines. A failure still prints the `error` lines.
- The `SIGTERM` exclusion in the test filter points to a test that stops a server. The reason is `UNKNOWN`.
- `cargo start` runs until you stop it. A phase is verified only after you have looked at it and stopped it.

### Send errors back to Claude

When a command fails, paste the full output of the first failing command, from the first `error` line to the end. Do not paste the secret values of `.env*` files.

## 4. Environment files

| File | Tracked in git | Purpose |
| --- | --- | --- |
| `.env.example` | Yes | Names of all variables, no real values |
| `.env.development` | No (ignored) | Local development values |
| `.env.preview` | No (ignored) | Preview values |
| `.env.production` | No (ignored) | Production values |

`FACT` from `.gitignore` and `git ls-files`: `.env.*` is ignored except `.env.example`.

- The variable names and rules are in `SPEC.md` section 10.
- The variable that selects the environment is `APP_ENV` (D13). The `NODE_ENV` to `APP_ENV` rename is done in `.env.development` and `.env.production`.
- Env files are loaded only by the `xtask` commands (`cargo dev`, `cargo start`, `cargo preview`, `cargo build-prod`), each of which prints which file it used. A plain `cargo run` loads no file and stops with `APP_NAME is required`.
- Real values must never be committed, pasted into chats, or written into docs.

## 5. Running the pieces

| What | Command | Phase available |
| --- | --- | --- |
| API, development | `cargo dev` | P1 |
| API, production mode | `cargo start` | P1 |
| API, preview values | `cargo preview` | P1 |
| Subscriber, development | `cargo dev-subscriber` | P7 |
| Subscriber, production mode | `cargo start-subscriber` | P7 |
| Subscriber in Docker | `cargo docker-up` | P10 |

Vercel deployment (D1): `vercel.json` already sets `framework: rust` (`FACT`). Whether you deploy through Git integration or the Vercel CLI is `UNKNOWN`; it is settled in P1. The function time limit for the Rust runtime is not known and is recorded in P1 (D2).

## 6. Docker (subscriber, D1, D27)

- The three Docker files are still Bun builds and are rewritten in P10 (`FACT`: `docker/Dockerfile`, `docker/Dockerfile.subscriber`, `docker/docker-compose.yml`).
- Images are built on the `aarch64` host without a platform override, so they are `linux/arm64` (`ASSUMPTION`, D27).
- The compose file uses `env_file: ../.env.production`, `network_mode: bridge` and `host.docker.internal`, so the container reaches services on its Docker host (`FACT`).
- The API service stays commented out because the API runs on Vercel.

## 7. Release (D25)

- `.github/workflows/release.yml` in the jobs repo is replaced in P10 by the one from `kafka-api-gateway-rust` (D25). The kept file still uses Bun and cannot work without a `package.json`.
- The kafka version uses `cycjimmy/semantic-release-action@v6.0.0` with the changelog, git and cargo plugins. `.releaserc.json` is identical in both repos.
- You have already applied the same workflow to `recordlog-api-rust` (your commit, D29). Testing it in GitHub Actions containers is still pending, so it is `UNVERIFIED`.
- `.github/workflows/test.yml` calls a reusable workflow `test-rust.yml` in your `.github` repository. Its content is not available here (`UNKNOWN`).

## 8. How a build phase runs

Each phase follows `PLAN.md`. The steps:

1. **Start.** You open a new chat and give the project files. The `.agents` skills carry the state between chats: `/restore`, verify, continue, `/review`, `/save` (`FACT`: `.agents/README.md`). The current `.agents/state/checkpoint.md` holds sample content, not real state, so the first `/save` replaces it.
2. **Read.** Claude reads the phase entry in `PLAN.md`, the features in `FEATURES.md` and the reference files, and re-reads any file it will change.
3. **Build.** Claude writes the code for the phase.
4. **Check.** Claude tries to install the Rust toolchain and run `cargo check` and `cargo test` (D15). If that cannot be done it says so, and nothing is described as compiled or tested.
5. **Deliver.** Claude gives one full-repo tarball excluding `target` and `.git`, including all `.env*` files (D16).
6. **Verify.** You extract the tarball over your working copy and run the verify in section 3.
7. **Report.** You send errors back, or say the verify passed.
8. **Commit.** Only after your verify passes, using section 9.
9. **Record.** New decisions go into `DECISIONS.md`; touched features are updated in `FEATURES.md`.

### Extract a delivered tarball over your working copy

```
tar -xzf <tarball-name>.tar.gz -C <path-to-your-working-copy>
```

The tarball has no `.git`, so your own git history stays untouched. It contains real `.env*` files; do not share it.

### Create the delivery tarball (Claude runs this)

Each phase is delivered as one full-repo tarball, excluding `.git`, `target` (at every level), `Cargo.lock`, `public/global.css`, and `structure.txt`.

```bash
tar --exclude='.git' --exclude='target' --exclude='Cargo.lock' --exclude='public/global.css' --exclude='structure.txt' -czf <phase-name>.tar.gz -C <repo-parent> <repo-name>
```

This pattern is a template (`PROPOSED`) (D16). The exact exclusions are re-checked each phase so that `target` is excluded at every level and `.env*` files stay in.

## 9. Commit rules

Branch names must match `^(feat|fix|refactor|chore|docs)/.+$` (`FACT`: `.github/workflows/pr-checks.yml`). The repository currently uses `feat/migrate-to-axum`.

Commit message (your rule):
- First line: `feat`, `fix`, `chore` or `refactor`, with a suitable scope in brackets, then a short summary.
- Then the description as numbered headings, each with `-` points.
- The head line and the description are given together in one code block so you can copy them.
- The number of headings and points depends on the number of changes in the commit:

| Changes in the commit | Headings | Points under each heading |
| --- | --- | --- |
| Under 20 | 4 | 2 |
| 20 to 30 | 8 | 3 |
| Over 30 | 12 | 4 |

Template for the under-20 tier:

```
feat(scope): short summary

1. First heading
- point one
- point two

2. Second heading
- point one
- point two

3. Third heading
- point one
- point two

4. Fourth heading
- point one
- point two
```

Rules:
- Commit only after the phase was delivered and your verify passed.
- The heading tiers follow the actual number of changes (my reading of your rule, D17).
- Pull request titles must pass the semantic title check in `pr-checks.yml` (`FACT`).

## 10. Unknowns that affect this file

1. Installed `rustc` and `cargo` versions on your server.
2. How `tailwindcss` is installed.
3. How you deploy to Vercel.
4. Whether Claude's environment can install the Rust toolchain (D15).
5. Content of the reusable `test-rust.yml` workflow.
6. Whether the kafka-style `release.yml` (now also in `recordlog-api-rust`) works in GitHub Actions containers.