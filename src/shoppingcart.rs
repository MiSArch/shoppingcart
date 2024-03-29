use std::{cmp::Ordering, collections::HashSet};

use async_graphql::{ComplexObject, Result, SimpleObject};

use bson::datetime::DateTime;

use serde::{Deserialize, Serialize};

use crate::{
    order_datatypes::{CommonOrderInput, OrderDirection},
    shoppingcart_item::ShoppingCartItem,
    shoppingcart_item_connection::ShoppingCartItemConnection,
};

/// The ShoppingCart of a user.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, SimpleObject)]
#[graphql(complex)]
pub struct ShoppingCart {
    /// Timestamp when ShoppingCart was last updated.
    pub last_updated_at: DateTime,
    #[graphql(skip)]
    pub internal_shoppingcart_items: HashSet<ShoppingCartItem>,
}

impl ShoppingCart {
    pub fn new() -> Self {
        Self {
            last_updated_at: DateTime::now(),
            internal_shoppingcart_items: HashSet::new(),
        }
    }
}

#[ComplexObject]
impl ShoppingCart {
    /// Retrieves product variants.
    async fn shoppingcart_items(
        &self,
        #[graphql(desc = "Describes that the `first` N shoppingcarts should be retrieved.")]
        first: Option<usize>,
        #[graphql(desc = "Describes how many shoppingcarts should be skipped at the beginning.")]
        skip: Option<usize>,
        #[graphql(desc = "Specifies the order in which shoppingcarts are retrieved.")]
        order_by: Option<CommonOrderInput>,
    ) -> Result<ShoppingCartItemConnection> {
        let mut shoppingcart_items: Vec<ShoppingCartItem> = self
            .internal_shoppingcart_items
            .clone()
            .into_iter()
            .collect();
        sort_shoppingcart_items(&mut shoppingcart_items, order_by);
        let total_count = shoppingcart_items.len();
        let definitely_skip = skip.unwrap_or(0);
        let definitely_first = first.unwrap_or(usize::MAX);
        let shoppingcart_items_part: Vec<ShoppingCartItem> = shoppingcart_items
            .into_iter()
            .skip(definitely_skip)
            .take(definitely_first)
            .collect();
        let has_next_page = total_count > shoppingcart_items_part.len() + definitely_skip;
        Ok(ShoppingCartItemConnection {
            nodes: shoppingcart_items_part,
            has_next_page,
            total_count: total_count as u64,
        })
    }
}

/// Sorts vector of product variants according to BaseOrder.
///
/// * `shoppingcart_items` - Vector of product variants to sort.
/// * `order_by` - Specifies order of sorted result.
fn sort_shoppingcart_items(
    shoppingcart_items: &mut Vec<ShoppingCartItem>,
    order_by: Option<CommonOrderInput>,
) {
    let comparator: fn(&ShoppingCartItem, &ShoppingCartItem) -> bool =
        match order_by.unwrap_or_default().direction.unwrap_or_default() {
            OrderDirection::Asc => |x, y| x < y,
            OrderDirection::Desc => |x, y| x > y,
        };
    shoppingcart_items.sort_by(|x, y| match comparator(x, y) {
        true => Ordering::Less,
        false => Ordering::Greater,
    });
}
