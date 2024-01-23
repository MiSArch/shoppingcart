# ShoppingCart service for MiSArch

### Quickstart (DevContainer)

1. Open VSCode Development Container
2. `cargo run` starts the GraphQL service + GraphiQL on port `8080`

### Quickstart (Docker Compose)

1. `docker compose -f docker-compose-dev.yaml up --build` in the repository root directory. **IMPORTANT:** MongoDB credentials should be configured for production.

### What it can do

- CRUD shoppingcarts:

  ```rust
  pub struct ShoppingCart {
      pub id: Uuid,
      pub user_id: Uuid,
      pub product_variants: HashSet<ProductVariant>,
      pub name: String,
      pub created_at: DateTime,
      pub last_updated_at: DateTime,
  }

  /// Foreign ProductVariant
  pub struct ProductVariant{
      id: Uuid
  }
  ```

- Validates all UUIDs input as strings
- Error prop to GraphQL
