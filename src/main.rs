use std::{net::TcpListener, sync::Arc};

use bom_version_control::{
    configuration::get_config,
    infrastructure::{
        aliases::DbPool, connection::create_db_pool, models::component::Component as DbComponent,
        repositories::bom_repository::BomRepository,
    },
    schema::components,
    services::bom_service::BomService,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use diesel::{insert_into, QueryDsl, RunQueryDsl};
use secrecy::ExposeSecret;
use uuid::Uuid;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("bom_version_control".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_config().expect("Failed to read configuration");
    let pool = create_db_pool(config.db.conn_string().expose_secret());

    if let Err(e) = populate_components_table(&pool) {
        eprintln!("Failed to populate components table: {}", e);
    }

    let repo = BomRepository::new(pool.clone());

    let bom_service = Arc::new(BomService::new(Arc::new(repo)));

    let addr = format!("{}:{}", config.app.host, config.app.port);
    println!("Server is running on: http://{}", addr);
    let listener = TcpListener::bind(addr).expect("Failed to bind to port");

    run(listener, bom_service)?.await?;

    Ok(())
}

fn populate_components_table(pool: &DbPool) -> Result<(), diesel::result::Error> {
    if is_components_table_empty(pool)? {
        let mut conn = pool.get().expect("Failed to get DB connection");
        let new_components: Vec<DbComponent> = (1..=100)
            .map(|i| DbComponent {
                id: Uuid::new_v4(),
                name: format!("Component {}", i),
                part_number: format!("PRT-{}", i),
                description: Some(format!("Description of component {}", i)),
                supplier: format!("Supplier {}", i),
                price_value: (rand::random::<f32>() * 100.0).floor(),
                price_currency: "USD".to_string(),
            })
            .collect();

        insert_into(components::table)
            .values(&new_components)
            .execute(&mut conn)?;
    }
    Ok(())
}

fn is_components_table_empty(pool: &DbPool) -> Result<bool, diesel::result::Error> {
    let mut conn = pool.get().expect("Failed to get DB connection");
    let count = components::table.count().get_result::<i64>(&mut conn)?;
    Ok(count == 0)
}
