use std::net::TcpListener;

use actix_web::{dev::Server, App, HttpServer};

use crate::{db::DbPool, routes::health_check};

pub fn run(listener: TcpListener, pool: DbPool) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || App::new().app_data(pool.clone()).service(health_check))
        .listen(listener)?
        .run();
    Ok(server)
}
