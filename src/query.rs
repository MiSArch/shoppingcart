use crate::{
    base_connection::{BaseConnection, FindResultWrapper},
    order_datatypes::ShoppingCartOrderInput,
    shoppingcart_connection::ShoppingCartConnection,
    ShoppingCart,
};
use async_graphql::{Context, Error, Object, Result};
use bson::Document;
use bson::Uuid;
use mongodb::{bson::doc, options::FindOptions, Collection, Database};
use mongodb_cursor_pagination::{error::CursorError, FindResult, PaginatedCursor};

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
