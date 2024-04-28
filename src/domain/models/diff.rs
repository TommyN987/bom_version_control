use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{BOMChangeEvent, BOMChangeEventVisitor, Component, BOM};

#[derive(Debug, Serialize, Deserialize)]
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
        diff.components_added.remove(&component.id);
        diff.components_updated.remove(&component.id);

        diff.components_removed.push(component.clone());
    }
}
