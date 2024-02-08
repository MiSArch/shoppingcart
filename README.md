# ShoppingCart service for MiSArch

### Quickstart (DevContainer)

1. Open VSCode Development Container
2. `cargo run` starts the GraphQL service + GraphiQL on port `8080`

### Quickstart (Docker Compose)

1. `docker compose -f docker-compose-dev.yaml up --build` in the repository root directory. **IMPORTANT:** MongoDB credentials should be configured for production.

### What it can do

- CRUD shoppingcarts (`ShoppingCart` is directly attached to user with UUID, therefore does not need its own UUID):

  ```rust
  pub struct ShoppingCart {
      pub shopping_cart_items: HashSet<ShoppingCartItem>,
      pub last_updated_at: DateTime,
  }

  pub struct ShoppingCartItem {
    pub _id: Uuid,
    pub count: u32,
    pub added_at: DateTime,
    pub product_variant: ProductVariant,
  }

  /// Foreign ProductVariant
  pub struct ProductVariant{
      id: Uuid
  }
  ```

- Validates all UUIDs input as strings
- Error prop to GraphQL
