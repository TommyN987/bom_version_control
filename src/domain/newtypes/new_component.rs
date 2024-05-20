use crate::domain::Price;

pub struct NewComponent {
    pub name: String,
    pub part_number: String,
    pub description: Option<String>,
    pub supplier: String,
    pub price: Price,
}
