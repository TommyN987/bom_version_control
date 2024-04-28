use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::Component;

use super::BOMChangeEvent;

#[derive(Debug, Serialize, Deserialize)]
pub struct BOM {
    pub id: Uuid,
    pub name: String,
    pub version: i32,
    pub description: Option<String>,
    pub components: Vec<(Component, i32)>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for BOM {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            version: 0,
            description: None,
            components: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl From<&Vec<BOMChangeEvent>> for BOM {
    fn from(value: &Vec<BOMChangeEvent>) -> Self {
        let mut bom = BOM::default();
        for event in value {
            bom.apply_change(event);
        }
        bom
    }
}

impl BOM {
    pub fn apply_change(&mut self, event: &BOMChangeEvent) {
        match event {
            BOMChangeEvent::NameChanged(name) => {
                self.name = name.clone();
            }
            BOMChangeEvent::DescriptionChanged(description) => {
                self.description = Some(description.clone());
            }
            BOMChangeEvent::ComponentAdded(component, qty) => {
                self.components.push((component.clone(), *qty));
            }
            BOMChangeEvent::ComponentRemoved(component) => {
                self.components.retain(|(c, _)| c.id != component.id);
            }
            BOMChangeEvent::ComponentUpdated(id, qty) => {
                self.components.iter_mut().for_each(|(c, q)| {
                    if c.id == *id {
                        *q = *qty;
                    }
                });
            }
        }
    }

    pub fn increment_version(&mut self) {
        self.version += 1;
    }
}
