use std::collections::HashSet;

use async_graphql::{Context, Error, Object, Result};
use bson::Bson;
use bson::Uuid;
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, DateTime},
    Collection, Database,
};

use crate::mutation_input_structs::AddShoppingCartItemInput;
use crate::mutation_input_structs::UpdateShoppingCartItemInput;
use crate::query::query_shoppingcart_item;
use crate::shoppingcart_item::ShoppingCartItem;
use crate::{
    foreign_types::{ProductVariant, User},
    mutation_input_structs::{AddShoppingCartInput, UpdateShoppingCartInput},
    query::query_shoppingcart,
    shoppingcart::ShoppingCart,
};

/// Describes GraphQL shoppingcart mutations.
pub struct Mutation;

#[Object]
impl Mutation {
    /// Adds a shoppingcart with a user_id, a list of product_variant_ids and a name.
    ///
    /// Formats UUIDs as hyphenated lowercase Strings.
    async fn add_shoppingcart<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "AddShoppingCartInput")] input: AddShoppingCartInput,
    ) -> Result<ShoppingCart> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<ShoppingCart> =
            db_client.collection::<ShoppingCart>("shoppingcarts");
        let product_variant_collection: Collection<ProductVariant> =
            db_client.collection::<ProductVariant>("product_variants");
        validate_shopping_cart_items(&product_variant_collection, &input.shopping_cart_items)
            .await?;
        let current_timestamp = DateTime::now();
        let normalized_shopping_cart_items: HashSet<ShoppingCartItem> = input
            .shopping_cart_items
            .iter()
            .map(|item_input| ShoppingCartItem {
                id: Uuid::new(),
                count: item_input.count,
                added_at: current_timestamp,
                product_variant: ProductVariant {
                    _id: item_input.product_variant_id.clone(),
                },
            })
            .collect();
        let shoppingcart = ShoppingCart {
            _id: Uuid::new(),
            user: User { _id: input.user_id },
            internal_shoppingcart_items: normalized_shopping_cart_items,
            last_updated_at: current_timestamp,
        };
        match collection.insert_one(shoppingcart, None).await {
            Ok(result) => {
                let id = uuid_from_bson(result.inserted_id)?;
                query_shoppingcart(&collection, id).await
            }
            Err(_) => Err(Error::new("Adding shoppingcart failed in MongoDB.")),
        }
    }

    /// Updates shoppingcart_items of a specific shoppingcart referenced with an id.
    ///
    /// Formats UUIDs as hyphenated lowercase Strings.
    async fn update_shoppingcart<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UpdateShoppingCartInput")] input: UpdateShoppingCartInput,
    ) -> Result<ShoppingCart> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<ShoppingCart> =
            db_client.collection::<ShoppingCart>("shoppingcarts");
        let product_variant_collection: Collection<ProductVariant> =
            db_client.collection::<ProductVariant>("product_variants");
        let current_timestamp = DateTime::now();
        update_shopping_cart_items(
            &collection,
            &product_variant_collection,
            &input,
            &current_timestamp,
        )
        .await?;
        let shoppingcart = query_shoppingcart(&collection, input.id).await?;
        Ok(shoppingcart)
    }

    /// Updates a single shoppingcart item.
    ///
    /// * `collection` - MongoDB collection to update.
    /// * `input` - `UpdateShoppingCartItemInput`.
    async fn update_shopping_cart_item<'a>(
        &self,
        ctx: &Context<'a>,
        input: UpdateShoppingCartItemInput,
    ) -> Result<ShoppingCartItem> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<ShoppingCart> =
            db_client.collection::<ShoppingCart>("shoppingcarts");
        if let Err(_) = collection
            .update_one(
                doc! {"internal_shoppingcart_items.id": input.id },
                doc! {"$set": {"count": input.count}},
                None,
            )
            .await
        {
            let message = format!(
                "Updating count of shoppingcart item of id: `{}` failed in MongoDB.",
                input.id
            );
            return Err(Error::new(message));
        }
        let shoppingcart_item = query_shoppingcart_item(&collection, input.id).await?;
        Ok(shoppingcart_item)
    }

    /// Deletes shoppingcart of id.
    async fn delete_shoppingcart<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of shoppingcart to delete.")] id: Uuid,
    ) -> Result<bool> {
        let db_client = ctx.data_unchecked::<Database>();
        let collection: Collection<ShoppingCart> =
            db_client.collection::<ShoppingCart>("shoppingcarts");
        if let Err(_) = collection.delete_one(doc! {"_id": id }, None).await {
            let message = format!("Deleting shoppingcart of id: `{}` failed in MongoDB.", id);
            return Err(Error::new(message));
        }
        Ok(true)
    }
}

