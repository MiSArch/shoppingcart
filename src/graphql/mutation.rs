use std::collections::HashSet;

use async_graphql::{Context, Error, Object, Result};
use bson::Uuid;
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, DateTime},
    Collection, Database,
};

use crate::authorization::authorize_user;

use super::{
    model::{
        foreign_types::ProductVariant, shoppingcart::ShoppingCart,
        shoppingcart_item::ShoppingCartItem, user::User,
    },
    mutation_input_structs::{
        CreateShoppingCartItemInput, ShoppingCartItemInput, UpdateShoppingCartInput,
        UpdateShoppingCartItemInput,
    },
    query::{
        query_object, query_shoppingcart, query_shoppingcart_item,
        query_shoppingcart_item_by_product_variant_id_and_user_id, query_shoppingcart_item_user,
    },
};

/// Describes GraphQL shopping cart mutations.
pub struct Mutation;

#[Object]
impl Mutation {
    /// Updates shopping cart items of a specific shopping cart referenced with a UUID.
    ///
    /// Formats UUIDs as hyphenated lowercase strings.
    async fn update_shoppingcart<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UpdateShoppingCartInput")] input: UpdateShoppingCartInput,
    ) -> Result<ShoppingCart> {
        authorize_user(&ctx, Some(input.id))?;
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<User> = db_client.collection::<User>("users");
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

    /// Adds shopping cart item to a shopping cart.
    ///
    /// Queries for existing item, otherwise adds new shoppingcart item.
    async fn create_shoppingcart_item<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "CreateShoppingCartItemInput")] input: CreateShoppingCartItemInput,
    ) -> Result<ShoppingCartItem> {
        authorize_user(&ctx, Some(input.id))?;
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<User> = db_client.collection::<User>("users");
        let product_variant_collection: Collection<ProductVariant> =
            db_client.collection::<ProductVariant>("product_variants");
        validate_user(&collection, input.id).await?;
        validate_shopping_cart_item(&product_variant_collection, &input.shopping_cart_item).await?;
        match query_shoppingcart_item_by_product_variant_id_and_user_id(
            &collection,
            input.shopping_cart_item.product_variant_id,
            input.id,
        )
        .await
        {
            Ok(shoppingcart_item) => Ok(shoppingcart_item),
            Err(_) => add_shoppingcart_item_to_monogdb(&collection, input).await,
        }
    }

    /// Updates a single shopping cart item.
    async fn update_shoppingcart_item<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UpdateShoppingCartItemInput")] input: UpdateShoppingCartItemInput,
    ) -> Result<ShoppingCartItem> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<User> = db_client.collection::<User>("users");
        let user = query_shoppingcart_item_user(&collection, input.id).await?;
        authorize_user(&ctx, Some(user._id))?;
        if let Err(_) = collection
            .update_one(
                doc! {"shoppingcart.internal_shoppingcart_items._id": input.id },
                doc! {"$set": {"shoppingcart.internal_shoppingcart_items.$.count": input.count}},
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

    /// Deletes shoppingcart item of UUID.
    async fn delete_shoppingcart_item<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of shoppingcart item to delete.")] id: Uuid,
    ) -> Result<bool> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<User> = db_client.collection::<User>("users");
        let user = query_shoppingcart_item_user(&collection, id).await?;
        authorize_user(&ctx, Some(user._id))?;
        if let Err(_) = collection
            .update_one(
                doc! {"shoppingcart.internal_shoppingcart_items._id": id },
                doc! {"$pull": {"shoppingcart.internal_shoppingcart_items": {"_id": id}}},
                None,
            )
            .await
        {
            let message = format!(
                "Deleting shoppingcart item of id: `{}` failed in MongoDB.",
                id
            );
            return Err(Error::new(message));
        }
        Ok(true)
    }
}

