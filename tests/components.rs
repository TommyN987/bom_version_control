mod helpers;

use crate::helpers::spawn_app;
use bom_version_control::{
    domain::{Component, Price},
    routes::NewComponent,
};
use reqwest::Client;

#[tokio::test]
async fn create_component_returns_created() {
    // Arrange
    let app = spawn_app().await;
    let client = Client::new();

    // Act
    let response = client
        .post(&format!("{}/components", &app.addr))
        .json(&NewComponent::new(
            "TestName1".to_string(),
            "12345".to_string(),
            Some("TestDescription".to_string()),
            "TestSupplier".to_string(),
            Price {
                value: 100.0,
                currency: "EUR".to_string(),
            },
        ))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 201);
}

#[tokio::test]
async fn get_component_by_id_returns_not_found_for_nonexistent_component() {
    // Arrange
    let app = spawn_app().await;
    let client = Client::new();

    // Act
    let response = client
        .get(&format!(
            "{}/components/00000000-0000-0000-0000-000000000000",
            &app.addr
        ))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn create_component_persists_component() {
    // Arrange
    let app = spawn_app().await;
    let client = Client::new();

    let added_component: Component = client
        .post(&format!("{}/components", &app.addr))
        .json(&NewComponent::new(
            "TestName1".to_string(),
            "12345".to_string(),
            Some("TestDescription".to_string()),
            "TestSupplier".to_string(),
            Price {
                value: 100.0,
                currency: "EUR".to_string(),
            },
        ))
        .send()
        .await
        .expect("Failed to execute request")
        .json()
        .await
        .expect("Failed to parse response");

    // Act
    let returned_component: Component = client
        .get(&format!("{}/components/{}", &app.addr, &added_component.id))
        .send()
        .await
        .expect("Failed to execute request")
        .json()
        .await
        .expect("Failed to parse response");

    // Assert
    assert_eq!(added_component, returned_component);
}

#[tokio::test]
async fn create_component_returns_bad_request_on_invalid_input() {
    // Arrange
    let app = spawn_app().await;
    let client = Client::new();

    #[derive(Debug, serde::Serialize)]
    struct EmptyStruct {}

    // Act
    let response = client
        .post(&format!("{}/components", &app.addr))
        .json(&EmptyStruct {})
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 400);
}
