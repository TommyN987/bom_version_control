use std::{net::TcpListener, sync::Arc};

use actix_web::{dev::Server, web::Data, App, HttpServer};
use tracing_actix_web::TracingLogger;

use crate::{
    routes::{
        create_bom, create_component, get_all_boms, get_bom_by_id, get_bom_diff, get_bom_version,
        get_component_by_id, get_components, health_check, revert_bom_to_version,
        search_components, update_bom,
    },
    services::bom_service::BomService,
};

pub fn run(listener: TcpListener, bom_service: Arc<BomService>) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(health_check)
            .service(get_all_boms)
            .service(search_components)
            .service(get_component_by_id)
            .service(get_components)
            .service(create_component)
            .service(get_bom_by_id)
            .service(create_bom)
            .service(update_bom)
            .service(get_bom_diff)
            .service(get_bom_version)
            .service(revert_bom_to_version)
            .app_data(Data::from(bom_service.clone()))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
