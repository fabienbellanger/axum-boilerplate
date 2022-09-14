//! Test helper for unit tests

use axum::{Extension, Router};
use axum_boilerplate::{
    layers::{self, MakeRequestUuid, SharedState, State},
    logger, routes,
};
use sqlx::{Connection, MySql, MySqlConnection, MySqlPool};
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

#[derive(Debug)]
pub struct TestApp {
    pub router: Router,
    pub database: Option<TestDatabase>,
}

impl TestApp {
    pub async fn drop_database(&self) {
        if let Some(database) = &self.database {
            database.drop_database().await;
        }
    }
}

pub struct TestAppBuilder {
    router: Router,
    database: Option<TestDatabase>,
}

impl TestAppBuilder {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            database: None,
        }
    }

    pub fn add_web_routes(self) -> Self {
        let router = self.router.nest("/", routes::web());
        Self {
            router,
            database: self.database,
        }
    }

    pub async fn add_api_routes(self) -> Self {
        let db = TestDatabase::new().await;

        let router = self
            .router
            .nest("/api/v1", routes::api())
            .layer(Extension(db.database().await));
        Self {
            router,
            database: Some(db),
        }
    }

    #[allow(unused)]
    pub fn with_logger(self) -> Self {
        logger::init("development", "", "").unwrap();
        let layers = ServiceBuilder::new()
            .set_x_request_id(MakeRequestUuid)
            .layer(layers::logger::LoggerLayer)
            .into_inner();

        Self {
            router: self.router.layer(layers),
            database: self.database,
        }
    }

    pub fn with_state(self) -> Self {
        let state = State {
            jwt_secret_key: String::from("mysecretjwtkey"),
            jwt_lifetime: 1025,
            smtp_host: String::from("127.0.0.1"),
            smtp_port: 1025,
            smtp_timeout: 30,
            forgotten_password_expiration_duration: 1,
            forgotten_password_base_url: String::from("http://localhost"),
            forgotten_password_email_from: String::from("contact@test.com"),
        };

        Self {
            router: self.router.layer(Extension(SharedState::new(state))),
            database: self.database,
        }
    }

    pub fn build(self) -> TestApp {
        TestApp {
            router: self.router,
            database: self.database,
        }
    }
}

//
// Example: https://github.com/davidpdrsn/witter/blob/master/backend/src/tests/test_helpers/test_db.rs
//

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
        let (conn, db_name) = parse_url(&self.url);

        let mut pool = MySqlConnection::connect(conn).await.unwrap();

        let sql = format!(
            r#"
            SELECT
            CONCAT('KILL ', id, ';')
            FROM INFORMATION_SCHEMA.PROCESSLIST
            WHERE `db` = '{}'"#,
            &db_name
        );
        sqlx::query::<MySql>(&sql).execute(&mut pool).await.unwrap();

        let sql = format!(r#"DROP DATABASE `{}`"#, &db_name);
        sqlx::query::<MySql>(&sql).execute(&mut pool).await.unwrap();
    }
}

// TODO: Not Work!
impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Drop the DB Pool
        let _ = self.pool.take();
        println!("DROP TestDatabase");
        // futures::executor::block_on(self.drop_database());
        // let rt = tokio::runtime::Handle::current();
        // rt.block_on(self.drop_database());
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
