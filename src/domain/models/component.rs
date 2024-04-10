use uuid::Uuid;

#[derive(Debug)]
pub struct Price {
    pub value: i32,
    pub currency: String,
}

#[derive(Debug)]
pub struct Component {
    pub id: Uuid,
    pub name: String,
    pub part_number: String,
    pub description: Option<String>,
    pub supplier: String,
    pub price: Price,
}
