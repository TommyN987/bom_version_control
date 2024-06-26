use diesel::{
    r2d2::{self, ConnectionManager},
    PgConnection,
};
use dotenv::dotenv;

use super::aliases::DbPool;

pub fn create_db_pool(connection_string: &str) -> DbPool {
    dotenv().ok();
    let manager = ConnectionManager::<PgConnection>::new(connection_string);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}
