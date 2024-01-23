use async_graphql::SimpleObject;
use bson::{doc, Bson, Uuid};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, hash::Hash};

/// Foreign type of a user.
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone, SimpleObject)]
#[graphql(unresolvable)]
pub struct User {
    /// UUID of the user.
    pub _id: Uuid,
}

/// Foreign type of a product variant.
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Copy, Clone, SimpleObject)]
#[graphql(unresolvable)]
pub struct ProductVariant {
    /// UUID of the product variant.
    pub _id: Uuid,
}

impl PartialOrd for ProductVariant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self._id.partial_cmp(&other._id)
    }
}

impl From<ProductVariant> for Bson {
    fn from(value: ProductVariant) -> Self {
        Bson::Document(doc!("id": value._id))
    }
}
