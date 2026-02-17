use axum::{extract, routing::{get, post}, Json, Router};
use std::fmt;
use serde::Serialize;
use std::sync::{Arc, RwLock};
use axum::extract::State;

struct AppState {
    docs: RwLock<Vec<Document>>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let shared_state = Arc::new(AppState {
        docs: RwLock::new(Vec::new()),
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/doc", get(list_docs))
        .with_state(shared_state);

    let address = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();

    println!("🚀 DocuFlow Server active at http://{}", address);

    axum::serve(listener, app).await.unwrap();
}

async fn list_docs(State(state): State<Arc<AppState>>, ) -> Json<Vec<Document>> {
    let docs = state.docs.read().unwrap();

    Json(docs.clone())
}

async fn get_doc() -> Json<Document> {
    let doc = Document {
        id: 1,
        title: String::from("Privacy Policy Update"),
        status: DocStatus::Reviewed,
    };

    Json(doc)
}

async fn root() -> &'static str {
    "Welcome to DocuFlow: Legal Audit Log System"
}

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "3000")]
    port: u16,

    #[arg(short, long)]
    verbose: bool,
}


#[derive(Debug, Serialize, Clone)]
enum DocStatus {
    Draft,
    Reviewed,
    Signed
}

#[derive(Debug, Serialize, Clone)]
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