/// Updates shopping cart items of a shopping cart.
///
/// * `collection` - MongoDB collection to update.
/// * `product_variant_collection` - MongoDB product variant collection used for product variant validation.
/// * `input` - Update withlist input containing shopping cart items.
/// * `current_timestamp` - Timestamp of product variant ids update.
async fn update_shopping_cart_items(
    collection: &Collection<User>,
    product_variant_collection: &Collection<ProductVariant>,
    input: &UpdateShoppingCartInput,
    current_timestamp: &DateTime,
) -> Result<()> {
    if let Some(definitely_shopping_cart_items) = &input.shopping_cart_items {
        validate_shopping_cart_items(&product_variant_collection, definitely_shopping_cart_items)
            .await?;
        validate_user(&collection, input.id).await?;
        let normalized_shopping_cart_items: Vec<ShoppingCartItem> = definitely_shopping_cart_items
            .iter()
            .map(|item_input| ShoppingCartItem {
                _id: Uuid::new(),
                count: item_input.count,
                added_at: *current_timestamp,
                product_variant: ProductVariant {
                    _id: item_input.product_variant_id,
                },
            })
            .collect();
        if let Err(_) = collection.update_one(doc!{"_id": input.id }, doc!{"$set": {"shoppingcart.internal_shoppingcart_items": normalized_shopping_cart_items, "shoppingcart.last_updated_at": current_timestamp}}, None).await {
            let message = format!("Updating product_variant_ids of shoppingcart of id: `{}` failed in MongoDB.", input.id);
            return Err(Error::new(message))
        }
    }
    Ok(())
}

/// Checks if product variants in shopping cart item inputs are in the system (MongoDB database populated with events).
///
/// Used before adding or modifying shoppingcart items.
///
/// * `collection` - MongoDB collection to validate against.
/// * `shoppingcart_items` - Shopping cart item inputs to validate.
async fn validate_shopping_cart_items(
    collection: &Collection<ProductVariant>,
    shoppingcart_items: &HashSet<ShoppingCartItemInput>,
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
            product_variant_ids_vec.iter().fold(Ok(()), |_, id| {
                match product_variants.contains(&ProductVariant { _id: *id }) {
                    true => Ok(()),
                    false => {
                        let message = format!(
                            "Product variant with the UUID: `{}` is not present in the system.",
                            id
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

/// Adds shopping cart item to MongoDB collection.
///
/// * `collection` - MongoDB collection to add the shopping cart item to.
/// * `input` - Create shopping cart item input containing shopping cart item.
async fn add_shoppingcart_item_to_monogdb(
    collection: &Collection<User>,
    input: CreateShoppingCartItemInput,
) -> Result<ShoppingCartItem> {
    let current_timestamp = DateTime::now();
    let shoppingcart_item = ShoppingCartItem {
        _id: Uuid::new(),
        count: input.shopping_cart_item.count,
        added_at: current_timestamp,
        product_variant: ProductVariant {
            _id: input.shopping_cart_item.product_variant_id,
        },
    };
    if let Err(_) = collection
        .update_one(
            doc! {"_id": input.id },
            doc! {"$push": {"shoppingcart.internal_shoppingcart_items": &shoppingcart_item}},
            None,
        )
        .await
    {
        let message = format!(
            "Add shoppingcart item of id: `{}` failed in MongoDB.",
            shoppingcart_item._id
        );
        return Err(Error::new(message));
    }
    Ok(shoppingcart_item)
}

/// Checks if user is in the system (MongoDB database populated with events).
///
/// * `collection` - MongoDB collection to validate against.
/// * `id` - User UUID to validate.
async fn validate_user(collection: &Collection<User>, id: Uuid) -> Result<()> {
    query_object(&collection, id).await.map(|_| ())
}

/// Checks if product variant in shoppingcart item input is in the system (MongoDB database populated with events).
///
/// Used before adding or modifying shopping cart items.
/// This is a separate function from `validate_shopping_cart_items`, which is designed for only checking one shopping cart items instead of multiple.
///
/// * `collection` - MongoDB collection to validate against.
/// * `shoppingcart_item_input` - Shopping cart item input to validate.
async fn validate_shopping_cart_item(
    collection: &Collection<ProductVariant>,
    shoppingcart_item_input: &ShoppingCartItemInput,
) -> Result<()> {
    let message = format!(
        "Product variant with the UUID: `{}` is not present in the system.",
        shoppingcart_item_input.product_variant_id
    );
    match collection
        .find_one(
            doc! {"_id": shoppingcart_item_input.product_variant_id },
            None,
        )
        .await
    {
        Ok(maybe_product_variant) => match maybe_product_variant {
            Some(_) => Ok(()),
            None => Err(Error::new(message)),
        },
        Err(_) => Err(Error::new(message)),
    }
}
