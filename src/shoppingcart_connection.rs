use async_graphql::SimpleObject;

use crate::{base_connection::BaseConnection, shoppingcart::ShoppingCart};

/// A connection of ShoppingCarts.
#[derive(SimpleObject)]
#[graphql(shareable)]
pub struct ShoppingCartConnection {
    /// The resulting entities.
    pub nodes: Vec<ShoppingCart>,
    /// Whether this connection has a next page.
    pub has_next_page: bool,
    /// The total amount of items in this connection.
    pub total_count: u64,
}

/// Implementation of conversion from BaseConnection<ShoppingCart> to ShoppingCartConnection.
///
/// Prevents GraphQL naming conflicts.
impl From<BaseConnection<ShoppingCart>> for ShoppingCartConnection {
    fn from(value: BaseConnection<ShoppingCart>) -> Self {
        Self {
            nodes: value.nodes,
            has_next_page: value.has_next_page,
            total_count: value.total_count,
        }
    }
}
