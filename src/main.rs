use axum::{
    extract::State,
    routing::{get, post},
    Json,
    Router
};
use clap::Parser;
use serde::{Serialize, Deserialize};
use sqlx::sqlite::SqlitePool;
use std::fmt;
use std::sync::Arc;
use tower_http::services::ServeDir;

struct AppState {
    db: SqlitePool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let db_pool = SqlitePool::connect("sqlite://documents.db?mode=rwc")
        .await
        .expect("❌ Could not connect to the database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            status TEXT NOT NULL
        )"
    )
        .execute(&db_pool)
        .await
        .expect("❌ Could not create table");

    let shared_state = Arc::new(AppState {
        db: db_pool
    });

    let app = Router::new()
        .route("/docs", get(list_docs))
        .route("/docs", post(create_doc))
        .fallback_service(ServeDir::new("static"))
        .with_state(shared_state);

    let address = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    println!("🚀 DocuFlow Server active at http://{}", address);

    axum::serve(listener, app).await.unwrap();
}

async fn list_docs(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<Document>> {
    let docs = sqlx::query_as::<_, Document>("SELECT id, title, status FROM documents")
        .fetch_all(&state.db)
        .await
        .expect("❌ Failed to fetch documents");

    Json(docs)
}

// async fn root() -> &'static str {
//     "Welcome to DocuFlow: Legal Audit Log System"
// }

async fn create_doc(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Document>,
) -> Json<Document> {
    sqlx::query("INSERT INTO documents (id, title, status) VALUES (?, ?, ?)")
        .bind(payload.id)
        .bind(&payload.title)
        .bind(&payload.status)
        .execute(&state.db) // This is the crucial part!
        .await
        .expect("❌ Failed to insert document");

    println!("✅ Document saved: {}", payload.title);
    Json(payload)
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
}


impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "📄 DOC #{}: [{}] - Status: {:?}",
            self.id, self.title, self.status
        )
    }
}


