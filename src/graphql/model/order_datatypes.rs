use async_graphql::{Enum, InputObject, SimpleObject};

/// GraphQL order direction.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum OrderDirection {
    /// Ascending order direction.
    Asc,
    /// Descending order direction.
    Desc,
}

impl Default for OrderDirection {
    fn default() -> Self {
        Self::Asc
    }
}

/// Implements conversion to `i32` for MongoDB document sorting.
impl From<OrderDirection> for i32 {
    fn from(value: OrderDirection) -> Self {
        match value {
            OrderDirection::Asc => 1,
            OrderDirection::Desc => -1,
        }
    }
}

/// Describes the fields that a shoppingcart can be ordered by.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ShoppingCartOrderField {
    /// Orders by "id".
    Id,
    /// Orders by "user_id".
    UserId,
    /// Orders by "name".
    Name,
    /// Orders by "created_at".
    CreatedAt,
    /// Orders by "last_updated_at".
    LastUpdatedAt,
}

impl ShoppingCartOrderField {
    pub fn as_str(&self) -> &'static str {
        match self {
            ShoppingCartOrderField::Id => "_id",
            ShoppingCartOrderField::UserId => "user_id",
            ShoppingCartOrderField::Name => "name",
            ShoppingCartOrderField::CreatedAt => "created_at",
            ShoppingCartOrderField::LastUpdatedAt => "last_updated_at",
        }
    }
}

impl Default for ShoppingCartOrderField {
    fn default() -> Self {
        Self::Id
    }
}

/// Specifies the order of shoppingcarts.
#[derive(SimpleObject, InputObject)]
pub struct ShoppingCartOrderInput {
    /// Order direction of shoppingcarts.
    pub direction: Option<OrderDirection>,
    /// Field that shoppingcarts should be ordered by.
    pub field: Option<ShoppingCartOrderField>,
}

impl Default for ShoppingCartOrderInput {
    fn default() -> Self {
        Self {
            direction: Some(Default::default()),
            field: Some(Default::default()),
        }
    }
}

/// Describes the fields that a foreign types can be ordered by.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum CommonOrderField {
    /// Orders by "id".
    Id,
}

impl CommonOrderField {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommonOrderField::Id => "_id",
        }
    }
}

impl Default for CommonOrderField {
    fn default() -> Self {
        Self::Id
    }
}

/// Specifies the order of foreign types.
#[derive(SimpleObject, InputObject)]
pub struct CommonOrderInput {
    /// Order direction of shoppingcarts.
    pub direction: Option<OrderDirection>,
    /// Field that shoppingcarts should be ordered by.
    pub field: Option<CommonOrderField>,
}

impl Default for CommonOrderInput {
    fn default() -> Self {
        Self {
            direction: Some(Default::default()),
            field: Some(Default::default()),
        }
    }
}
