use diesel::{Connection, PgConnection};
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::{env, sync::Arc, thread, time::Duration};
use tokio::sync::Mutex;
use tracing::{error, info};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[derive(Clone)]
pub struct DatabaseConnections {
    pub postgres: Option<Pool<AsyncPgConnection>>,
    pub redis: Arc<Mutex<redis::Connection>>,
}

impl DatabaseConnections {
    fn run_migrations(db_url: &str) -> Result<(), std::io::Error> {
        for attempt in 1..=MAX_RETRIES {
            match PgConnection::establish(db_url) {
                Ok(mut conn) => {
                    if let Err(e) = conn.run_pending_migrations(MIGRATIONS) {
                        error!(
                            "Migration failed on attempt {}/{}: {}",
                            attempt, MAX_RETRIES, e
                        );
                    } else {
                        info!("Ran migration successfully");
                        return Ok(());
                    }
                }
                Err(e) => error!(
                    "Database connection failed on attempt {}/{}: {}",
                    attempt, MAX_RETRIES, e
                ),
            }

            thread::sleep(Duration::from_secs(1));
        }

        error!("Migration failed after {} attempts", MAX_RETRIES);
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Migration failed after multiple attempts",
        ))
    }

    pub fn postgres_pool(db_url: String) -> Pool<AsyncPgConnection> {
        let config = AsyncDieselConnectionManager::new(db_url);
        let max_pool_size = env::var("MAX_POOL_SIZE")
            .unwrap_or("8".to_string())
            .parse()
            .unwrap();
        Pool::builder(config)
            .max_size(max_pool_size)
            .build()
            .expect("Failed to create pool")
    }

    async fn init_postgres() -> Result<Pool<AsyncPgConnection>, std::io::Error> {
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db_url_pool = db_url.clone();
        tokio::task::spawn_blocking(move || Self::run_migrations(&db_url.clone())).await??;
        let pool = Self::postgres_pool(db_url_pool);

        Ok(pool)
    }

    pub fn init_redis() -> redis::Connection {
        let redis_host_name: String =
            env::var("REDIS_HOSTNAME").unwrap_or("localhost:6379".to_string());
        let redis_password = env::var("REDIS_PASSWORD").unwrap_or("redis".to_string());

        let uri_scheme = match env::var("IS_TLS") {
            Ok(_) => "rediss",
            Err(_) => "redis",
        };

        let redis_conn_url = format!("{}://:{}@{}", uri_scheme, redis_password, redis_host_name);

        redis::Client::open(redis_conn_url)
            .expect("Invalid connection URL")
            .get_connection()
            .expect("failed to connect to Redis")
    }

    pub async fn init() -> Result<Self, std::io::Error> {
        Ok(Self {
            postgres: None, //Some(Self::init_postgres().await?),
            redis: Arc::new(Mutex::new(Self::init_redis())),
        })
    }
}

const MAX_RETRIES: u32 = 100;
