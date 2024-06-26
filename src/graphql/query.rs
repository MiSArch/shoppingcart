use std::any::type_name;

use async_graphql::{Context, Error, Object, Result};

use bson::Uuid;
use mongodb::{bson::doc, options::FindOneOptions, Collection, Database};
use serde::Deserialize;

use crate::authorization::authorize_user;

use super::model::{shoppingcart::ShoppingCart, shoppingcart_item::ShoppingCartItem, user::User};

/// Describes GraphQL shopping cart queries.
pub struct Query;

#[Object]
impl Query {
    /// Entity resolver for user of specific UUID.
    #[graphql(entity)]
    async fn user_entity_resolver<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of user to retrieve.")] id: Uuid,
    ) -> Result<User> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<User> = db_client.collection::<User>("users");
        query_object(&collection, id).await
    }

    /// Retrieves shopping cart item of specific UUID.
    async fn shoppingcart_item<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of shoppingcart to retrieve.")] id: Uuid,
    ) -> Result<ShoppingCartItem> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<User> = db_client.collection::<User>("users");
        let user = query_shoppingcart_item_user(&collection, id).await?;
        authorize_user(&ctx, Some(user._id))?;
        project_user_to_shopping_cart_item(user)
    }

    /// Entity resolver for shopping cart item of specific UUID.
    #[graphql(entity)]
    async fn shoppingcart_item_entity_resolver<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(key, desc = "UUID of shoppingcart to retrieve.")] id: Uuid,
    ) -> Result<ShoppingCartItem> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<User> = db_client.collection::<User>("users");
        let user = query_shoppingcart_item_user(&collection, id).await?;
        project_user_to_shopping_cart_item(user)
    }
}

/// Shared function to query a shopping cart from a MongoDB collection of shopping carts.
///
/// * `connection` - MongoDB database connection.
/// * `id` - UUID of shopping cart.
pub async fn query_shoppingcart(collection: &Collection<User>, id: Uuid) -> Result<ShoppingCart> {
    match collection.find_one(doc! {"_id": id }, None).await {
        Ok(maybe_user) => match maybe_user {
            Some(user) => Ok(user.shoppingcart),
            None => {
                let message = format!("ShoppingCart with UUID: `{}` not found.", id);
                Err(Error::new(message))
            }
        },
        Err(_) => {
            let message = format!("ShoppingCart with UUID: `{}` not found.", id);
            Err(Error::new(message))
        }
    }
}

/// Shared function to query a shopping cart item from a MongoDB collection of users.
/// Returns user which only contains the queried shopping cart item.
///
/// * `connection` - MongoDB database connection.
/// * `id` - UUID of shopping cart item.
pub async fn query_shoppingcart_item_user(collection: &Collection<User>, id: Uuid) -> Result<User> {
    let find_options = FindOneOptions::builder()
        .projection(Some(doc! {
            "shoppingcart.internal_shoppingcart_items.$": 1,
            "shoppingcart.last_updated_at": 1,
            "_id": 1
        }))
        .build();
    let message = format!("ShoppingCartItem of UUID: `{}` not found.", id);
    match collection
        .find_one(
            doc! {"shoppingcart.internal_shoppingcart_items": {
                "$elemMatch": {
                    "_id": id
                }
            }},
            Some(find_options),
        )
        .await
    {
        Ok(maybe_user) => maybe_user.ok_or(Error::new(message.clone())),
        Err(e) => Err(e.into()),
    }
}

/// Projects result of shopping cart item query, which is of type `User``, to the contained shopping cart items.
///
/// * `user` - User to project to shoppingc art items.
pub fn project_user_to_shopping_cart_item(user: User) -> Result<ShoppingCartItem> {
    let message = format!("Projection failed, shoppingcart item could not be extracted from user.");
    user.shoppingcart
        .internal_shoppingcart_items
        .iter()
        .next()
        .cloned()
        .ok_or(Error::new(message.clone()))
}

/// Queries shopping cart item user and applies projection directly.
///
/// * `connection` - MongoDB database connection.
/// * `id` - UUID of user.
pub async fn query_shoppingcart_item(
    collection: &Collection<User>,
    id: Uuid,
) -> Result<ShoppingCartItem> {
    let user = query_shoppingcart_item_user(&collection, id).await?;
    project_user_to_shopping_cart_item(user)
}

/// Queries shopping cart item user by a product variant UUID and user UUID and applies projection directly.
///
/// * `connection` - MongoDB database connection.
/// * `product_variant_id` - UUID of product variant.
/// * `id` - UUID of user.
pub async fn query_shoppingcart_item_by_product_variant_id_and_user_id(
    collection: &Collection<User>,
    product_variant_id: Uuid,
    user_id: Uuid,
) -> Result<ShoppingCartItem> {
    let user = query_shoppingcart_item_user_by_product_variant_id_and_user_id(
        &collection,
        product_variant_id,
        user_id,
    )
    .await?;
    project_user_to_shopping_cart_item(user)
}

/// Shared function to query a shopping cart item from a MongoDB collection of users by a product variant UUID and user UUID.
/// Returns user which only contains the queried shopping cart item.
///
/// * `connection` - MongoDB database connection.
/// * `product_variant_id` - UUID of product variant.
/// * `id` - UUID of user.
pub async fn query_shoppingcart_item_user_by_product_variant_id_and_user_id(
    collection: &Collection<User>,
    product_variant_id: Uuid,
    user_id: Uuid,
) -> Result<User> {
    let find_options = FindOneOptions::builder()
        .projection(Some(doc! {
            "shoppingcart.internal_shoppingcart_items.$": 1,
            "_id": 0
        }))
        .build();
    let message = format!("ShoppingCartItem referencing product variant of UUID: `{}` in shopping cart of user with UUID: `{}` not found.", product_variant_id, user_id);
    match collection
        .find_one(
            doc! {"_id": user_id, "shoppingcart.internal_shoppingcart_items": {
                "$elemMatch": {
                    "product_variant._id": product_variant_id
                }
            }},
            Some(find_options),
        )
        .await
    {
        Ok(maybe_user) => maybe_user.ok_or(Error::new(message.clone())),
        Err(_) => Err(Error::new(message)),
    }
}

/// Shared function to query an object: `T`` from a MongoDB collection of object: `T`.
///
/// * `connection` - MongoDB database connection.
/// * `id` - UUID of object.
pub async fn query_object<T: for<'a> Deserialize<'a> + Unpin + Send + Sync>(
    collection: &Collection<T>,
    id: Uuid,
) -> Result<T> {
    match collection.find_one(doc! {"_id": id }, None).await {
        Ok(maybe_object) => match maybe_object {
            Some(object) => Ok(object),
            None => {
                let message = format!("{} with UUID: `{}` not found.", type_name::<T>(), id);
                Err(Error::new(message))
            }
        },
        Err(_) => {
            let message = format!("{} with UUID: `{}` not found.", type_name::<T>(), id);
            Err(Error::new(message))
        }
    }
}
