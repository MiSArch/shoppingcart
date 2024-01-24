use async_graphql::{ComplexObject, Context, Error, Result, SimpleObject};
use bson::{doc, Document, Uuid};
use mongodb::{options::FindOptions, Collection, Database};
use mongodb_cursor_pagination::{error::CursorError, FindResult, PaginatedCursor};
use serde::{Deserialize, Serialize};

use crate::{
    base_connection::{BaseConnection, FindResultWrapper},
    order_datatypes::CommonOrderInput,
    shoppingcart::ShoppingCart,
    shoppingcart_connection::ShoppingCartConnection,
};

/// Type of a user owning shoppingcarts.
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone, SimpleObject)]
#[graphql(complex)]
pub struct User {
    /// UUID of the user.
    pub _id: Uuid,
}

#[ComplexObject]
impl User {
    /// Retrieves shoppingcarts of user.
    async fn shoppingcarts<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "Describes that the `first` N shoppingcarts should be retrieved.")]
        first: Option<u32>,
        #[graphql(desc = "Describes how many shoppingcarts should be skipped at the beginning.")]
        skip: Option<u64>,
        #[graphql(desc = "Specifies the order in which shoppingcarts are retrieved.")]
        order_by: Option<CommonOrderInput>,
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
        let filter = doc! {"user._id": self._id};
        let maybe_find_results: Result<FindResult<ShoppingCart>, CursorError> =
            PaginatedCursor::new(Some(find_options.clone()), None, None)
                .find(&document_collection, Some(&filter))
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
}
