use std::net::TcpListener;

use bom_version_control::startup::run;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to port 8080");
    run(listener)?.await?;
    Ok(())
}
