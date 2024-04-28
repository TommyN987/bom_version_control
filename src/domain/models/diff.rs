use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{BOMChangeEvent, BOMChangeEventVisitor, Component, BOM};

#[derive(thiserror::Error, Debug)]
#[error("Invalid input: {0}")]
pub struct ConversionError(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PartialDiff<T> {
    pub from: T,
    pub to: T,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BOMDiff {
    pub name_changed: Option<PartialDiff<String>>,
    pub description_changed: Option<PartialDiff<String>>,
    pub components_added: HashMap<Uuid, (Component, i32)>,
    pub components_removed: Vec<Component>,
    pub components_updated: HashMap<Uuid, PartialDiff<(Component, i32)>>,
}

impl From<(&BOM, &Vec<BOMChangeEvent>)> for BOMDiff {
    fn from(value: (&BOM, &Vec<BOMChangeEvent>)) -> Self {
        let (bom, events) = value;
        let mut diff = BOMDiff::default();
        let mut visitor = BOMDiffVisitor;

        for event in events {
            event.accept(&mut visitor, bom, &mut diff);
        }

        diff
    }
}

struct BOMDiffVisitor;

impl BOMChangeEventVisitor for BOMDiffVisitor {
    fn visit_name_changed(&mut self, name: &str, bom: &BOM, diff: &mut BOMDiff) {
        diff.name_changed = Some(PartialDiff {
            from: bom.name.clone(),
            to: name.to_string(),
        });
    }

    fn visit_description_changed(&mut self, description: &str, bom: &BOM, diff: &mut BOMDiff) {
        diff.description_changed = Some(PartialDiff {
            from: bom.description.clone().unwrap_or_default(),
            to: description.to_string(),
        });
    }

    fn visit_component_added(&mut self, component: &Component, qty: i32, diff: &mut BOMDiff) {
        diff.components_added
            .insert(component.id, (component.clone(), qty));
    }

    fn visit_component_updated(&mut self, id: &Uuid, qty: i32, bom: &BOM, diff: &mut BOMDiff) {
        if let Some((c, q)) = bom.components.iter().find(|(c, _)| c.id == *id) {
            diff.components_updated.insert(
                *id,
                PartialDiff {
                    from: (c.clone(), *q),
                    to: (c.clone(), qty),
                },
            );
        }

        if let Some((_, q)) = diff.components_added.get_mut(id) {
            *q = qty;
        }
    }

    fn visit_component_removed(&mut self, component: &Component, diff: &mut BOMDiff) {
        let removed_from_added = diff.components_added.remove(&component.id);

        diff.components_updated.remove(&component.id);

        if removed_from_added.is_none() {
            diff.components_removed.push(component.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::domain::Price;

    use super::*;

    fn setup_test_bom_and_components() -> (BOM, Component, Component) {
        let component_1 = Component {
            id: Uuid::new_v4(),
            name: "Component 1".to_string(),
            part_number: "12345".to_string(),
            description: None,
            supplier: "Test Supplier".to_string(),
            price: Price {
                value: 10.0,
                currency: "USD".to_string(),
            },
        };
        let component_2 = Component {
            id: Uuid::new_v4(),
            name: "Component 2".to_string(),
            description: Some("Test description".to_string()),
            part_number: "54321".to_string(),
            supplier: "Test Supplier".to_string(),
            price: Price {
                value: 20.0,
                currency: "EUR".to_string(),
            },
        };

        let bom = BOM {
            id: Uuid::new_v4(),
            name: "Test BOM".to_string(),
            version: 1,
            description: Some("Test description".to_string()),
            components: vec![(component_1.clone(), 1)],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        (bom, component_1, component_2)
    }

    #[test]
    fn test_name_change() {
        let (bom, _, _) = setup_test_bom_and_components();
        let diff = BOMDiff::from((
            &bom,
            &vec![BOMChangeEvent::NameChanged("New Name".to_string())],
        ));

        assert_eq!(
            diff.name_changed,
            Some(PartialDiff {
                from: "Test BOM".to_string(),
                to: "New Name".to_string()
            })
        );
    }

    #[test]
    fn test_description_change() {
        let (bom, _, _) = setup_test_bom_and_components();
        let diff = BOMDiff::from((
            &bom,
            &vec![BOMChangeEvent::DescriptionChanged(
                "New description".to_string(),
            )],
        ));

        assert_eq!(
            diff.description_changed,
            Some(PartialDiff {
                from: "Test description".to_string(),
                to: "New description".to_string()
            })
        );
    }

    #[test]
    fn test_component_added() {
        let (bom, _, component_2) = setup_test_bom_and_components();

        let diff = BOMDiff::from((
            &bom,
            &vec![BOMChangeEvent::ComponentAdded(component_2.clone(), 2)],
        ));

        assert_eq!(
            diff.components_added.get(&component_2.id),
            Some(&(component_2.clone(), 2))
        );
    }

    #[test]
    fn test_multiple_components_added() {
        let (bom, _, component_2) = setup_test_bom_and_components();

        let component_3 = Component {
            id: Uuid::new_v4(),
            name: "Component 3".to_string(),
            description: Some("Test description".to_string()),
            part_number: "54321".to_string(),
            supplier: "Test Supplier".to_string(),
            price: Price {
                value: 20.0,
                currency: "EUR".to_string(),
            },
        };

        let diff = BOMDiff::from((
            &bom,
            &vec![
                BOMChangeEvent::ComponentAdded(component_2.clone(), 2),
                BOMChangeEvent::ComponentAdded(component_3.clone(), 3),
            ],
        ));

        assert_eq!(
            diff.components_added.get(&component_2.id),
            Some(&(component_2.clone(), 2))
        );

        assert_eq!(
            diff.components_added.get(&component_3.id),
            Some(&(component_3.clone(), 3))
        );
    }

    #[test]
    fn test_component_updated() {
        let (bom, component_1, _) = setup_test_bom_and_components();

        let diff = BOMDiff::from((
            &bom,
            &vec![BOMChangeEvent::ComponentUpdated(component_1.id, 2)],
        ));

        assert_eq!(
            diff.components_updated.get(&component_1.id),
            Some(&PartialDiff {
                from: (component_1.clone(), 1),
                to: (component_1.clone(), 2)
            })
        );
    }

    #[test]
    fn test_multiple_components_updated() {
        let (mut bom, component_1, component_2) = setup_test_bom_and_components();

        bom.components.push((component_2.clone(), 1));

        let diff = BOMDiff::from((
            &bom,
            &vec![
                BOMChangeEvent::ComponentUpdated(component_1.id, 2),
                BOMChangeEvent::ComponentUpdated(component_2.id, 3),
            ],
        ));

        assert_eq!(
            diff.components_updated.get(&component_1.id),
            Some(&PartialDiff {
                from: (component_1.clone(), 1),
                to: (component_1.clone(), 2)
            })
        );

        assert_eq!(
            diff.components_updated.get(&component_2.id),
            Some(&PartialDiff {
                from: (component_2.clone(), 1),
                to: (component_2.clone(), 3)
            })
        );
    }

    #[test]
    fn test_component_removed() {
        let (bom, component_1, _) = setup_test_bom_and_components();

        let diff = BOMDiff::from((
            &bom,
            &vec![BOMChangeEvent::ComponentRemoved(component_1.clone())],
        ));

        assert_eq!(diff.components_removed, vec![component_1]);
    }

    #[test]
    fn test_component_added_then_updated() {
        let (bom, _, component_2) = setup_test_bom_and_components();

        let diff = BOMDiff::from((
            &bom,
            &vec![
                BOMChangeEvent::ComponentAdded(component_2.clone(), 2),
                BOMChangeEvent::ComponentUpdated(component_2.id, 3),
            ],
        ));

        assert_eq!(
            diff.components_added.get(&component_2.id),
            Some(&(component_2.clone(), 3))
        );
    }

    #[test]
    fn test_component_added_then_removed() {
        let (bom, _, component_2) = setup_test_bom_and_components();

        let diff = BOMDiff::from((
            &bom,
            &vec![
                BOMChangeEvent::ComponentAdded(component_2.clone(), 2),
                BOMChangeEvent::ComponentRemoved(component_2.clone()),
            ],
        ));

        assert!(diff.components_added.is_empty());
        assert!(diff.components_removed.is_empty());
    }

    #[test]
    fn test_component_updated_then_removed() {
        let (bom, component_1, _) = setup_test_bom_and_components();

        let diff = BOMDiff::from((
            &bom,
            &vec![
                BOMChangeEvent::ComponentUpdated(component_1.id, 2),
                BOMChangeEvent::ComponentRemoved(component_1.clone()),
            ],
        ));

        assert!(diff.components_updated.is_empty());
        assert_eq!(diff.components_removed, vec![component_1]);
    }

    #[test]
    fn test_many_events_happened() {
        let (bom, component_1, component_2) = setup_test_bom_and_components();

        let component_3 = Component {
            id: Uuid::new_v4(),
            name: "Component 3".to_string(),
            description: Some("Test description".to_string()),
            part_number: "54321".to_string(),
            supplier: "Test Supplier".to_string(),
            price: Price {
                value: 20.0,
                currency: "EUR".to_string(),
            },
        };

        let events = vec![
            BOMChangeEvent::NameChanged("New Name".to_string()),
            BOMChangeEvent::DescriptionChanged("New description".to_string()),
            BOMChangeEvent::ComponentAdded(component_2.clone(), 2),
            BOMChangeEvent::NameChanged("Another Name".to_string()),
            BOMChangeEvent::ComponentUpdated(component_1.id, 2),
            BOMChangeEvent::ComponentUpdated(component_2.id, 3),
            BOMChangeEvent::DescriptionChanged("Another description".to_string()),
            BOMChangeEvent::ComponentRemoved(component_1.clone()),
            BOMChangeEvent::ComponentAdded(component_3.clone(), 3),
        ];

        let diff = BOMDiff::from((&bom, &events));

        assert_eq!(
            diff.name_changed,
            Some(PartialDiff {
                from: "Test BOM".to_string(),
                to: "Another Name".to_string()
            })
        );

        assert_eq!(
            diff.description_changed,
            Some(PartialDiff {
                from: "Test description".to_string(),
                to: "Another description".to_string()
            })
        );

        assert_eq!(
            diff.components_added.get(&component_2.id),
            Some(&(component_2.clone(), 3))
        );

        assert_eq!(
            diff.components_added.get(&component_3.id),
            Some(&(component_3.clone(), 3))
        );

        assert!(diff.components_updated.is_empty());

        assert_eq!(diff.components_removed, vec![component_1]);
    }
}
