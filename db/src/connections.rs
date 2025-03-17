use diesel::{Connection, PgConnection};
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::{env, sync::Arc};
use tokio::sync::Mutex;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[derive(Clone)]
pub struct DatabaseConnections {
    pub postgres: Pool<AsyncPgConnection>,
    pub redis: Arc<Mutex<redis::Connection>>,
}

impl DatabaseConnections {
    fn run_migrations(db_url: &str) -> Result<(), std::io::Error> {
        let mut conn = PgConnection::establish(db_url).expect("Can't connect to database");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Can't run migrations");
        Ok(())
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
        let mut db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let postgres_user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
        let postgres_password =
            env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");

        db_url = db_url.replace("$(POSTGRES_USER)", &postgres_user);
        db_url = db_url.replace("$(POSTGRES_PASSWORD)", &postgres_password);

        let db_url_pool = db_url.clone();
        tokio::task::spawn_blocking(move || Self::run_migrations(&db_url.clone())).await??;
        let pool = Self::postgres_pool(db_url_pool);

        Ok(pool)
    }

    pub fn init_redis() -> redis::Connection {
        let redis_host_name: String =
            env::var("REDIS_HOSTNAME").expect("missing environment variable REDIS_HOSTNAME");
        let redis_password = env::var("REDIS_PASSWORD").unwrap_or_default();

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
            postgres: Self::init_postgres().await?,
            redis: Arc::new(Mutex::new(Self::init_redis())),
        })
    }
}
