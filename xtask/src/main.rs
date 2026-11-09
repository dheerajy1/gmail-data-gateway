mod extract_mails;
mod gen_token;
mod quiet;

use std::env;
use std::path::PathBuf;
use std::process::{Command, exit};

use jobs_api_gateway::config::env::Env;

use quiet::Show;

fn main() {
    let task = env::args().nth(1).unwrap_or_else(|| {
        eprintln!(
            "Usage: cargo <css|dev|start|preview|build-prod|dev-subscriber|start-subscriber|build-subscriber|gen-token|extract-mails|verify|docker-*>"
        );
        exit(1);
    });

    match task.as_str() {
        "css" => run_css(),
        "dev" => run_server("development", false),
        "start" => run_server("production", true),
        "preview" => run_server("preview", true),
        "build" => run_build(),
        "dev-subscriber" => run_subscriber("development", false),
        "start-subscriber" => run_subscriber("production", true),
        "build-subscriber" => run_build_subscriber(),
        "gen-token" => gen_token::run(load_env("production").google_appid),
        "extract-mails" => {
            load_env("development");
            extract_mails::run(&env::args().skip(2).collect::<Vec<_>>());
        }
        "verify" => run_verify(),
        "docker" => run_docker(env::args().nth(2)),
        _ => {
            eprintln!("Unknown task: {task}");
            exit(1);
        }
    }
}

// Loads the env file for the profile, then validates it against the schema
fn load_env(profile: &str) -> Env {
    let env_file = format!(".env.{profile}");

    dotenvy::from_filename(&env_file)
        .unwrap_or_else(|_| panic!("Failed to load environment file: {env_file}"));

    Env::load().unwrap_or_else(|error| {
        eprintln!("[env] Validation failed for {env_file}: {error}");
        exit(1);
    })
}

// Compiles src/tailwind.css into public/global.css with the standalone Tailwind CLI
fn run_css() {
    let status = Command::new("tailwindcss")
        .args(["-i", "./src/tailwind.css", "-o", "./public/global.css"])
        .status()
        .expect("Failed to execute tailwindcss, is it installed and on PATH?");

    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
}

// Compiles Tailwind CSS, then runs the server on the port from the env file
fn run_server(profile: &str, release: bool) {
    let env = load_env(profile);

    run_css();

    println!(
        "-> Loaded .env.{profile} (Port: {}, Mode: {profile})",
        env.app_port
    );

    build_quiet(&package_args("build", "jobs-api-gateway", release));

    let mut cmd = Command::new("cargo");
    cmd.args(package_args("run", "jobs-api-gateway", release));
    cmd.arg("--quiet");

    // Passes the validated port to the server process
    cmd.env("VERCEL_DEV_PORT", env.app_port.to_string());

    let status = cmd
        .status()
        .expect("Failed to execute cargo run for jobs-api-gateway");

    exit(status.code().unwrap_or(1));
}

// Validates the production env, compiles Tailwind CSS, then builds in release mode
fn run_build() {
    load_env("production");

    println!("-> Validated .env.production");

    run_css();

    build_quiet(&package_args("build", "jobs-api-gateway", true));
}

// Loads and validates the env file for the profile, then runs the subscriber
fn run_subscriber(profile: &str, release: bool) {
    load_env(profile);

    println!(
        "-> Loaded .env.{profile} (Subscriber, Mode: {})",
        if release { "production" } else { "development" }
    );

    build_quiet(&package_args("build", "subscriber", release));

    let mut cmd = Command::new("cargo");
    cmd.args(package_args("run", "subscriber", release));
    cmd.arg("--quiet");

    let status = cmd
        .status()
        .expect("Failed to execute cargo run for subscriber");

    exit(status.code().unwrap_or(1));
}

// Validates the production env, then builds the subscriber binary in release mode
fn run_build_subscriber() {
    load_env("production");

    build_quiet(&package_args("build", "subscriber", true));

    println!("-> Binary: target/release/subscriber");
}

// Path of the compose file, found from the xtask folder so it works from any directory
fn compose_file() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("docker")
        .join("docker-compose.yml")
}

// Runs one docker compose action on docker/docker-compose.yml
fn run_docker(action: Option<String>) {
    let action = action.unwrap_or_else(|| {
        eprintln!(
            "Usage: cargo docker-<build|up|down|start|stop|restart|logs|logs-clear|rmi|clean>"
        );
        exit(1);
    });

    let args: &[&str] = match action.as_str() {
        "build" => &["build"],
        "up" => &["up", "-d"],
        "down" => &["down"],
        "start" => &["start"],
        "stop" => &["stop"],
        "restart" => &["restart"],
        "logs" => &["logs", "-f"],
        "rmi" => &["down", "--rmi", "local"],
        "clean" => &["down", "--rmi", "local", "--volumes"],
        "logs-clear" => {
            clear_docker_logs();
            return;
        }
        _ => {
            eprintln!("Unknown docker action: {action}");
            exit(1);
        }
    };

    let status = Command::new("docker")
        .arg("compose")
        .arg("-f")
        .arg(compose_file())
        .args(args)
        .status()
        .expect("Failed to execute docker compose, is Docker installed?");

    exit(status.code().unwrap_or(1));
}

// Empties the log file of every container in the compose project, asks for the sudo password
fn clear_docker_logs() {
    let script = format!(
        "for c in $(docker compose -f '{}' ps -q); do sudo truncate -s 0 $(docker inspect --format='{{{{.LogPath}}}}' $c); done",
        compose_file().display()
    );

    let status = Command::new("sh")
        .args(["-c", &script])
        .status()
        .expect("Failed to clear the docker logs");

    exit(status.code().unwrap_or(1));
}

// Builds the argument list of a cargo command for one package, in release mode when asked
fn package_args(command: &str, package: &str, release: bool) -> Vec<String> {
    let mut args = vec![
        command.to_string(),
        "--package".to_string(),
        package.to_string(),
    ];

    if release {
        args.push("--release".to_string());
    }

    args
}

// Runs one cargo command with the one-line progress, exits with 1 when it fails
fn run_step(args: &[String], show: Show) {
    let mut cmd = Command::new("cargo");
    cmd.args(args);

    match quiet::run(&cmd, show) {
        Ok(true) => {}
        Ok(false) => exit(1),
        Err(error) => {
            eprintln!("Failed to run cargo: {error}");
            exit(1);
        }
    }
}

// Builds with one overwritten progress line instead of a Compiling line per crate
fn build_quiet(args: &[String]) {
    run_step(args, Show::All);
}

// Runs the verify sequence of docs/BUILD.md section 3: fmt, check, clippy and test of the whole workspace
fn run_verify() {
    let steps: [(&str, Vec<&str>, Show); 4] = [
        ("cargo fmt", vec!["fmt"], Show::All),
        (
            "cargo check",
            vec!["check", "--workspace", "--all-targets"],
            Show::All,
        ),
        (
            "cargo clippy",
            vec![
                "clippy",
                "--workspace",
                "--all-targets",
                "--locked",
                "--",
                "-D",
                "warnings",
            ],
            Show::All,
        ),
        ("cargo test", vec!["test", "--workspace"], Show::TestSummary),
    ];

    for (label, args, show) in steps {
        println!("-> {label}");

        let args: Vec<String> = args.into_iter().map(str::to_string).collect();

        run_step(&args, show);
    }

    println!("-> verify passed");
}
