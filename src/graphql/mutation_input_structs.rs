use async_graphql::{InputObject, SimpleObject};
use bson::Uuid;
use std::collections::HashSet;

#[derive(SimpleObject, InputObject)]
pub struct UpdateShoppingCartInput {
    /// UUID of user owning shopping cart.
    pub id: Uuid,
    /// Shopping cart items of shopping cart to update.
    pub shopping_cart_items: Option<HashSet<ShoppingCartItemInput>>,
}

#[derive(SimpleObject, InputObject, Eq, Hash, PartialEq)]
pub struct ShoppingCartItemInput {
    /// Count of shopping cart items in cart.
    pub count: u32,
    /// UUID of product variant.
    pub product_variant_id: Uuid,
}

#[derive(SimpleObject, InputObject)]
pub struct CreateShoppingCartItemInput {
    /// UUID of user owning the shopping cart.
    pub id: Uuid,
    /// shopping cart item in shopping cart to update
    pub shopping_cart_item: ShoppingCartItemInput,
}

#[derive(SimpleObject, InputObject, Eq, Hash, PartialEq)]
pub struct UpdateShoppingCartItemInput {
    /// UUID of shoppingcart item to update.
    pub id: Uuid,
    /// Count of shopping cart items in cart.
    pub count: u32,
}
