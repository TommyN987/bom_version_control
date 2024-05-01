mod helpers;

use std::collections::HashMap;

use bom_version_control::{
    db::models::db_bom_version::BOMVersion,
    domain::{BOMChangeEvent, BOMDiff, Component, PartialDiff, Price, BOM},
    routes::NewBOM,
    schema::bom_versions,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::helpers::spawn_app;

#[tokio::test]
async fn create_bom_returns_created() {
    // Arrange
    let app = spawn_app().await;

    let comp = app
        .post_component("TestComp1".to_string(), "123456".to_string())
        .await;

    // Act
    let response = app.post_bom(&vec![comp]).await;

    // Assert
    assert_eq!(response.status().as_u16(), 201);
}

#[tokio::test]
async fn create_bom_returns_bad_request() {
    // Arrange
    let app = spawn_app().await;

    let comp: Component = Component {
        id: Uuid::new_v4(),
        name: "abcde".to_string(),
        part_number: "123456".to_string(),
        description: None,
        supplier: "supplier".to_string(),
        price: Price {
            value: 100.0,
            currency: "USD".to_string(),
        },
    };

    let event = BOMChangeEvent::ComponentAdded(comp, 1);

    // Act
    let response = app
        .client
        .post(&format!("{}/boms", &app.addr))
        .json(&NewBOM {
            events: vec![event],
        })
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

    let comp = app
        .post_component("TestComp1".to_string(), "123".to_string())
        .await;

    let added_bom: BOM = app
        .post_bom(&vec![comp])
        .await
        .json()
        .await
        .expect("Failed to parse response");

    // Act
    let response = app
        .client
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

    // Act
    let response = app
        .client
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

    let comp: Component = app
        .post_component("name".to_string(), "part_number".to_string())
        .await;

    let added_bom = app
        .post_bom(&vec![comp])
        .await
        .json::<BOM>()
        .await
        .expect("Failed to parse response");

    // Act
    let response = app
        .client
        .put(&format!("{}/boms/{}", &app.addr, added_bom.id))
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

    let comp: Component = app
        .post_component("name".to_string(), "part_number".to_string())
        .await;

    let added_bom = app
        .post_bom(&vec![comp.clone()])
        .await
        .json::<BOM>()
        .await
        .expect("Failed to parse response");

    // Act
    let response = app
        .client
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

    let comp: Component = app
        .post_component("name".to_string(), "part_number".to_string())
        .await;

    let added_bom = app
        .post_bom(&vec![comp.clone()])
        .await
        .json::<BOM>()
        .await
        .expect("Failed to parse response");

    // Act
    app.client
        .put(&format!("{}/boms/{}", &app.addr, added_bom.id))
        .json(&vec![
            BOMChangeEvent::ComponentUpdated(comp.id, 2),
            BOMChangeEvent::NameChanged("UpdatedName".to_string()),
        ])
        .send()
        .await
        .expect("Failed to execute update bom request");

    let old_boms: Vec<BOMVersion> = bom_versions::table
        .filter(bom_versions::bom_id.eq(added_bom.id))
        .load::<BOMVersion>(&mut app.pool.get().unwrap())
        .expect("Failed to get old bom");

    let expected_version = 1;
    let expected_bom_id = added_bom.id;
    let expected_change_events = serde_json::json!(vec![
        BOMChangeEvent::ComponentUpdated(comp.id, 2),
        BOMChangeEvent::NameChanged("UpdatedName".to_string())
    ]);
    // Assert
    assert_eq!(old_boms.last().unwrap().bom_id, expected_bom_id);
    assert_eq!(old_boms.last().unwrap().version, expected_version);
    assert_eq!(old_boms.last().unwrap().changes, expected_change_events);
}

#[tokio::test]
async fn get_bom_diff_returns_with_invalid_version_range_bad_request() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(&format!(
            "{}/boms/{}/diffs?from=1&to=1",
            &app.addr,
            Uuid::new_v4()
        ))
        .send()
        .await
        .expect("Failed to execute get bom diffs request");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn get_bom_diff_returns_correct_diffs() {
    // Arrange
    let app = spawn_app().await;

    let comp: Component = app
        .post_component("name".to_string(), "part_number".to_string())
        .await;

    let added_bom = app
        .post_bom(&vec![comp.clone()])
        .await
        .json::<BOM>()
        .await
        .expect("Failed to parse response");

    app.client
        .put(&format!("{}/boms/{}", &app.addr, added_bom.id))
        .json(&vec![
            BOMChangeEvent::ComponentUpdated(comp.id, 2),
            BOMChangeEvent::NameChanged("UpdatedName".to_string()),
        ])
        .send()
        .await
        .expect("Failed to execute update bom request");

    // Act
    let response = app
        .client
        .get(&format!(
            "{}/boms/{}/diffs?from=1&to=2",
            &app.addr, added_bom.id
        ))
        .send()
        .await
        .expect("Failed to execute get bom diffs request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let returned_diff = response
        .json::<BOMDiff>()
        .await
        .expect("Failed to parse response");

    let mut expected_components_added = HashMap::new();
    expected_components_added.insert(
        comp.id,
        PartialDiff {
            from: (comp.clone(), 1),
            to: (comp.clone(), 2),
        },
    );

    let expected_diff = BOMDiff {
        name_changed: Some(PartialDiff {
            from: "TestBom".to_string(),
            to: "UpdatedName".to_string(),
        }),
        description_changed: None,
        components_added: HashMap::new(),
        components_updated: expected_components_added,
        components_removed: Vec::new(),
    };

    assert_eq!(returned_diff, expected_diff);
}
