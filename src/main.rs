use std::{collections::HashSet, env, fs::File, io::Write};

use async_graphql::{
    extensions::Logger, http::GraphiQLSource, EmptySubscription, SDLExportOptions, Schema,
};
use async_graphql_axum::GraphQL;
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router, Server,
};
use clap::{arg, command, Parser};
use simple_logger::SimpleLogger;

use log::info;
use mongodb::{bson::DateTime, options::ClientOptions, Client, Collection, Database};

use dapr::dapr::dapr::proto::runtime::v1::app_callback_server::AppCallbackServer;
use tonic::transport::Server as TonicServer;

use bson::Uuid;
use shoppingcart::ShoppingCart;

mod shoppingcart;
mod shoppingcart_item;

mod query;
use query::Query;

mod mutation;
use mutation::Mutation;

mod user;
use user::User;

mod app_callback_service;
use app_callback_service::AppCallbackService;

use crate::foreign_types::ProductVariant;

mod base_connection;
mod foreign_types;
mod mutation_input_structs;
mod order_datatypes;
mod shoppingcart_connection;
mod shoppingcart_item_connection;

/// Builds the GraphiQL frontend.
async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
}

/// Establishes database connection and returns the client.
async fn db_connection() -> Client {
    let uri = match env::var_os("MONGODB_URI") {
        Some(uri) => uri.into_string().unwrap(),
        None => panic!("$MONGODB_URI is not set."),
    };

    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse(uri).await.unwrap();

    // Manually set an option.
    client_options.app_name = Some("ShoppingCart".to_string());

    // Get a handle to the deployment.
    Client::with_options(client_options).unwrap()
}

/// Establishes connection to Dapr.
///
/// Adds AppCallbackService which defines pub/sub interaction with Dapr.
async fn dapr_connection(db_client: Database) {
    let addr = "[::]:50051".parse().unwrap();
    let collection: mongodb::Collection<ProductVariant> =
        db_client.collection::<ProductVariant>("product_variants");

    let callback_service = AppCallbackService { collection };
    //callback_service.add_product_variant_to_mongodb(Uuid::parse_str("2df77aa0-fa9e-4d09-a263-ff9047af881e").unwrap()).await.unwrap();

    info!("AppCallback server listening on: {}", addr);
    // Create a gRPC server with the callback_service.
    TonicServer::builder()
        .add_service(AppCallbackServer::new(callback_service))
        .serve(addr)
        .await
        .unwrap();
}

/// Can be used to insert dummy shoppingcart data in the MongoDB database.
#[allow(dead_code)]
async fn insert_dummy_data(collection: &Collection<ShoppingCart>) {
    let shoppingcarts: Vec<ShoppingCart> = vec![ShoppingCart {
        _id: Uuid::new(),
        user: User { _id: Uuid::new() },
        internal_shoppingcart_items: HashSet::new(),
        last_updated_at: DateTime::now(),
    }];
    collection.insert_many(shoppingcarts, None).await.unwrap();
}

/// Command line argument to toggle schema generation instead of service execution.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Generates GraphQL schema in `./schemas/shoppingcart.graphql`.
    #[arg(long)]
    generate_schema: bool,
}

/// Activates logger and parses argument for optional schema generation. Otherwise starts gRPC and GraphQL server.
#[tokio::main]
async fn main() -> std::io::Result<()> {
    SimpleLogger::new().init().unwrap();

    let args = Args::parse();
    if args.generate_schema {
        let schema = Schema::build(Query, Mutation, EmptySubscription).finish();
        let mut file = File::create("./schemas/shoppingcart.graphql")?;
        let sdl_export_options = SDLExportOptions::new().federation();
        let schema_sdl = schema.sdl_with_options(sdl_export_options);
        file.write_all(schema_sdl.as_bytes())?;
        info!("GraphQL schema: ./schemas/shoppingcart.graphql was successfully generated!");
    } else {
        start_service().await;
    }
    Ok(())
}

/// Starts shoppingcart service on port 8000.
async fn start_service() {
    let client = db_connection().await;
    let db_client: Database = client.database("shoppingcart-database");

    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .extension(Logger)
        .data(db_client.clone())
        .enable_federation()
        .finish();

    let app = Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)));

    let t1 = tokio::spawn(async {
        info!("GraphiQL IDE: http://0.0.0.0:8080");
        Server::bind(&"0.0.0.0:8080".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    let t2 = tokio::spawn(async {
        dapr_connection(db_client).await;
    });

    t1.await.unwrap();
    t2.await.unwrap();
}
