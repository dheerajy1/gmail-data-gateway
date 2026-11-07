use std::sync::LazyLock;

use validator::Validate;

use crate::schemas::env::EnvSchema;

// Alias so the rest of the app refers to the schema as Env
pub type Env = EnvSchema;

// Global env, loaded and validated on first use, the process exits if it is invalid
static ENV: LazyLock<Env> = LazyLock::new(|| {
    Env::load().unwrap_or_else(|error| {
        eprintln!("[env] Validation failed: {error}");
        std::process::exit(1);
    })
});

// Returns the global env
pub fn env() -> &'static Env {
    &ENV
}

// Forces the env to load now, so a bad env stops the server at boot
pub fn init() {
    LazyLock::force(&ENV);
}

// Prefix of the text envy gives for a variable that is not set
const MISSING_PREFIX: &str = "missing value for field ";

// Text envy puts before the variable name when a value cannot be parsed
const PROVIDED_BY: &str = " provided by ";

// Name of a variable as written in the env file, envy gives it in lowercase or in the original case
fn env_name(field: &str) -> String {
    match field.to_lowercase().as_str() {
        "google_appid" => "Google_APPID".to_string(),
        _ => field.to_uppercase(),
    }
}

// Turns an envy error into a message that names the variable as written in the env file
fn describe_envy_error(error: &str) -> String {
    if let Some(field) = error.strip_prefix(MISSING_PREFIX) {
        return format!("{} is required", env_name(field));
    }

    // APP_ENV is the only enum, envy gives no variable name for a wrong variant
    if error.starts_with("unknown variant") {
        return "APP_ENV must be development, preview or production".to_string();
    }

    // The parse error text can hold the value, so only the variable name is kept
    if let Some((_, field)) = error.rsplit_once(PROVIDED_BY) {
        return format!("{} has an invalid value", env_name(field));
    }

    error.to_string()
}

impl Env {
    // Reads all variables from the process env into the schema, then validates the values
    pub fn load() -> Result<Self, String> {
        Self::from_pairs(std::env::vars())
    }

    // Builds the schema from name and value pairs, used by load and by the tests
    pub fn from_pairs<I>(pairs: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = (String, String)>,
    {
        let env = envy::from_iter::<_, Self>(pairs)
            .map_err(|error| describe_envy_error(&error.to_string()))?;

        env.validate().map_err(|error| error.to_string())?;

        Ok(env)
    }
}
