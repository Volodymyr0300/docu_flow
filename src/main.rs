use axum::{
    extract::{State, Path},
    routing::{get, post, delete, patch},
    Json, Router,
    http::StatusCode,
};
use clap::Parser;
use serde::{Serialize, Deserialize};
use sqlx::sqlite::{SqlitePool, /*SqlitePoolOptions*/};
use std::sync::Arc;
use tower_http::services::ServeDir;
use std::fmt;
use chrono::{DateTime, Utc};

struct AppState {
    db: SqlitePool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let port = std::env::var("PORT").unwrap_or_else(|_| args.port.to_string());

    let db_pool = SqlitePool::connect("sqlite://documents.db?mode=rwc")
        .await
        .expect("Could not connect to the database");

    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY, title TEXT NOT NULL, status TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP, created_by TEXT
         );
         CREATE TABLE IF NOT EXISTS audit_logs (
            id INTEGER PRIMARY KEY, doc_id INTEGER, action TEXT, user TEXT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
         );"
    )
        .execute(&db_pool)
        .await
        .expect("Could not create tables");

    let _ = sqlx::query("INSERT OR IGNORE INTO users (username, password) VALUES (?, ?)")
        .bind("admin")
        .bind("password123")
        .execute(&db_pool)
        .await;

    let shared_state = Arc::new(AppState {
        db: db_pool
    });

    let app = Router::new()
        .route("/login", post(login_user))
        .route("/register", post(register_user))
        .route("/docs", get(list_docs))
        .route("/docs", post(create_doc))
        .route("/docs/{id}", delete(delete_doc))
        .route("/docs/{id}/status", patch(update_doc_status))
        .route("/docs/{id}/rename", patch(rename_doc))
        .fallback_service(ServeDir::new("static"))
        .with_state(shared_state);

    let address = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    println!("🚀 DocuFlow Server active at http://{}", address);

    axum::serve(listener, app).await.unwrap();
}

async fn list_docs(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<Document>> {
    let docs = sqlx::query_as::<_, Document>(
        "SELECT id, title, status, created_at, created_by FROM documents"
    )
        .fetch_all(&state.db)
        .await
        .expect("❌ Failed to fetch documents");

    Json(docs)
}

async fn create_doc(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateDocument>,
) -> Json<CreateDocument> {
    sqlx::query("INSERT INTO documents (id, title, status, created_by) VALUES (?, ?, ?, ?)")
        .bind(payload.id)
        .bind(&payload.title)
        .bind(&payload.status)
        .bind(&payload.created_by)
        .execute(&state.db) // This is the crucial part!
        .await
        .expect("Failed to insert document");

    println!("Document saved: {}: {}", payload.created_by, payload.title);
    Json(payload)
}

async fn delete_doc(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u32>,
) -> StatusCode {
    sqlx::query("DELETE FROM documents WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .expect("Failed to delete document");

    println!("Deleted document ID: {}", id);
    StatusCode::OK
}

async fn update_doc_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u32>,
    Json(payload): Json<StatusUpdate>,
) -> StatusCode {
    sqlx::query("UPDATE documents SET status = ? WHERE id = ?")
        .bind(&payload.status)
        .bind(id)
        .execute(&state.db)
        .await
        .expect("Failed to update document status");

    sqlx::query("INSERT INTO audit_logs (doc_id, action, user) VALUES (?, ?, ?)")
        .bind(id)
        .bind(format!("Status changed to {:?}", payload.status))
        .bind(&payload.user)
        .execute(&state.db).await.unwrap();

    StatusCode::OK
}

async fn rename_doc(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u32>,
    Json(new_title): Json<String>,
) -> StatusCode {
    sqlx::query("UPDATE documents SET title = ? WHERE id = ?")
        .bind(&new_title)
        .bind(id)
        .execute(&state.db)
        .await
        .expect("Failed to rename document");

    println!("Rename: Document {} is now '{}'", id, new_title);
    StatusCode::OK
}

async fn login_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<AuthResponse>) {
    let user = sqlx::query("SELECT username FROM users WHERE username = ? AND password = ?")
        .bind(&payload.username)
        .bind(&payload.password)
        .fetch_optional(&state.db)
        .await
        .expect("Database error during login");

    if user.is_some() {
        (StatusCode::OK, Json(AuthResponse { success: true, username: payload.username }))
    } else {
        (StatusCode::UNAUTHORIZED, Json(AuthResponse { success: false, username: "".to_string() }))
    }
}

async fn register_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<AuthResponse>) {
    let existing = sqlx::query("SELECT username FROM users WHERE username = ?")
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await
        .unwrap();

    if existing.is_some() {
        return (StatusCode::CONFLICT, Json(AuthResponse { success: false, username: "".into() }));
    }

    let result = sqlx::query("INSERT INTO users (username, password) VALUES (?, ?)")
        .bind(&payload.username)
    .bind(&payload.password)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => (StatusCode::CREATED, Json(AuthResponse { success: true, username: payload.username })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(AuthResponse { success: false, username: "".into() })),
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "3000")]
    port: u16,

    #[arg(short, long)]
    verbose: bool,
}


#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(rename_all = "PascalCase")]
enum DocStatus {
    Draft,
    Reviewed,
    Signed,
}

impl From<DocStatus> for String {
    fn from(status: DocStatus) -> Self {
        format!("{:?}", status)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
struct Document {
    id: u32,
    title: String,
    status: DocStatus,
    created_at: Option<DateTime<Utc>>,
    created_by: Option<String>,
}
#[derive(Deserialize)]
struct StatusUpdate {
    status: DocStatus,
    user: String,
}

#[derive(Serialize, Deserialize)]
struct CreateDocument {
    id: u32,
    title: String,
    status: DocStatus,
    created_by: String,
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DOC #{}: [{}] - Status: {:?}",
            self.id, self.title, self.status
        )
    }
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    success: bool,
    username: String,
}

