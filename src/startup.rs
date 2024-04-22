use std::net::TcpListener;

use actix_web::{dev::Server, web::Data, App, HttpServer};
use tracing_actix_web::TracingLogger;

use crate::{
    db::DbPool,
    routes::{
        create_bom, create_component, get_bom_by_id, get_component_by_id, get_components,
        health_check,
    },
};

pub fn run(listener: TcpListener, pool: DbPool) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(health_check)
            .service(get_components)
            .service(get_component_by_id)
            .service(create_component)
            .service(get_bom_by_id)
            .service(create_bom)
            .app_data(Data::new(pool.clone()))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
