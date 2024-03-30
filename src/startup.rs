use std::net::TcpListener;

use actix_web::{dev::Server, App, HttpServer};

use crate::routes::health_check;

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || App::new().service(health_check))
        .listen(listener)?
        .run();
    Ok(server)
}
