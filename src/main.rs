use axum::{routing::get, Json, Router};
use std::fmt;
use serde::Serialize;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let app = Router::new()
        .route("/", get(root))
        .route("/doc", get(get_doc));

    let address = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();

    println!("🚀 DocuFlow Server active at http://{}", address);

    axum::serve(listener, app).await.unwrap();
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


#[derive(Debug, Serialize)]
enum DocStatus {
    Draft,
    Reviewed,
    Signed
}

#[derive(Debug, Serialize)]
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

// fn main() {
//     let args = Args::parse();
//
//     println!("Starting DocuFlow on port: {}", args.port);
//
//     if args.verbose {
//         println!("Verbose mode is On. Looking for files...");
//     }
// }
