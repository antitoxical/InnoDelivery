use crate::db::DbPool;
use crate::models::Order;
use crate::repository::order_repository::{self, OrderProductData};
use crate::services::order_service::{self, OrderServiceError};
use async_graphql::http::GraphiQLSource;
use async_graphql::{Context, InputObject, Object, Result, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::response::IntoResponse;
use axum::Extension;
use uuid::Uuid;

pub struct QueryRoot;

#[axum::debug_handler]
pub async fn graphql_handler(schema: Extension<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn graphql_playground() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/").finish())
}

#[derive(InputObject)]
pub struct ProductInput {
    pub product_id: Uuid,
    pub quantity: i32,
}

#[Object]
impl QueryRoot {
    async fn order_by_id(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<Order>> {
        let pool: &DbPool = ctx.data()?;
        let mut conn = pool
            .get()
            .map_err(|e| async_graphql::Error::new(format!("Database connection error: {}", e)))?;

        match order_repository::find_order_by_id(&mut conn, id) {
            Ok(order) => Ok(Some(order)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(async_graphql::Error::new(format!(
                "Database query error: {}",
                e
            ))),
        }
    }

    async fn orders_by_user(
        &self,
        ctx: &Context<'_>,
        user_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Order>> {
        let pool: &DbPool = ctx.data()?;
        let mut conn = pool
            .get()
            .map_err(|e| async_graphql::Error::new(format!("Database connection error: {}", e)))?;
        let limit = limit.unwrap_or(20).clamp(1, 100);
        let offset = offset.unwrap_or(0).max(0);
        order_repository::list_orders_by_user(&mut conn, user_id, limit, offset)
            .map_err(|e| async_graphql::Error::new(format!("Database query error: {}", e)))
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_order(
        &self,
        ctx: &Context<'_>,
        user_id: Uuid,
        delivery_address: String,
        products: Vec<ProductInput>,
    ) -> Result<Order> {
        let pool: &DbPool = ctx.data()?;

        if products.is_empty() {
            return Err(async_graphql::Error::new("Product list cannot be empty"));
        }

        let product_data = products
            .into_iter()
            .map(|p| OrderProductData {
                product_id: p.product_id,
                quantity: p.quantity,
            })
            .collect();

        match order_service::create_order(pool, user_id, delivery_address, product_data).await {
            Ok(order) => Ok(order),
            Err(OrderServiceError::DatabaseError(e)) => {
                Err(async_graphql::Error::new(format!("Database error: {}", e)))
            }
            Err(OrderServiceError::UserServiceError(e)) => Err(async_graphql::Error::new(format!(
                "User service error: {}",
                e
            ))),
            Err(OrderServiceError::InvalidInput(e)) => {
                Err(async_graphql::Error::new(format!("Invalid input: {}", e)))
            }
            Err(OrderServiceError::RatingWindowExpired) => {
                Err(async_graphql::Error::new("Rating window expired"))
            }
            Err(OrderServiceError::InvalidRating) => {
                Err(async_graphql::Error::new("Invalid rating"))
            }
        }
    }

    async fn update_order_address(
        &self,
        ctx: &Context<'_>,
        order_id: Uuid,
        delivery_address: String,
    ) -> Result<Order> {
        let pool: &DbPool = ctx.data()?;

        match order_service::update_order_address(pool, order_id, delivery_address).await {
            Ok(order) => Ok(order),

            Err(OrderServiceError::DatabaseError(e)) => {
                Err(async_graphql::Error::new(format!("Database error: {}", e)))
            }
            Err(OrderServiceError::UserServiceError(e)) => Err(async_graphql::Error::new(format!(
                "User service error: {}",
                e
            ))),
            Err(OrderServiceError::InvalidInput(e)) => {
                Err(async_graphql::Error::new(format!("Invalid input: {}", e)))
            }
            Err(OrderServiceError::RatingWindowExpired) => {
                Err(async_graphql::Error::new("Rating window expired"))
            }
            Err(OrderServiceError::InvalidRating) => {
                Err(async_graphql::Error::new("Invalid rating"))
            }
        }
    }

    async fn finish_order(&self, ctx: &Context<'_>, id: Uuid) -> Result<Order> {
        let pool: &DbPool = ctx.data()?;
        match order_service::finish_order(pool, id).await {
            Ok(order) => Ok(order),
            Err(OrderServiceError::DatabaseError(e)) => {
                Err(async_graphql::Error::new(format!("Database error: {}", e)))
            }
            Err(OrderServiceError::UserServiceError(e)) => Err(async_graphql::Error::new(format!(
                "User service error: {}",
                e
            ))),
            Err(OrderServiceError::InvalidInput(e)) => {
                Err(async_graphql::Error::new(format!("Invalid input: {}", e)))
            }
            Err(OrderServiceError::RatingWindowExpired) => {
                Err(async_graphql::Error::new("Rating window expired"))
            }
            Err(OrderServiceError::InvalidRating) => {
                Err(async_graphql::Error::new("Invalid rating"))
            }
        }
    }

    async fn cancel_order(&self, ctx: &Context<'_>, id: Uuid) -> Result<Order> {
        let pool: &DbPool = ctx.data()?;
        match order_service::cancel_order_wrapper(pool, id).await {
            Ok(order) => Ok(order),
            Err(OrderServiceError::DatabaseError(e)) => {
                Err(async_graphql::Error::new(format!("Database error: {}", e)))
            }
            Err(OrderServiceError::UserServiceError(e)) => Err(async_graphql::Error::new(format!(
                "User service error: {}",
                e
            ))),
            Err(OrderServiceError::InvalidInput(e)) => {
                Err(async_graphql::Error::new(format!("Invalid input: {}", e)))
            }
            Err(OrderServiceError::RatingWindowExpired) => {
                Err(async_graphql::Error::new("Rating window expired"))
            }
            Err(OrderServiceError::InvalidRating) => {
                Err(async_graphql::Error::new("Invalid rating"))
            }
        }
    }

    async fn rate_order(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        rating: f32,
        user_id: Uuid,
        rating_window_minutes: Option<i32>,
    ) -> Result<Order> {
        let pool: &DbPool = ctx.data()?;

        let rating_window = rating_window_minutes.unwrap_or(1440);

        match order_service::rate_order(pool, id, user_id, rating, rating_window).await {
            Ok(order) => Ok(order),
            Err(OrderServiceError::DatabaseError(e)) => {
                Err(async_graphql::Error::new(format!("Database error: {}", e)))
            }
            Err(OrderServiceError::UserServiceError(e)) => Err(async_graphql::Error::new(format!(
                "User service error: {}",
                e
            ))),
            Err(OrderServiceError::InvalidInput(e)) => {
                Err(async_graphql::Error::new(format!("Invalid input: {}", e)))
            }
            Err(OrderServiceError::RatingWindowExpired) => {
                Err(async_graphql::Error::new("Rating window expired"))
            }
            Err(OrderServiceError::InvalidRating) => {
                Err(async_graphql::Error::new("Invalid rating"))
            }
        }
    }
}

pub type AppSchema = Schema<QueryRoot, MutationRoot, async_graphql::EmptySubscription>;
