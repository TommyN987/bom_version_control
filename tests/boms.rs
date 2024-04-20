mod helpers;

use bom_version_control::{
    db::{models::db_component::DbComponent, operations::insert_component},
    routes::NewBOM,
};
use reqwest::Client;
use uuid::Uuid;

use crate::helpers::spawn_app;

#[tokio::test]
async fn create_bom_returns_created() {
    // Arrange
    let app = spawn_app().await;
    let client = Client::new();

    let comp = tokio::task::spawn_blocking(move || {
        insert_component(
            &mut app.pool.get().unwrap(),
            DbComponent {
                id: Uuid::new_v4(),
                name: "TestComponent".to_string(),
                part_number: "12345".to_string(),
                description: Some("TestComponentDescription".to_string()),
                supplier: "TestSupplier".to_string(),
                price_value: 100.0,
                price_currency: "EUR".to_string(),
            },
        )
        .unwrap()
    })
    .await
    .unwrap();

    println!("{:?}", comp);

    // Act
    let response = client
        .post(&format!("{}/boms", &app.addr))
        .json(&NewBOM::new(
            "TestBom".to_string(),
            Some("TestBomDescription".to_string()),
            vec![(comp.id, 1)],
        ))
        .send()
        .await
        .expect("Failed to execute create bom request");

    // Assert
    assert_eq!(response.status().as_u16(), 201);
}
