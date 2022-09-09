//! Test helper for unit tests

// Exemple: https://github.com/letsgetrusty/builder_pattern/blob/master/src/main.rs

use axum::{Extension, Router};
use axum_boilerplate::{databases, routes};
use sqlx::MySqlPool;

#[derive(Debug)]
pub struct TestApp {
    pub router: Router,
}

pub struct TestAppBuilder {
    router: Router,
}

impl TestAppBuilder {
    pub fn new() -> Self {
        Self { router: Router::new() }
    }

    pub fn add_web_routes(self) -> Self {
        let router = self.router.nest("/", routes::web());
        Self { router }
    }

    pub async fn _add_api_routes(self) -> Self {
        let pool = databases::init_test("mysql://root:root@127.0.0.1:3306/axum_test")
            .await
            .unwrap();
        let router = self.router.nest("/api/v1", routes::api()).layer(Extension(pool));
        Self { router }
    }

    pub fn build(self) -> TestApp {
        TestApp { router: self.router }
    }
}

//
// Example: https://github.com/davidpdrsn/witter/blob/master/backend/src/tests/test_helpers/test_db.rs
//

#[derive(Debug)]
pub struct TestDatabase {
    _url: String,
    pool: Option<MySqlPool>,
}

impl TestDatabase {
    fn _parse_url(&self) -> (&str, &str) {
        let separator_pos = self._url.rfind("/").unwrap();
        let conn = &self._url[..=separator_pos];
        let name = &self._url[separator_pos + 1..];

        (conn, name)
    }

    fn get_url() -> String {
        let e = dotenv::dotenv().ok();
        dbg!(e);

        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL missing from environment");

        format!("{}_{}", url, "test")
    }

    pub fn new() -> Self {
        Self {
            _url: Self::get_url(),
            pool: None,
        }
    }

    pub async fn _database(&self) -> MySqlPool {
        dbg!(self._url.clone());
        self.pool.clone().unwrap()
    }

    fn _run_migrations() {}

    async fn _create(&self) {
        self._parse_url();
    }

    async fn drop() {
        // SELECT
        // CONCAT('KILL ', id, ';')
        // FROM INFORMATION_SCHEMA.PROCESSLIST
        // WHERE `User` = 'some_user'
        // AND `Host` = '192.168.1.1'
        // AND `db` = 'my_db';
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Drop the DB Pool
        let _ = self.pool.take();
        futures::executor::block_on(Self::drop());
    }
}
