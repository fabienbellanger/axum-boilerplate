//! Test helper for unit tests

use axum::{Extension, Router};
use axum_boilerplate::routes;
use sqlx::{Connection, MySql, MySqlConnection, MySqlPool};

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
            .layer(Extension(db.pool.clone()));
        Self {
            router,
            database: Some(db),
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

    pub async fn _database(&self) -> MySqlPool {
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
