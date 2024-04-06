use std::net::TcpListener;

use bom_version_control::{configuration::get_config, db::create_db_pool, startup::run};
use secrecy::ExposeSecret;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let config = get_config().expect("Failed to read configuration");
    let pool = create_db_pool(config.db.conn_string().expose_secret());
    let addr = format!("{}:{}", config.app.host, config.app.port);
    let listener = TcpListener::bind(addr).expect("Failed to bind to port");
    run(listener, pool)?.await?;
    Ok(())
}
