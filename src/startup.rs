use std::net::TcpListener;

use actix_web::{dev::Server, web::Data, App, HttpServer};

use crate::{
    db::DbPool,
    routes::{create_component, get_component_by_id, get_components, health_check},
};

pub fn run(listener: TcpListener, pool: DbPool) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(health_check)
            .service(get_components)
            .service(get_component_by_id)
            .service(create_component)
    })
    .listen(listener)?
    .run();
    Ok(server)
}
