use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub environment: String,
    pub database_url: String,
    pub database_max_connections: u32,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_seconds: i64,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()?;

        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/inventory_db".to_string());

        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".to_string())
            .parse::<u32>()?;

        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "super_secret_jwt_key_minimum_32_characters_long_12345".to_string());

        let jwt_expiration_seconds = env::var("JWT_EXPIRATION_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<i64>()?;

        Ok(Self {
            host,
            port,
            environment,
            database_url,
            database_max_connections,
            redis_url,
            jwt_secret,
            jwt_expiration_seconds,
        })
    }
}
