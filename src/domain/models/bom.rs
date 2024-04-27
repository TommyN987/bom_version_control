use std::fmt::Display;

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

impl From<&Vec<BOMChangeEvent>> for BOM {
    fn from(value: &Vec<BOMChangeEvent>) -> Self {
        let mut bom = BOM::default();
        for event in value {
            bom.apply_change(event);
        }
        bom
    }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PartialDiff<T> {
    pub from: T,
    pub to: T,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BOMDiff {
    pub name_changed: Option<PartialDiff<String>>,
    pub description_changed: Option<PartialDiff<String>>,
    pub components_added: Option<Vec<(Component, i32)>>,
    pub components_removed: Option<Vec<Component>>,
    pub components_updated: Option<Vec<PartialDiff<(Component, i32)>>>,
}

impl TryFrom<(&BOM, &Vec<BOMChangeEvent>)> for BOMDiff {
    type Error = anyhow::Error;

    fn try_from(value: (&BOM, &Vec<BOMChangeEvent>)) -> Result<Self, Self::Error> {
        let mut diff = BOMDiff::default();
        let (bom, events) = value;

        if events.is_empty() {
            return Ok(diff);
        }

        for event in events {
            match event {
                BOMChangeEvent::NameChanged(name) => {
                    diff.name_changed = Some(PartialDiff {
                        from: bom.name.clone(),
                        to: name.clone(),
                    });
                }
                BOMChangeEvent::DescriptionChanged(description) => {
                    diff.description_changed = Some(PartialDiff {
                        from: bom.description.clone().unwrap_or_default(),
                        to: description.clone(),
                    });
                }
                BOMChangeEvent::ComponentAdded(component, qty) => {
                    if diff.components_added.is_none() {
                        diff.components_added = Some(vec![(component.clone(), *qty)])
                    } else {
                        diff.components_added
                            .as_mut()
                            .unwrap()
                            .push((component.clone(), *qty));
                    }
                }
                BOMChangeEvent::ComponentUpdated(id, qty) => {
                    // Check if the component exists in the original BOM
                    // If it does, add it to the updated list
                    if let Some(component) = bom.components.iter().find(|(c, _)| c.id == *id) {
                        if diff.components_updated.is_none() {
                            diff.components_updated = Some(vec![PartialDiff {
                                from: component.clone(),
                                to: (component.0.clone(), *qty),
                            }]);
                        // If the component is already in the updated list, update the quantity
                        } else if let Some(diff) = diff
                            .components_updated
                            .as_mut()
                            .unwrap()
                            .iter_mut()
                            .find(|diff| diff.from.0.id == *id)
                        {
                            diff.to.1 = *qty;
                        } else {
                            diff.components_updated.as_mut().unwrap().push(PartialDiff {
                                from: component.clone(),
                                to: (component.0.clone(), *qty),
                            });
                        }
                    }
                    // If it doesn't, it was apparently added at some point during the lifetime of the BOM
                    // So it should already be in the added list, and we just need to update the quantity
                    else if let Some(component) = diff
                        .components_added
                        .as_mut()
                        .unwrap()
                        .iter_mut()
                        .find(|(c, _)| c.id == *id)
                    {
                        component.1 = *qty;
                    }
                }
                BOMChangeEvent::ComponentRemoved(component) => {
                    // Check if the component has already been pushed to the added list
                    // If it has, remove it from there
                    if diff.components_added.is_some() {
                        diff.components_added
                            .as_mut()
                            .unwrap()
                            .retain(|(c, _)| c.id != component.id);
                    }

                    // Check if the component has already been pushed to the updated list
                    // If it has, remove it from there
                    if diff.components_updated.is_some() {
                        diff.components_updated
                            .as_mut()
                            .unwrap()
                            .retain(|diff| diff.from.0.id != component.id);
                    }

                    if diff.components_removed.is_none() {
                        diff.components_removed = Some(vec![component.clone()]);
                    }
                }
            }
        }
        Ok(diff)
    }
}
