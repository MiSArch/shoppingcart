use crate::{shoppingcart_item::ShoppingCartItem, user::User, ShoppingCart};
use async_graphql::{Context, Error, Object, Result};

use bson::Uuid;
use mongodb::{bson::doc, options::FindOneOptions, Collection, Database};

use serde::{Deserialize, Serialize};

/// Describes GraphQL shoppingcart queries.
pub struct Query;

#[Object]
impl Query {
    /// Retrieves user owning shoppingcarts.
    async fn user<'a>(
        &self,
        _ctx: &Context<'a>,
        #[graphql(desc = "UUID of user to retrieve.")] id: Uuid,
    ) -> Result<User> {
        Ok(User { _id: id })
    }

    /// Retrieves shoppingcart of specific id.
    async fn shoppingcart<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of shoppingcart to retrieve.")] id: Uuid,
    ) -> Result<ShoppingCart> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<ShoppingCart> =
            db_client.collection::<ShoppingCart>("shoppingcarts");
        query_shoppingcart(&collection, id).await
    }

    /// Entity resolver for shoppingcart of specific key.
    #[graphql(entity)]
    async fn shoppingcart_entity_resolver<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(key, desc = "UUID of shoppingcart to retrieve.")] id: Uuid,
    ) -> Result<ShoppingCart> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<ShoppingCart> =
            db_client.collection::<ShoppingCart>("shoppingcarts");
        query_shoppingcart(&collection, id).await
    }

    /// Retrieves shoppingcart item of specific id.
    async fn shoppingcart_item<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of shoppingcart to retrieve.")] id: Uuid,
    ) -> Result<ShoppingCartItem> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<ShoppingCart> =
            db_client.collection::<ShoppingCart>("shoppingcarts");
        query_shoppingcart_item(&collection, id).await
    }

    /// Entity resolver for shoppingcart item of specific key.
    #[graphql(entity)]
    async fn shoppingcart_item_entity_resolver<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(key, desc = "UUID of shoppingcart to retrieve.")] id: Uuid,
    ) -> Result<ShoppingCartItem> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<ShoppingCart> =
            db_client.collection::<ShoppingCart>("shoppingcarts");
        query_shoppingcart_item(&collection, id).await
    }
}

/// Shared function to query a shoppingcart from a MongoDB collection of shoppingcarts
///
/// * `connection` - MongoDB database connection.
/// * `stringified_uuid` - UUID of shoppingcart as String.
pub async fn query_shoppingcart(
    collection: &Collection<ShoppingCart>,
    id: Uuid,
) -> Result<ShoppingCart> {
    match collection.find_one(doc! {"_id": id }, None).await {
        Ok(maybe_shoppingcart) => match maybe_shoppingcart {
            Some(shoppingcart) => Ok(shoppingcart),
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
    #[serde(rename = "internal_shoppingcart_items")]
    internal_shoppingcart_items: Vec<ShoppingCartItem>,
}

/// Shared function to query a shoppingcart item from a MongoDB collection of shoppingcarts
///
/// * `connection` - MongoDB database connection.
/// * `stringified_uuid` - UUID of shoppingcart item as String.
///
/// Specifies options with projection.
pub async fn query_shoppingcart_item(
    collection: &Collection<ShoppingCart>,
    id: Uuid,
) -> Result<ShoppingCartItem> {
    let find_options = FindOneOptions::builder()
        .projection(Some(doc! {
            "internal_shoppingcart_items.$": 1,
            "_id": 0
        }))
        .build();
    let projected_collection = collection.clone_with_type::<ProjectedShoppingCart>();
    let message = format!("ShoppingCartItem with UUID id: `{}` not found.", id);
    match projected_collection
        .find_one(
            doc! {"internal_shoppingcart_items": {
                "$elemMatch": {
                    "id": id
                }
            }},
            Some(find_options),
        )
        .await
    {
        Ok(maybe_shoppingcart_projection) => maybe_shoppingcart_projection
            .and_then(|projection| projection.internal_shoppingcart_items.first().cloned())
            .ok_or_else(|| Error::new(message.clone())),
        Err(_) => Err(Error::new(message)),
    }
}

/// Shared function to query a shoppingcart item from a MongoDB collection of shoppingcarts
///
/// * `connection` - MongoDB database connection.
/// * `stringified_uuid` - UUID of shoppingcart item as String.
///
/// Specifies options with projection.
pub async fn query_shoppingcart_item_by_product_variant_id_and_shopping_cart(
    collection: &Collection<ShoppingCart>,
    product_variant_id: Uuid,
    shopping_cart_id: Uuid,
) -> Result<ShoppingCartItem> {
    let find_options = FindOneOptions::builder()
        .projection(Some(doc! {
            "internal_shoppingcart_items.$": 1,
            "_id": 0
        }))
        .build();
    let projected_collection = collection.clone_with_type::<ProjectedShoppingCart>();
    let message = format!("ShoppingCartItem referencing product variant of UUID: `{}` in shopping cart of UUID: `{}` not found.", product_variant_id, shopping_cart_id);
    match projected_collection
        .find_one(
            doc! {"_id": shopping_cart_id, "internal_shoppingcart_items.product_variant._id": product_variant_id},
            Some(find_options),
        )
        .await
    {
        Ok(maybe_shoppingcart_projection) => maybe_shoppingcart_projection
            .and_then(|projection| projection.internal_shoppingcart_items.first().cloned())
            .ok_or_else(|| Error::new(message.clone())),
        Err(_) => Err(Error::new(message)),
    }
}
