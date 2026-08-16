pub mod accounts;
pub mod groups;
pub mod journal;
pub mod summary;

use sea_query::Order as SeaOrder;

#[derive(Debug, Clone, Copy)]
pub enum Order {
    Asc,
    Desc,
}

impl From<Order> for SeaOrder {
    fn from(value: Order) -> Self {
        match value {
            Order::Asc => SeaOrder::Asc,
            Order::Desc => SeaOrder::Desc,
        }
    }
}
