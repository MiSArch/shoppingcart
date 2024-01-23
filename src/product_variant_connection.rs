use async_graphql::SimpleObject;

use crate::{base_connection::BaseConnection, foreign_types::ProductVariant};

/// A connection of ProductVariants.
#[derive(SimpleObject)]
#[graphql(shareable)]
pub struct ProductVariantConnection {
    /// The resulting entities.
    pub nodes: Vec<ProductVariant>,
    /// Whether this connection has a next page.
    pub has_next_page: bool,
    /// The total amount of items in this connection.
    pub total_count: u64,
}

/// Implementation of conversion from BaseConnection<ProductVariant> to ProductVariantConnection.
///
/// Prevents GraphQL naming conflicts.
impl From<BaseConnection<ProductVariant>> for ProductVariantConnection {
    fn from(value: BaseConnection<ProductVariant>) -> Self {
        Self {
            nodes: value.nodes,
            has_next_page: value.has_next_page,
            total_count: value.total_count,
        }
    }
}
