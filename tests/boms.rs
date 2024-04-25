mod helpers;

use bom_version_control::{
    domain::{Component, Price, BOM},
    routes::{NewBOM, NewComponent},
};
use reqwest::Client;
use uuid::Uuid;

use crate::helpers::spawn_app;

#[tokio::test]
async fn create_bom_returns_created() {
    // Arrange
    let app = spawn_app().await;
    let client = Client::new();

    let comp: Component = client
        .post(&format!("{}/components", &app.addr))
        .json(&NewComponent::new(
            "TestComponent".to_string(),
            "12345".to_string(),
            Some("TestComponentDescription".to_string()),
            "TestSupplier".to_string(),
            Price {
                value: 100.0,
                currency: "EUR".to_string(),
            },
        ))
        .send()
        .await
        .expect("Failed to execute create component request")
        .json::<Component>()
        .await
        .expect("Failed to parse response");

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

#[tokio::test]
async fn create_bom_returns_bad_request() {
    // Arrange
    let app = spawn_app().await;
    let client = Client::new();

    // Act
    let response = client
        .post(&format!("{}/boms", &app.addr))
        .json(&NewBOM::new(
            "TestBom".to_string(),
            Some("TestBomDescription".to_string()),
            vec![],
        ))
        .send()
        .await
        .expect("Failed to execute create bom request");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn get_bom_by_id_returns_correct_bom() {
    // Arrange
    let app = spawn_app().await;
    let client = Client::new();

    let comp: Component = client
        .post(&format!("{}/components", &app.addr))
        .json(&NewComponent::new(
            "TestComponent".to_string(),
            "12345".to_string(),
            Some("TestComponentDescription".to_string()),
            "TestSupplier".to_string(),
            Price {
                value: 100.0,
                currency: "EUR".to_string(),
            },
        ))
        .send()
        .await
        .expect("Failed to execute create component request")
        .json::<Component>()
        .await
        .expect("Failed to parse response");

    let added_bom = client
        .post(&format!("{}/boms", &app.addr))
        .json(&NewBOM::new(
            "TestBom".to_string(),
            Some("TestBomDescription".to_string()),
            vec![(comp.id, 1)],
        ))
        .send()
        .await
        .expect("Failed to execute create bom request")
        .json::<BOM>()
        .await
        .expect("Failed to parse response");

    // Act
    let response = client
        .get(&format!("{}/boms/{}", &app.addr, added_bom.id))
        .send()
        .await
        .expect("Failed to execute get bom request");

    assert_eq!(response.status().as_u16(), 200);

    let received_bom = response
        .json::<BOM>()
        .await
        .expect("Failed to parse response");

    // Assert
    assert_eq!(received_bom.id, added_bom.id);
}

#[tokio::test]
async fn get_bom_by_id_with_invalid_id_returns_not_found() {
    // Arrange
    let app = spawn_app().await;
    let client = Client::new();

    // Act
    let response = client
        .get(&format!("{}/boms/{}", &app.addr, Uuid::new_v4()))
        .send()
        .await
        .expect("Failed to execute get bom request");

    // Assert
    assert_eq!(response.status().as_u16(), 404);
}
