use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::env;
use tokio::net::TcpListener;

// - GET  /        -> healthcheck/приветствие
// - GET  /items   -> список items из БД
// - POST /items   -> создать item

#[derive(Serialize, FromRow)]
struct Item {
    id: i32,
    name: String,
    description: String,
}
#[derive(Deserialize)]
struct RequestItem {
    name: String,
    description: String,
}

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

async fn root() -> &'static str {
    "Hello from Virus Studio :_+"
}

// HTTP 500.
fn internal_error<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

// возвращает все строки из `items`.
async fn list_items(
    State(state): State<AppState>,
) -> Result<Json<Vec<Item>>, (StatusCode, String)> {
    let items = sqlx::query_as::<_, Item>(
        r#"
        SELECT id, name, descriptions AS description
        FROM items
        ORDER BY id
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(Json(items))
}

// создаём новую строку в `items` и возвращает созданный объект.
async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<RequestItem>,
) -> Result<(StatusCode, Json<Item>), (StatusCode, String)> {
    let item = sqlx::query_as::<_, Item>(
        r#"
        INSERT INTO items (name, descriptions)
        VALUES ($1, $2)
        RETURNING id, name, descriptions AS description
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(item)))
}

#[tokio::main]
async fn main() {
    // загружает переменные из `.env` в окружение процесса (если файл есть).
    dotenvy::dotenv().ok();

    // стандартная переменная для Postgre
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    let db = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");
    
    let state = AppState { db };
    
    // роуты + состояние приложения.
    let app = Router::new()
        .route("/", get(root))
        .route("/items", get(list_items).post(create_item))
        .with_state(state);

    // Слушаем локально порт 3000.
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}