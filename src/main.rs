use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use sqlx::PgPool;
use tower_http::{classify::ServerErrorsFailureClass::StatusCode, cors::CorsLayer};

mod follows;
mod tweets;
mod users;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let app = Router::new()
        .route("/users", get(users::get_all).post(users::create))
        .route("/users/:username", get(users::get_by_username))
        .route("/users/:username/tweets", get(users::get_tweets))
        .route("/tweets", get(tweets::get_all).post(tweets::create))
        .route("/tweets/:id", axum::routing::delete(tweets::delete))
        .route("/tweets/feed/:user_id", get(tweets::get_feed))
        .route("/follows/toggle", post(toggle_follow))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let addr = "0.0.0.0:30000";
    println!("Server running at http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
async fn get_users(State(pool): State<PgPool>) -> Result<Json<users::User>, StatusCode> {
    users::get_all(&pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
async fn get_user(
    State(pool): State<PgPool>,
    Path(username): Path<String>,
) -> Result<Json<users::User>, StatusCode> {
    users::get_by_username(&pool, &username)
        .await
        .map(|user_opt| user_opt.map(Json).ok_or(StatusCode::NOT_FOUND))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
}
async fn create_user(
    State(pool): State<PgPool>,
    Json(input): Json<users::NewUser>,
) -> Result<Json<users::User>, StatusCode> {
    users::create(&pool, input)
        .await
        .map(|u| (StatusCode::CREATED, Json(u)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_tweets(
    State(pool): State<PgPool>,
    Path(username): Path<String>,
) -> Result<Json<Vec<tweets::Tweet>>, StatusCode> {
    tweets::get_by_username(&pool, &username)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_user_tweets(
    State(pool): State<PgPool>,
    Path(username): Path<String>,
) -> Result<Json<Vec<tweets::Tweet>>, StatusCode> {
    tweets::get_by_username(&pool, &username)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_feed(
    State(pool): State<PgPool>,
    Json(input): Json<tweets::CreateTweet>,
) -> Result<(StatusCode, Json<tweets::Tweet>), StatusCode> {
    tweets::create(&pool, input)
        .await
        .map(|t| (StatusCode::CREATED, Json(t)))
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn delete_tweet(
    State(pool): State<PgPool>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, StatusCode> {
    tweets::delete(&pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|_| StatusCode::NO_CONTENT)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn toggle_follow(
    State(pool): State<PgPool>,
    Json(input): Json<follows::ToggleFollow>,
) -> Result<Json<follows::FollowResult>, StatusCode> {
    follows::toggle(&pool, input)
        .await
        .map(|is_following| Json(follows::FollowResult { is_following }))
        .map_err(|_| StatusCode::BAD_REQUEST)
}
