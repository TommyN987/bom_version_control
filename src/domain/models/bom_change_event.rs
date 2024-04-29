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

#[cfg(test)]
mod tests {
    use mockall::{
        mock,
        predicate::{self, *},
    };

    use crate::domain::Price;

    use super::*;

    mock! {
        pub BOMDiffVisitor {}
        impl BOMChangeEventVisitor for BOMDiffVisitor {
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
    }

    #[test]
    fn test_name_changed_event() {
        let mut visitor = MockBOMDiffVisitor::new();
        let mut diff = BOMDiff::default();
        let bom = BOM::default();

        visitor
            .expect_visit_name_changed()
            .with(
                predicate::always(),
                predicate::always(),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _| {});

        let event = BOMChangeEvent::NameChanged("new name".to_string());
        event.accept(&mut visitor, &bom, &mut diff);
    }

    #[test]
    fn test_description_changed_event() {
        let mut visitor = MockBOMDiffVisitor::new();
        let mut diff = BOMDiff::default();
        let bom = BOM::default();

        visitor
            .expect_visit_description_changed()
            .with(
                predicate::always(),
                predicate::always(),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _| {});

        let event = BOMChangeEvent::DescriptionChanged("new description".to_string());
        event.accept(&mut visitor, &bom, &mut diff);
    }

    #[test]
    fn test_component_added_event() {
        let mut visitor = MockBOMDiffVisitor::new();
        let mut diff = BOMDiff::default();
        let bom = BOM::default();
        let component = Component {
            id: Uuid::new_v4(),
            name: "Component".to_string(),
            description: Some("Description".to_string()),
            supplier: "Supplier".to_string(),
            part_number: "12345".to_string(),
            price: Price {
                value: 10.0,
                currency: "USD".to_string(),
            },
        };

        visitor
            .expect_visit_component_added()
            .with(
                predicate::always(),
                predicate::always(),
                predicate::always(),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {});

        let event = BOMChangeEvent::ComponentAdded(component, 10);
        event.accept(&mut visitor, &bom, &mut diff);
    }

    #[test]
    fn test_component_updated_event() {
        let mut visitor = MockBOMDiffVisitor::new();
        let mut diff = BOMDiff::default();
        let bom = BOM::default();
        let component = Uuid::new_v4();

        visitor
            .expect_visit_component_updated()
            .with(
                predicate::always(),
                predicate::always(),
                predicate::always(),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {});

        let event = BOMChangeEvent::ComponentUpdated(component, 10);
        event.accept(&mut visitor, &bom, &mut diff);
    }

    #[test]
    fn test_component_removed_event() {
        let mut visitor = MockBOMDiffVisitor::new();
        let mut diff = BOMDiff::default();
        let bom = BOM::default();
        let component = Component {
            id: Uuid::new_v4(),
            name: "Component".to_string(),
            description: Some("Description".to_string()),
            supplier: "Supplier".to_string(),
            part_number: "12345".to_string(),
            price: Price {
                value: 10.0,
                currency: "USD".to_string(),
            },
        };

        visitor
            .expect_visit_component_removed()
            .with(predicate::always(), predicate::always())
            .times(1)
            .returning(|_, _| {});

        let event = BOMChangeEvent::ComponentRemoved(component);
        event.accept(&mut visitor, &bom, &mut diff);
    }
}
