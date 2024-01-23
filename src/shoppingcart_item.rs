use std::cmp::Ordering;

use async_graphql::{
    connection::{Edge, EmptyFields},
    OutputType, SimpleObject,
};
use bson::Uuid;
use bson::{datetime::DateTime, doc, Bson};
use serde::{Deserialize, Serialize};

use crate::foreign_types::ProductVariant;

/// The ShoppingCart of a user.
#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone, SimpleObject)]
pub struct ShoppingCartItem {
    /// ShoppingCartItem UUID.
    pub id: Uuid,
    /// Count of items in basket.
    pub count: u32,
    /// Timestamp when ShoppingCartItem was added.
    pub added_at: DateTime,
    /// Product variant of shopping cart item.
    pub product_variant: ProductVariant,
}

impl From<ShoppingCartItem> for Uuid {
    fn from(value: ShoppingCartItem) -> Self {
        value.id
    }
}

pub struct NodeWrapper<Node>(pub Node);

impl<Node> From<NodeWrapper<Node>> for Edge<uuid::Uuid, Node, EmptyFields>
where
    Node: Into<uuid::Uuid> + OutputType + Clone,
{
    fn from(value: NodeWrapper<Node>) -> Self {
        let uuid = Into::<uuid::Uuid>::into(value.0.clone());
        Edge::new(uuid, value.0)
    }
}

impl PartialOrd for ShoppingCartItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl From<ShoppingCartItem> for Bson {
    fn from(value: ShoppingCartItem) -> Self {
        Bson::Document(
            doc! {"id": value.id, "count": value.count, "added_at": value.added_at, "product_variant": value.product_variant},
        )
    }
}
