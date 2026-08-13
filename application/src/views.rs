pub mod accounts;

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
