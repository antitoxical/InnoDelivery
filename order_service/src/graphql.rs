use crate::db::DbPool;
use crate::models::Order;
use crate::repository::order_repository::{self, OrderProductData};
use crate::services::order_service::{self, OrderServiceError};
use async_graphql::http::GraphiQLSource;
use async_graphql::{Context, ErrorExtensions, FieldError, InputObject, Object, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::response::IntoResponse;
use axum::Extension;
use diesel::r2d2;
use diesel::result::Error as DieselError;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum GraphQLError {
    DatabaseError(String),
    ConnectionError(String),
    NotFound,
    InvalidInput(String),
    ServiceError(OrderServiceError),
}

impl fmt::Display for GraphQLError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GraphQLError::DatabaseError(e) => write!(f, "Database error: {}", e),
            GraphQLError::ConnectionError(e) => write!(f, "Connection error: {}", e),
            GraphQLError::NotFound => write!(f, "Resource not found"),
            GraphQLError::InvalidInput(e) => write!(f, "Invalid input: {}", e),
            GraphQLError::ServiceError(e) => write!(f, "Service error: {}", e),
        }
    }
}

impl From<DieselError> for GraphQLError {
    fn from(err: DieselError) -> GraphQLError {
        match err {
            DieselError::NotFound => GraphQLError::NotFound,
            _ => GraphQLError::DatabaseError(err.to_string()),
        }
    }
}

impl From<r2d2::Error> for GraphQLError {
    fn from(err: r2d2::Error) -> GraphQLError {
        GraphQLError::ConnectionError(err.to_string())
    }
}
impl From<OrderServiceError> for GraphQLError {
    fn from(err: OrderServiceError) -> GraphQLError {
        GraphQLError::ServiceError(err)
    }
}

impl From<async_graphql::Error> for GraphQLError {
    fn from(err: async_graphql::Error) -> GraphQLError {
        GraphQLError::InvalidInput(format!("{:?}", err))
    }
}

impl ErrorExtensions for GraphQLError {
    fn extend(&self) -> FieldError {
        let (code, message) = match self {
            GraphQLError::DatabaseError(s) => ("DATABASE_ERROR", s.clone()),
            GraphQLError::ConnectionError(s) => ("DB_CONNECTION_ERROR", s.clone()),
            GraphQLError::NotFound => ("NOT_FOUND", "Resource not found".to_string()),
            GraphQLError::InvalidInput(s) => ("INVALID_INPUT", s.clone()),

            GraphQLError::ServiceError(OrderServiceError::InvalidInput(s)) => {
                ("INVALID_INPUT", s.clone())
            }
            GraphQLError::ServiceError(OrderServiceError::RatingWindowExpired) => (
                "RATING_WINDOW_EXPIRED",
                "Rating window has expired".to_string(),
            ),
            GraphQLError::ServiceError(OrderServiceError::InvalidRating) => {
                ("INVALID_RATING", "Invalid rating value".to_string())
            }
            GraphQLError::ServiceError(e) => ("SERVICE_ERROR", e.to_string()),
        };

        self.extend_with(|_err, e| {
            e.set("code", code);
            e.set("details", message);
        })
    }
}

type GraphQLResult<T> = std::result::Result<T, GraphQLError>;

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
    async fn order_by_id(&self, ctx: &Context<'_>, id: Uuid) -> GraphQLResult<Option<Order>> {
        let pool: &DbPool = ctx.data()?;
        let mut conn = pool
            .get()
            .map_err(|e| GraphQLError::ConnectionError(e.to_string()))?;

        match order_repository::find_order_by_id(&mut conn, id) {
            Ok(order) => Ok(Some(order)),
            Err(DieselError::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn orders_by_user(
        &self,
        ctx: &Context<'_>,
        user_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> GraphQLResult<Vec<Order>> {
        let pool: &DbPool = ctx.data()?;
        let mut conn = pool
            .get()
            .map_err(|e| GraphQLError::ConnectionError(e.to_string()))?;
        let limit = limit.unwrap_or(20).clamp(1, 100);
        let offset = offset.unwrap_or(0).max(0);

        let orders = order_repository::list_orders_by_user(&mut conn, user_id, limit, offset)?;
        Ok(orders)
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
    ) -> GraphQLResult<Order> {
        let pool: &DbPool = ctx.data()?;

        if products.is_empty() {
            return Err(GraphQLError::InvalidInput(
                "Product list cannot be empty".to_string(),
            ));
        }

        let product_data = products
            .into_iter()
            .map(|p| OrderProductData {
                product_id: p.product_id,
                quantity: p.quantity,
            })
            .collect();

        let user_service_url: &str = ctx.data::<String>()?.as_str();

        let order = order_service::create_order(
            pool,
            user_service_url,
            user_id,
            &delivery_address,
            product_data,
        )
        .await?;
        Ok(order)
    }

    async fn update_order_address(
        &self,
        ctx: &Context<'_>,
        order_id: Uuid,
        delivery_address: String,
    ) -> GraphQLResult<Order> {
        let pool: &DbPool = ctx.data()?;

        let order = order_service::update_order_address(pool, order_id, &delivery_address).await?;
        Ok(order)
    }

    async fn complete_order(&self, ctx: &Context<'_>, id: Uuid) -> GraphQLResult<Order> {
        let pool: &DbPool = ctx.data()?;
        let user_service_url: &str = ctx.data::<String>()?.as_str();

        let order = order_service::complete_order(pool, user_service_url, id).await?;
        Ok(order)
    }

    async fn cancel_order(&self, ctx: &Context<'_>, id: Uuid) -> GraphQLResult<Order> {
        let pool: &DbPool = ctx.data()?;
        let order = order_service::cancel_order_wrapper(pool, id).await?;
        Ok(order)
    }

    async fn rate_order(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        rating: f32,
        user_id: Uuid,
        rating_window_minutes: Option<i32>,
    ) -> GraphQLResult<Order> {
        let pool: &DbPool = ctx.data()?;

        let rating_window = rating_window_minutes.unwrap_or(1440);

        let user_service_url: &str = ctx.data::<String>()?.as_str();

        let order =
            order_service::rate_order(pool, user_service_url, id, user_id, rating, rating_window)
                .await?;
        Ok(order)
    }
}

pub type AppSchema = Schema<QueryRoot, MutationRoot, async_graphql::EmptySubscription>;
