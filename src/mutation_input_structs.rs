use async_graphql::{InputObject, SimpleObject};
use bson::Uuid;
use std::collections::HashSet;

#[derive(SimpleObject, InputObject)]
pub struct AddShoppingCartInput {
    /// UUID of user owning the shoppingcart.
    pub user_id: Uuid,
    /// ShoppingCartItems in shoppingcart.
    pub shopping_cart_items: HashSet<ShoppingCartItemInput>,
}

#[derive(SimpleObject, InputObject)]
pub struct UpdateShoppingCartInput {
    /// UUID of shoppingcart to update.
    pub id: Uuid,
    /// ShoppingCartItems of shoppingcart to update
    pub shopping_cart_items: Option<HashSet<ShoppingCartItemInput>>,
}

#[derive(SimpleObject, InputObject, Eq, Hash, PartialEq)]
pub struct ShoppingCartItemInput {
    /// Count of items in basket.
    pub count: u32,
    /// Uuid of product variant.
    pub product_variant_id: Uuid,
}

#[derive(SimpleObject, InputObject)]
pub struct AddShoppingCartItemInput {
    /// UUID of shoppingcart to add the item to.
    pub id: Uuid,
    /// ShoppingCartItem in shoppingcart to update
    pub shopping_cart_item: ShoppingCartItemInput,
}

#[derive(SimpleObject, InputObject, Eq, Hash, PartialEq)]
pub struct UpdateShoppingCartItemInput {
    /// UUID of shoppingcart item to update.
    pub id: Uuid,
    /// Count of items in basket.
    pub count: u32,
}
