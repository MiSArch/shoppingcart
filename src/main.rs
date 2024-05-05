use std::{env, fs::File, io::Write};

use async_graphql::{
    extensions::Logger, http::GraphiQLSource, EmptySubscription, SDLExportOptions, Schema,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};

use authorization::AuthorizedUserHeader;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{self, IntoResponse},
    routing::{get, post},
    Router, Server,
};
use clap::{arg, command, Parser};
use event::http_event_service::{
    list_topic_subscriptions, on_order_creation_event, on_topic_event, HttpEventServiceState,
};

use log::{info, Level};
use mongodb::{options::ClientOptions, Client, Database};

mod authorization;
mod event;
mod graphql;

use graphql::model::{foreign_types::ProductVariant, user::User};

use crate::graphql::{mutation::Mutation, query::Query};

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

/// Returns Router that establishes connection to Dapr.
///
/// Adds endpoints to define pub/sub interaction with Dapr.
///
/// * `db_client` - MongoDB database client.
async fn build_dapr_router(db_client: Database) -> Router {
    let product_variant_collection: mongodb::Collection<ProductVariant> =
        db_client.collection::<ProductVariant>("product_variants");
    let user_collection: mongodb::Collection<User> = db_client.collection::<User>("users");

    // Define routes.
    let app = Router::new()
        .route("/dapr/subscribe", get(list_topic_subscriptions))
        .route("/on-order-creation-event", post(on_order_creation_event))
        .route("/on-topic-event", post(on_topic_event))
        .with_state(HttpEventServiceState {
            product_variant_collection,
            user_collection,
        });
    app
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
    simple_logger::init_with_level(Level::Warn).unwrap();

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

/// Describes the handler for GraphQL requests.
///
/// Parses the `Authorized-User` header and writes it in the context data of the specfic request.
/// Then executes the GraphQL schema with the request.
///
/// * `schema` - GraphQL schema used by handler.
/// * `headers` - Header map containing headers of request.
/// * `request` - GraphQL request.
async fn graphql_handler(
    State(schema): State<Schema<Query, Mutation, EmptySubscription>>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    if let Ok(authenticate_user_header) = AuthorizedUserHeader::try_from(&headers) {
        req = req.data(authenticate_user_header);
    }
    schema.execute(req).await.into()
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

    let graphiql = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .route("/health", get(StatusCode::OK))
        .with_state(schema);
    let dapr_router = build_dapr_router(db_client).await;
    let app = Router::new().merge(graphiql).merge(dapr_router);

    info!("GraphiQL IDE: http://0.0.0.0:8080");
    Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
