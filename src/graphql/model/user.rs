use async_graphql::SimpleObject;
use bson::Uuid;
use serde::{Deserialize, Serialize};

use super::shoppingcart::ShoppingCart;

/// Type of a user owning shoppingcarts.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, SimpleObject)]
pub struct User {
    /// UUID of the user.
    pub _id: Uuid,
    /// Shopping cart of the user.
    pub shoppingcart: ShoppingCart,
}
