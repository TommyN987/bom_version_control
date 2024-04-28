use std::fmt::Display;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{BOMDiff, Component, BOM};

pub trait BOMChangeEventVisitor {
    fn visit_name_changed(&mut self, name: &str, bom: &BOM, diff: &mut BOMDiff);
    fn visit_description_changed(&mut self, description: &str, bom: &BOM, diff: &mut BOMDiff);
    fn visit_component_added(
        &mut self,
        component: &Component,
        qty: i32,
        bom: &BOM,
        diff: &mut BOMDiff,
    );
    fn visit_component_updated(
        &mut self,
        component: &Uuid,
        qty: i32,
        bom: &BOM,
        diff: &mut BOMDiff,
    );
    fn visit_component_removed(&mut self, component: &Component, diff: &mut BOMDiff);
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum BOMChangeEvent {
    NameChanged(String),
    DescriptionChanged(String),
    ComponentAdded(Component, i32),
    ComponentRemoved(Component),
    ComponentUpdated(Uuid, i32),
}

impl BOMChangeEvent {
    pub fn accept(&self, visitor: &mut impl BOMChangeEventVisitor, bom: &BOM, diff: &mut BOMDiff) {
        match self {
            BOMChangeEvent::NameChanged(name) => visitor.visit_name_changed(name, bom, diff),
            BOMChangeEvent::DescriptionChanged(description) => {
                visitor.visit_description_changed(description, bom, diff)
            }
            BOMChangeEvent::ComponentAdded(component, qty) => {
                visitor.visit_component_added(component, *qty, bom, diff)
            }
            BOMChangeEvent::ComponentUpdated(id, qty) => {
                visitor.visit_component_updated(id, *qty, bom, diff)
            }
            BOMChangeEvent::ComponentRemoved(component) => {
                visitor.visit_component_removed(component, diff)
            }
        }
    }
}

impl Display for BOMChangeEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BOMChangeEvent::NameChanged(name) => write!(f, "NameChanged({})", name),
            BOMChangeEvent::DescriptionChanged(description) => {
                write!(f, "DescriptionChanged({})", description)
            }
            BOMChangeEvent::ComponentAdded(component, qty) => {
                write!(f, "ComponentAdded({}, {})", component.name, qty)
            }
            BOMChangeEvent::ComponentRemoved(component) => {
                write!(f, "ComponentRemoved({})", component.name)
            }
            BOMChangeEvent::ComponentUpdated(id, qty) => {
                write!(f, "ComponentUpdated({}, {})", id, qty)
            }
        }
    }
}
