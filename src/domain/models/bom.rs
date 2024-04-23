use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::Component;

#[derive(Debug, Serialize, Deserialize)]
pub struct BOM {
    pub id: Uuid,
    pub name: String,
    pub version: i32,
    pub description: Option<String>,
    pub components: Vec<(Component, i32)>,
    pub created_at: DateTime<Utc>,
}

impl BOM {
    pub fn new(
        name: String,
        description: Option<String>,
        components: Vec<(Component, i32)>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            version: 1,
            description,
            components,
            created_at: Utc::now(),
        }
    }

    pub fn apply_change(&mut self, event: BOMChangeEvent) {
        match event {
            BOMChangeEvent::NameChanged(name) => {
                self.name = name;
            }
            BOMChangeEvent::DescriptionChanged(description) => {
                self.description = Some(description);
            }
            BOMChangeEvent::ComponentAdded(component, qty) => {
                self.components.push((component, qty));
            }
            BOMChangeEvent::ComponentRemoved(component) => {
                self.components.retain(|(c, _)| c.id != component.id);
            }
            BOMChangeEvent::ComponentUpdated(id, qty) => {
                self.components.iter_mut().for_each(|(c, q)| {
                    if c.id == id {
                        *q = qty;
                    }
                });
            }
        }
    }

    pub fn increment_version(&mut self) {
        self.version += 1;
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum BOMChangeEvent {
    NameChanged(String),
    DescriptionChanged(String),
    ComponentAdded(Component, i32),
    ComponentRemoved(Component),
    ComponentUpdated(Uuid, i32),
}
