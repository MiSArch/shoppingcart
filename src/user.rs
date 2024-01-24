use async_graphql::SimpleObject;
use bson::Uuid;
use serde::{Deserialize, Serialize};

use crate::shoppingcart::ShoppingCart;

/// Type of a user owning shoppingcarts.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, SimpleObject)]
pub struct User {
    /// UUID of the user.
    pub _id: Uuid,
    pub shoppingcart: ShoppingCart,
}
