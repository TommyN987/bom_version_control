mod helpers;

use bom_version_control::{
    db::models::db_bom_version::BOMVersion,
    domain::{BOMChangeEvent, Component, Price, BOM},
    routes::{NewBOM, NewComponent},
    schema::bom_versions,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
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

#[tokio::test]
async fn update_bom_with_invalid_input_returns_bad_request() {
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
        .put(&format!("{}/boms/{}", &app.addr, added_bom.id))
        .json(&NewBOM::new(
            "TestBom".to_string(),
            Some("TestBomDescription".to_string()),
            vec![(comp.id, 2)],
        ))
        .send()
        .await
        .expect("Failed to execute update bom request");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn update_bom_returns_created() {
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
        .put(&format!("{}/boms/{}", &app.addr, added_bom.id))
        .json(&vec![
            BOMChangeEvent::ComponentUpdated(comp.id, 2),
            BOMChangeEvent::NameChanged("UpdatedName".to_string()),
        ])
        .send()
        .await
        .expect("Failed to execute update bom request");

    // Assert
    assert_eq!(response.status().as_u16(), 201);
}

#[tokio::test]
async fn update_bom_archives_old_bom() {
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
    client
        .put(&format!("{}/boms/{}", &app.addr, added_bom.id))
        .json(&vec![
            BOMChangeEvent::ComponentUpdated(comp.id, 2),
            BOMChangeEvent::NameChanged("UpdatedName".to_string()),
        ])
        .send()
        .await
        .expect("Failed to execute update bom request");

    let old_bom = bom_versions::table
        .filter(bom_versions::bom_id.eq(added_bom.id))
        .first::<BOMVersion>(&mut app.pool.get().unwrap())
        .expect("Failed to get old bom");

    let expected_version = 1;
    let expected_bom_id = added_bom.id;
    let expected_change_events = serde_json::json!(vec![
        BOMChangeEvent::ComponentUpdated(comp.id, 2),
        BOMChangeEvent::NameChanged("UpdatedName".to_string())
    ]);
    // Assert
    assert_eq!(old_bom.bom_id, expected_bom_id);
    assert_eq!(old_bom.version, expected_version);
    assert_eq!(old_bom.changes, expected_change_events);
}
