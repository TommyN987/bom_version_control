use std::net::TcpListener;

use bom_version_control::{configuration::get_config, db::DbPool};
use diesel::{
    r2d2::{self, ConnectionManager, CustomizeConnection},
    Connection, PgConnection,
};
use reqwest::Client;
use secrecy::ExposeSecret;
use uuid::Uuid;

pub struct TestApp {
    pub addr: String,
    pub pool: DbPool,
}

#[derive(Debug)]
pub struct AlwaysTestingTransaction {}

impl<C: Connection, E> CustomizeConnection<C, E> for AlwaysTestingTransaction {
    fn on_acquire(&self, conn: &mut C) -> Result<(), E> {
        conn.begin_test_transaction().unwrap();
        Ok(())
    }
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port.");
    let port = listener.local_addr().unwrap().port();
    let addr = format!("http://127.0.0.1:{}", port);

    let mut config = get_config().expect("Failed to read configuration.");
    config.db.db_name = Uuid::new_v4().to_string();

    let always_testing = AlwaysTestingTransaction {};

    let manager =
        ConnectionManager::<PgConnection>::new(config.db.conn_string_without_db().expose_secret());

    let pool = r2d2::Builder::new()
        .max_size(1)
        .connection_customizer(Box::new(always_testing))
        .build(manager)
        .expect("Failed to create db pool");

    let server =
        bom_version_control::startup::run(listener, pool.clone()).expect("Failed to bind address");

    let _ = tokio::spawn(server);

    TestApp { addr, pool }
}

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;
    let client = Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &app.addr))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
