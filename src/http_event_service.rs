use axum::{debug_handler, extract::State, http::StatusCode, Json};
use bson::{doc, Uuid};
use log::info;
use mongodb::Collection;
use serde::{Deserialize, Serialize};

use crate::{foreign_types::ProductVariant, shoppingcart::ShoppingCart, user::User};

/// Data to send to Dapr in order to describe a subscription.
#[derive(Serialize)]
pub struct Pubsub {
    #[serde(rename(serialize = "pubsubName"))]
    pub pubsubname: String,
    pub topic: String,
    pub route: String,
}

/// Reponse data to send to Dapr when receiving an event.
#[derive(Serialize)]
pub struct TopicEventResponse {
    pub status: u8,
}

/// Default status is `0` -> Ok, according to Dapr specs.
impl Default for TopicEventResponse {
    fn default() -> Self {
        Self { status: 0 }
    }
}


/// Relevant part of Dapr event wrapped in a CloudEnvelope.
#[derive(Deserialize, Debug)]
pub struct Event<T> {
    pub topic: String,
    pub data: T,
}

/// Relevant part of Dapr event.data.
#[derive(Deserialize, Debug)]
pub struct EventData {
    pub id: Uuid,
}


#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderEventData {
    /// Order UUID.
    pub id: Uuid,
    /// UUID of user connected with Order.
    pub user_id: Uuid,
    /// OrderItems associated with the order.
    pub order_items: Vec<OrderItemEventData>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemEventData {
    /// UUID of shopping cart item associated with OrderItem.
    pub shopping_cart_item_id: Uuid,
    /// Specifies the quantity of the OrderItem.
    pub count: u64,
}

/// Service state containing database connections.
#[derive(Clone)]
pub struct HttpEventServiceState {
    pub product_variant_collection: Collection<ProductVariant>,
    pub user_collection: Collection<User>,
}

/// HTTP endpoint to list topic subsciptions.
pub async fn list_topic_subscriptions() -> Result<Json<Vec<Pubsub>>, StatusCode> {
    let pubsub_user = Pubsub {
        pubsubname: "pubsub".to_string(),
        topic: "user/user/created".to_string(),
        route: "/on-topic-event".to_string(),
    };
    let pubsub_product_variant = Pubsub {
        pubsubname: "pubsub".to_string(),
        topic: "catalog/product-variant/created".to_string(),
        route: "/on-topic-event".to_string(),
    };
    let pubsub_order = Pubsub {
        pubsubname: "pubsub".to_string(),
        topic: "order/order/created".to_string(),
        route: "/on-order-creation-event".to_string(),
    }; 
    Ok(Json(vec![pubsub_user, pubsub_product_variant, pubsub_order]))
}

/// HTTP endpoint to receive events.
#[debug_handler(state = HttpEventServiceState)]
pub async fn on_topic_event(
    State(state): State<HttpEventServiceState>,
    Json(event): Json<Event<EventData>>,
) -> Result<Json<TopicEventResponse>, StatusCode> {
    info!("{:?}", event);

    match event.topic.as_str() {
        "catalog/product-variant/created" => {
            add_product_variant_to_mongodb(state.product_variant_collection, event.data.id).await?
        }
        "user/user/created" => add_user_to_mongodb(state.user_collection, event.data.id).await?,
        _ => {
            // TODO: This message can be used for further Error visibility.
            let _message = format!(
                "Event of topic: `{}` is not a handleable by this service.",
                event.topic.as_str()
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    Ok(Json(TopicEventResponse::default()))
}

/// HTTP endpoint to receive user Order creation events.
#[debug_handler(state = HttpEventServiceState)]
pub async fn on_order_creation_event(
    State(state): State<HttpEventServiceState>,
    Json(event): Json<Event<OrderEventData>>,
) -> Result<Json<TopicEventResponse>, StatusCode> {
    info!("{:?}", event);

    match event.topic.as_str() {
        "order/order/created" => {
            delete_ordered_shoppingcart_items_in_mongodb(&state.user_collection, event.data).await?
        }
        _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
    Ok(Json(TopicEventResponse::default()))
}

pub async fn delete_ordered_shoppingcart_items_in_mongodb(collection: &Collection<User>, order_event_data: OrderEventData) -> Result<(), StatusCode> {
    let shoppingcart_item_ids: Vec<Uuid> = order_event_data.order_items.iter().map(|o| o.shopping_cart_item_id).collect();
    match collection
        .update_one(
            doc! {"_id": order_event_data.user_id },
            doc! {"$pull": {
                "shoppingcart.internal_shoppingcart_items": {
                    "_id": {
                        "$in": shoppingcart_item_ids
                    }
                }
            }},
            None,
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Add a newly created product variant to MongoDB.
pub async fn add_product_variant_to_mongodb(
    collection: Collection<ProductVariant>,
    id: Uuid,
) -> Result<(), StatusCode> {
    let product_variant = ProductVariant { _id: id };
    match collection.insert_one(product_variant, None).await {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Add a newly created user to MongoDB.
pub async fn add_user_to_mongodb(collection: Collection<User>, id: Uuid) -> Result<(), StatusCode> {
    let user = User {
        _id: id,
        shoppingcart: ShoppingCart::new(),
    };
    match collection.insert_one(user, None).await {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
