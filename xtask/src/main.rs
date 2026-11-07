use std::env;
use std::process::{Command, exit};

use jobs_api_gateway::config::env::Env;

fn main() {
    let task = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: cargo <css|dev|start|preview|build-prod>");
        exit(1);
    });

    match task.as_str() {
        "css" => run_css(),
        "dev" => run_server("development", false),
        "start" => run_server("production", true),
        "preview" => run_server("preview", true),
        "build" => run_build(),
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

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--package", "jobs-api-gateway"]);

    if release {
        cmd.arg("--release");
    }

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

    let status = Command::new("cargo")
        .args(["build", "--release", "--package", "jobs-api-gateway"])
        .status()
        .expect("Failed to execute cargo build for jobs-api-gateway");

    exit(status.code().unwrap_or(1));
}
