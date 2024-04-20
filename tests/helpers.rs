use std::{error::Error, net::TcpListener};

use bom_version_control::{
    configuration::get_config,
    db::DbPool,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use diesel::{
    pg::Pg,
    r2d2::{self, ConnectionManager},
    PgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use uuid::Uuid;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub addr: String,
    pub pool: DbPool,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
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
    let manager = ConnectionManager::<PgConnection>::new(connection_string);
    r2d2::Builder::new()
        .max_size(1)
        .build(manager)
        .expect("Failed to create pool.")
}
