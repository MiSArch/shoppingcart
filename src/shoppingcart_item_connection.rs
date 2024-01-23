use async_graphql::SimpleObject;

use crate::{base_connection::BaseConnection, shoppingcart_item::ShoppingCartItem};

/// A connection of ShoppingCart Items.
#[derive(SimpleObject)]
#[graphql(shareable)]
pub struct ShoppingCartItemConnection {
    /// The resulting entities.
    pub nodes: Vec<ShoppingCartItem>,
    /// Whether this connection has a next page.
    pub has_next_page: bool,
    /// The total amount of items in this connection.
    pub total_count: u64,
}

/// Implementation of conversion from BaseConnection<ShoppingCart> to ShoppingCartItemConnection.
///
/// Prevents GraphQL naming conflicts.
impl From<BaseConnection<ShoppingCartItem>> for ShoppingCartItemConnection {
    fn from(value: BaseConnection<ShoppingCartItem>) -> Self {
        Self {
            nodes: value.nodes,
            has_next_page: value.has_next_page,
            total_count: value.total_count,
        }
    }
}
