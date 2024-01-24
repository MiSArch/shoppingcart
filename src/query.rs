use crate::{
    base_connection::{BaseConnection, FindResultWrapper},
    order_datatypes::ShoppingCartOrderInput,
    shoppingcart_connection::ShoppingCartConnection,
    shoppingcart_item::ShoppingCartItem,
    ShoppingCart,
};
use async_graphql::{Context, Error, Object, Result};
use bson::Document;
use bson::Uuid;
use mongodb::{
    bson::doc,
    options::{FindOneOptions, FindOptions},
    Collection, Database,
};
use mongodb_cursor_pagination::{error::CursorError, FindResult, PaginatedCursor};
use serde::{Deserialize, Serialize};

/// Describes GraphQL shoppingcart queries.
pub struct Query;

#[Object]
impl Query {
    /// Retrieves all shoppingcarts.
    async fn shoppingcarts<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "Describes that the `first` N shoppingcarts should be retrieved.")]
        first: Option<u32>,
        #[graphql(desc = "Describes how many shoppingcarts should be skipped at the beginning.")]
        skip: Option<u64>,
        #[graphql(desc = "Specifies the order in which shoppingcarts are retrieved.")]
        order_by: Option<ShoppingCartOrderInput>,
    ) -> Result<ShoppingCartConnection> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<ShoppingCart> =
            db_client.collection::<ShoppingCart>("shoppingcarts");
        let shoppingcart_order = order_by.unwrap_or_default();
        let sorting_doc = doc! {shoppingcart_order.field.unwrap_or_default().as_str(): i32::from(shoppingcart_order.direction.unwrap_or_default())};
        let find_options = FindOptions::builder()
            .skip(skip)
            .limit(first.map(|v| i64::from(v)))
            .sort(sorting_doc)
            .build();
        let document_collection = collection.clone_with_type::<Document>();
        let maybe_find_results: Result<FindResult<ShoppingCart>, CursorError> =
            PaginatedCursor::new(Some(find_options.clone()), None, None)
                .find(&document_collection, None)
                .await;
        match maybe_find_results {
            Ok(find_results) => {
                let find_result_wrapper = FindResultWrapper(find_results);
                let connection = Into::<BaseConnection<ShoppingCart>>::into(find_result_wrapper);
                Ok(Into::<ShoppingCartConnection>::into(connection))
            }
            Err(_) => return Err(Error::new("Retrieving shoppingcarts failed in MongoDB.")),
        }
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
