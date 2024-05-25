use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::domain::Price;

#[derive(Debug, Deserialize, Serialize)]
pub struct NewComponent {
    pub name: String,
    pub part_number: String,
    pub description: Option<String>,
    pub supplier: String,
    pub price: Price,
}

impl Display for NewComponent {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "NewComponent {{ name: {}, part_number: {}, description: {:?}, supplier: {}, price: {} {} }}",
            self.name, self.part_number, self.description, self.supplier, self.price.value, self.price.currency
        )
    }
}

impl NewComponent {
    pub fn new(
        name: String,
        part_number: String,
        description: Option<String>,
        supplier: String,
        price: Price,
    ) -> Self {
        Self {
            name,
            part_number,
            description,
            supplier,
            price,
        }
    }
}
