use std::{sync::LazyLock, time::Duration};

use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use tiberius::{AuthMethod, Config, EncryptionLevel};

use crate::config::env::env;

// Pool of Azure SQL connections
pub type AzurePool = Pool<ConnectionManager>;

// Builds a pool for one Azure SQL user, no connection is opened until the first query
fn build_pool(user: &str, password: &str) -> AzurePool {
    let env = env();

    let mut config = Config::new();

    config.host(&env.azure_db_server);
    config.database(&env.azure_db_database);
    config.authentication(AuthMethod::sql_server(user, password));
    config.encryption(EncryptionLevel::Required);
    config.application_name("jobs-api-gateway");

    Pool::builder()
        .max_size(5)
        .min_idle(Some(0))
        .idle_timeout(Some(Duration::from_secs(120)))
        .connection_timeout(Duration::from_secs(90))
        .build_unchecked(ConnectionManager::new(config))
}

// Pool for the public user, used by the logins and the health check
static PUBLIC_POOL: LazyLock<AzurePool> = LazyLock::new(|| {
    let env = env();

    build_pool(&env.azure_db_user_public, &env.azure_db_password_public)
});

// Returns the public user pool, created on first use
pub fn public_pool() -> &'static AzurePool {
    &PUBLIC_POOL
}
