use std::cmp::Ordering;

use async_graphql::SimpleObject;
use bson::Uuid;
use bson::{datetime::DateTime, doc, Bson};
use serde::{Deserialize, Serialize};

use super::foreign_types::ProductVariant;

/// Shopping cart item in a shopping cart of a user.
#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone, SimpleObject)]
pub struct ShoppingCartItem {
    /// Shopping cart item UUID.
    pub _id: Uuid,
    /// Count of items in basket.
    pub count: u32,
    /// Timestamp when shopping cart item was added.
    pub added_at: DateTime,
    /// Product variant of shopping cart item.
    pub product_variant: ProductVariant,
}

impl From<ShoppingCartItem> for Uuid {
    fn from(value: ShoppingCartItem) -> Self {
        value._id
    }
}

impl PartialOrd for ShoppingCartItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self._id.partial_cmp(&other._id)
    }
}

impl From<ShoppingCartItem> for Bson {
    fn from(value: ShoppingCartItem) -> Self {
        Bson::Document(
            doc! {"_id": value._id, "count": value.count, "added_at": value.added_at, "product_variant": value.product_variant},
        )
    }
}
