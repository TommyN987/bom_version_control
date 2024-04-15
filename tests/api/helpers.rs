use std::{error::Error, net::TcpListener};

use bom_version_control::{configuration::get_config, db::DbPool, startup::run};
use diesel::{
    pg::Pg,
    r2d2::{self, ConnectionManager, CustomizeConnection},
    Connection, PgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use secrecy::ExposeSecret;
use uuid::Uuid;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

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

pub async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");

    let port = listener.local_addr().unwrap().port();

    let addr = format!("http://127.0.0.1:{}", port);

    let config = {
        let mut c = get_config().expect("Failed to read configuration");
        c.db.db_name = Uuid::new_v4().to_string();
        c.app.port = port;
        c
    };

    let pool = create_testing_pool(config.db.conn_string_without_db().expose_secret());

    run_migrations(&mut pool.get().expect("Failed to get connection to db"))
        .expect("Failed to run migrations");

    let server = run(listener, pool.clone()).expect("Failed to bind address");

    let _ = tokio::spawn(server);

    TestApp { addr, pool }
}

fn run_migrations(
    conn: &mut impl MigrationHarness<Pg>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    conn.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}

fn create_testing_pool(connection_string: &str) -> DbPool {
    dotenv().ok();
    let always_testing = AlwaysTestingTransaction {};
    let manager = ConnectionManager::<PgConnection>::new(connection_string);
    r2d2::Builder::new()
        .max_size(1)
        .connection_customizer(Box::new(always_testing))
        .build(manager)
        .expect("Failed to create pool.")
}
