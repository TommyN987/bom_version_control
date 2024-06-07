use std::{error::Error, net::TcpListener, sync::Arc};

use bom_version_control::{
    configuration::get_config,
    domain::{
        newtypes::{new_bom::NewBOM, new_component::NewComponent},
        BOMChangeEvent, Component, Price,
    },
    infrastructure::{aliases::DbPool, repositories::bom_repository::BomRepository},
    services::bom_service::BomService,
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
    pub bom_service: Arc<BomService>,
    pub client: reqwest::Client,
}

#[allow(dead_code)]
impl TestApp {
    pub async fn post_component(&self, name: String, part_number: String) -> Component {
        self.client
            .post(&format!("{}/components", self.addr))
            .json(&NewComponent::new(
                name,
                part_number,
                Some("TestComponentDescription".to_string()),
                "TestSupplier".to_string(),
                Price {
                    value: 100.0,
                    currency: "EUR".to_string(),
                },
            ))
            .send()
            .await
            .expect("Failed to execute create component request")
            .json::<Component>()
            .await
            .expect("Failed to parse response")
    }

    pub async fn post_bom(&self, components: &[Component]) -> reqwest::Response {
        let name_change = BOMChangeEvent::NameChanged("TestBom".to_string());

        let description_change =
            BOMChangeEvent::DescriptionChanged("TestBomDescription".to_string());

        let events: Vec<BOMChangeEvent> = components
            .iter()
            .map(|c| BOMChangeEvent::ComponentAdded(c.clone(), 1))
            .chain(vec![name_change, description_change])
            .collect();

        self.client
            .post(&format!("{}/boms", self.addr))
            .json(&NewBOM { events })
            .send()
            .await
            .expect("Failed to execute create bom request")
    }
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

    let repo = BomRepository::new(pool.clone());

    let bom_service = Arc::new(BomService::new(Arc::new(repo)));

    run_migrations(&mut pool.get().expect("Failed to get connection to db"))
        .expect("Failed to run migrations");

    let server = run(listener, bom_service.clone()).expect("Failed to bind address");

    tokio::spawn(server);

    TestApp {
        addr,
        bom_service,
        client: reqwest::Client::new(),
    }
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