/// Extracts UUID from Bson.
///
/// Adding a shoppingcart returns a UUID in a Bson document. This function helps to extract the UUID.
fn uuid_from_bson(bson: Bson) -> Result<Uuid> {
    match bson {
        Bson::Binary(id) => Ok(id.to_uuid()?),
        _ => {
            let message = format!(
                "Returned id: `{}` needs to be a Binary in order to be parsed as a Uuid",
                bson
            );
            Err(Error::new(message))
        }
    }
}

/// Updates shopping cart items of a shoppingcart.
///
/// * `collection` - MongoDB collection to update.
/// * `input` - `UpdateShoppingCartInput`.
async fn update_shopping_cart_items(
    collection: &Collection<ShoppingCart>,
    product_variant_collection: &Collection<ProductVariant>,
    input: &UpdateShoppingCartInput,
    current_timestamp: &DateTime,
) -> Result<()> {
    if let Some(definitely_shopping_cart_items) = &input.shopping_cart_items {
        validate_shopping_cart_items(&product_variant_collection, definitely_shopping_cart_items)
            .await?;
        let normalized_shopping_cart_items: Vec<ShoppingCartItem> = definitely_shopping_cart_items
            .iter()
            .map(|item_input| ShoppingCartItem {
                id: Uuid::new(),
                count: item_input.count,
                added_at: *current_timestamp,
                product_variant: ProductVariant {
                    _id: item_input.product_variant_id.clone(),
                },
            })
            .collect();
        if let Err(_) = collection.update_one(doc!{"_id": input.id }, doc!{"$set": {"internal_shoppingcart_items": normalized_shopping_cart_items, "last_updated_at": current_timestamp}}, None).await {
            let message = format!("Updating product_variant_ids of shoppingcart of id: `{}` failed in MongoDB.", input.id);
            return Err(Error::new(message))
        }
    }
    Ok(())
}

/// Checks if product variants in update shoppingcart item inputs are in the system (MongoDB database populated with events).
///
/// Used before adding or modifying shopping cart items.
async fn validate_shopping_cart_items(
    collection: &Collection<ProductVariant>,
    shoppingcart_items: &HashSet<AddShoppingCartItemInput>,
) -> Result<()> {
    let product_variant_ids_vec: Vec<Uuid> = shoppingcart_items
        .into_iter()
        .map(|item| item.product_variant_id)
        .collect();
    match collection
        .find(doc! {"_id": { "$in": &product_variant_ids_vec } }, None)
        .await
    {
        Ok(cursor) => {
            let product_variants: Vec<ProductVariant> = cursor.try_collect().await?;
            product_variant_ids_vec.iter().fold(Ok(()), |_, p| {
                match product_variants.contains(&ProductVariant { _id: *p }) {
                    true => Ok(()),
                    false => {
                        let message = format!(
                            "Product variant with the UUID: `{}` is not present in the system.",
                            p
                        );
                        Err(Error::new(message))
                    }
                }
            })
        }
        Err(_) => Err(Error::new(
            "Product variants with the specified UUIDs are not present in the system.",
        )),
    }
}
