use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::Component;

#[derive(Debug, Serialize)]
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

    pub fn add_component(&self, component: Component, qty: i32) -> BOMChangeEvent {
        BOMChangeEvent::ComponentAdded(component, qty)
    }

    pub fn remove_component(&self, component: Component) -> BOMChangeEvent {
        BOMChangeEvent::ComponentRemoved(component)
    }

    pub fn update_component(&self, component_id: Uuid, qty: i32) -> BOMChangeEvent {
        BOMChangeEvent::ComponentUpdated(component_id, qty)
    }

    pub fn apply_change(mut self, event: BOMChangeEvent) -> Self {
        match event {
            BOMChangeEvent::ComponentAdded(component, qty) => {
                self.components.push((component, qty));
                self
            }
            BOMChangeEvent::ComponentRemoved(component) => {
                self.components.retain(|(c, _)| c.id != component.id);
                self
            }
            BOMChangeEvent::ComponentUpdated(id, qty) => {
                self.components.iter_mut().for_each(|(c, q)| {
                    if c.id == id {
                        *q = qty;
                    }
                });
                self
            }
        }
    }
}

pub enum BOMChangeEvent {
    ComponentAdded(Component, i32),
    ComponentRemoved(Component),
    ComponentUpdated(Uuid, i32),
}
