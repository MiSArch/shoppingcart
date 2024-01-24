use crate::{shoppingcart_item::ShoppingCartItem, user::User, ShoppingCart};
use async_graphql::{Context, Error, Object, Result};

use bson::Uuid;
use mongodb::{bson::doc, options::FindOneOptions, Collection, Database};

use serde::{Deserialize, Serialize};

/// Describes GraphQL shoppingcart queries.
pub struct Query;

#[Object]
impl Query {
    /// Entity resolver for user of specific id.
    #[graphql(entity)]
    async fn user_entity_resolver<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of user to retrieve.")] id: Uuid,
    ) -> Result<User> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<User> = db_client.collection::<User>("users");
        query_user(&collection, id).await
    }

    /// Retrieves shoppingcart item of specific id.
    async fn shoppingcart_item<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of shoppingcart to retrieve.")] id: Uuid,
    ) -> Result<ShoppingCartItem> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<User> = db_client.collection::<User>("users");
        query_shoppingcart_item(&collection, id).await
    }

    /// Entity resolver for shoppingcart item of specific id.
    #[graphql(entity)]
    async fn shoppingcart_item_entity_resolver<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(key, desc = "UUID of shoppingcart to retrieve.")] id: Uuid,
    ) -> Result<ShoppingCartItem> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<User> = db_client.collection::<User>("users");
        query_shoppingcart_item(&collection, id).await
    }
}

/// Shared function to query a shoppingcart from a MongoDB collection of shoppingcarts.
///
/// * `connection` - MongoDB database connection.
/// * `stringified_uuid` - UUID of shoppingcart as String.
pub async fn query_shoppingcart(collection: &Collection<User>, id: Uuid) -> Result<ShoppingCart> {
    match collection.find_one(doc! {"_id": id }, None).await {
        Ok(maybe_user) => match maybe_user {
            Some(user) => Ok(user.shoppingcart),
            None => {
                let message = format!("ShoppingCart with UUID id: `{}` not found.", id);
                Err(Error::new(message))
            }
        },
        Err(_) => {
            let message = format!("ShoppingCart with UUID id: `{}` not found.", id);
            Err(Error::new(message))
        }
    }
}

/// Helper struct for MongoDB projection.
#[derive(Serialize, Deserialize)]
struct ProjectedShoppingCart {
    #[serde(rename = "shoppingcart")]
    projected_inner_shoppingcart: ProjectedInnerShoppingCart,
}

/// Helper struct for MongoDB projection.
#[derive(Serialize, Deserialize)]
struct ProjectedInnerShoppingCart {
    internal_shoppingcart_items: Vec<ShoppingCartItem>,
}

/// Shared function to query a shoppingcart item from a MongoDB collection of users.
///
/// * `connection` - MongoDB database connection.
/// * `id` - UUID of shoppingcart item.
///
/// Specifies options with projection.
pub async fn query_shoppingcart_item(
    collection: &Collection<User>,
    id: Uuid,
) -> Result<ShoppingCartItem> {
    let find_options = FindOneOptions::builder()
        .projection(Some(doc! {
            "shoppingcart.internal_shoppingcart_items.$": 1,
            "_id": 0
        }))
        .build();
    let projected_collection = collection.clone_with_type::<ProjectedShoppingCart>();
    let message = format!("ShoppingCartItem of UUID id: `{}` not found.", id);
    match projected_collection
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
        Ok(maybe_shoppingcart_projection) => maybe_shoppingcart_projection
            .and_then(|projection| {
                projection
                    .projected_inner_shoppingcart
                    .internal_shoppingcart_items
                    .first()
                    .cloned()
            })
            .ok_or_else(|| Error::new(message.clone())),
        Err(_) => Err(Error::new(message)),
    }
}

/// Shared function to query a shoppingcart item from a MongoDB collection of users.
///
/// * `connection` - MongoDB database connection.
/// * `id` - UUID of user.
///
/// Specifies options with projection.
pub async fn query_shoppingcart_item_by_product_variant_id_and_user_id(
    collection: &Collection<User>,
    product_variant_id: Uuid,
    user_id: Uuid,
) -> Result<ShoppingCartItem> {
    let find_options = FindOneOptions::builder()
        .projection(Some(doc! {
            "shoppingcart.internal_shoppingcart_items.$": 1,
            "_id": 0
        }))
        .build();
    let projected_collection = collection.clone_with_type::<ProjectedShoppingCart>();
    let message = format!("ShoppingCartItem referencing product variant of UUID: `{}` in shopping cart of user with UUID: `{}` not found.", product_variant_id, user_id);
    match projected_collection
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
        Ok(maybe_shoppingcart_projection) => maybe_shoppingcart_projection
            .and_then(|projection| {
                projection
                    .projected_inner_shoppingcart
                    .internal_shoppingcart_items
                    .first()
                    .cloned()
            })
            .ok_or_else(|| Error::new(message.clone())),
        Err(_) => Err(Error::new(message)),
    }
}

/// Shared function to query a user from a MongoDB collection of users.
///
/// * `connection` - MongoDB database connection.
/// * `id` - UUID of user.
pub async fn query_user(collection: &Collection<User>, id: Uuid) -> Result<User> {
    match collection.find_one(doc! {"_id": id }, None).await {
        Ok(maybe_user) => match maybe_user {
            Some(user) => Ok(user),
            None => {
                let message = format!("User with UUID id: `{}` not found.", id);
                Err(Error::new(message))
            }
        },
        Err(_) => {
            let message = format!("User with UUID id: `{}` not found.", id);
            Err(Error::new(message))
        }
    }
}
