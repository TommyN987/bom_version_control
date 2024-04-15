use crate::helpers::spawn_app;
use bom_version_control::domain::Component;
use reqwest::Client;

#[tokio::test]
async fn get_components_returns_empty_list() {
    // Arrange
    let app = spawn_app().await;
    let client = Client::new();

    // Act
    let response = client
        .get(&format!("{}/components", &app.addr))
        .send()
        .await
        .expect("Failed to execute request");

    println!("{:?}", response);
    assert_eq!(response.status().as_u16(), 200);

    let components: Vec<Component> = response.json().await.expect("Failed to parse response");

    // Assert
    assert_eq!(0, components.len());
}
