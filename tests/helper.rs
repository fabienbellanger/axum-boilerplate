//! Test helper for unit tests

use axum::{Extension, Router};
use axum_boilerplate::{
    layers::{self, ConfigState, MakeRequestUuid, SharedState, State},
    logger, routes,
};
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::{mysql::MySqlPoolOptions, Connection, MySql, MySqlConnection, MySqlPool};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

//
// Examples:
// - https://github.com/davidpdrsn/witter/blob/master/backend/src/tests/test_helpers/test_db.rs
// - https://github.com/tokio-rs/axum/blob/main/examples/testing/src/main.rs
// - https://github.com/wolf4ood/realworld-axum/blob/main/src/web/src/app.rs

pub struct TestApp {
    pub router: Router,
    pub database: TestDatabase,
}

impl TestApp {
    pub fn database(&self) -> &TestDatabase {
        &self.database
    }

    pub async fn drop_database(&self) {
        self.database.drop_database().await;
    }
}

pub struct TestAppBuilder {
    router: Router,
    database: TestDatabase,
}

impl TestAppBuilder {
    pub async fn new() -> Self {
        let state = Self::get_state();
        let db = TestDatabase::new().await;

        let router = Router::new().nest("/api/v1", routes::api(state.clone()));
        let mut router = router.nest("/", routes::web());
        router = router.layer(Extension(db.database().await));

        let router = router.with_state(state);

        Self { router, database: db }
    }

    #[allow(unused)]
    pub fn with_logger(self) -> Self {
        logger::init("test", "", "").unwrap();
        let layers = ServiceBuilder::new()
            .set_x_request_id(MakeRequestUuid)
            .layer(layers::logger::LoggerLayer)
            .into_inner();

        Self {
            router: self.router.layer(layers),
            database: self.database,
        }
    }

    fn get_state() -> SharedState {
        let jwt_secret_key = "mysecretjwtkey";
        let state = State {
            config: ConfigState {
                jwt_encoding_key: EncodingKey::from_secret(jwt_secret_key.as_bytes()),
                jwt_decoding_key: DecodingKey::from_secret(jwt_secret_key.as_bytes()),
                jwt_lifetime: 1025,
                smtp_host: String::from("127.0.0.1"),
                smtp_port: 1025,
                smtp_timeout: 30,
                forgotten_password_expiration_duration: 1,
                forgotten_password_base_url: String::from("http://localhost"),
                forgotten_password_email_from: String::from("contact@test.com"),
            },
        };

        SharedState::new(state)
    }

    pub fn build(self) -> TestApp {
        TestApp {
            router: self.router,
            database: self.database,
        }
    }
}

#[derive(Debug)]
pub struct TestDatabase {
    url: String,
    pool: Option<MySqlPool>,
}

/// Sets up a new DB for running tests with.
impl TestDatabase {
    pub async fn new() -> Self {
        let db_url = url();

        create_database(&db_url).await;
        run_migrations(&db_url).await;

        let pool = MySqlPool::connect(&db_url).await.unwrap();

        Self {
            url: db_url,
            pool: Some(pool),
        }
    }

    pub async fn database(&self) -> MySqlPool {
        self.pool.clone().unwrap()
    }

    /// Drop database after the test
    pub async fn drop_database(&self) {
        let (_conn, db_name) = parse_url(&self.url);

        let pool = MySqlPoolOptions::new()
            .max_connections(1)
            .min_connections(1)
            .max_lifetime(Some(Duration::from_secs(5)))
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(5))
            .test_before_acquire(false)
            .connect(&self.url)
            .await
            .expect("error during MySQL pool creation");

        let sql = format!(
            r#"
            SELECT
            CONCAT('KILL ', id, ';')
            FROM INFORMATION_SCHEMA.PROCESSLIST
            WHERE `db` = '{}'"#,
            &db_name
        );
        sqlx::query::<MySql>(&sql)
            .execute(&pool)
            .await
            .expect("error during killing database processes");

        let sql = format!(r#"DROP DATABASE `{}`"#, &db_name);
        sqlx::query::<MySql>(&sql)
            .execute(&pool)
            .await
            .expect("error when dropping database");
    }
}

// TODO: Not Work!
impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Drop the DB Pool
        // let _ = self.pool.take();
        // println!("DROP TestDatabase");
    }
}

/// Parse database URL and return the database name in a separate variable
fn parse_url(url: &str) -> (&str, &str) {
    let separator_pos = url.rfind("/").unwrap();
    let conn = &url[..=separator_pos];
    let name = &url[separator_pos + 1..];

    (conn, name)
}

/// Generate url with a random database name
fn url() -> String {
    use rand::distributions::{Alphanumeric, DistString};

    dotenv::dotenv().ok();

    // Set up the database per tests
    let suffix: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL missing from environment.");

    format!("{}_{}", db_url, suffix)
}

/// Create the test database
async fn create_database(url: &str) {
    let (conn, db_name) = parse_url(url);

    let mut pool = MySqlConnection::connect(conn).await.unwrap();

    let sql = format!(r#"CREATE DATABASE `{}`"#, &db_name);
    sqlx::query::<MySql>(&sql).execute(&mut pool).await.unwrap();
}

/// Launch migrations
async fn run_migrations(url: &str) {
    let (conn, db_name) = parse_url(url);
    let mut pool = MySqlConnection::connect(&format!("{}/{}", conn, db_name))
        .await
        .unwrap();

    // Run the migrations
    sqlx::migrate!("./migrations")
        .run(&mut pool)
        .await
        .expect("Failed to migrate the database");
}
