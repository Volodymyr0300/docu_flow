use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "3000")]
    port: u16,

    #[arg(short, long)]
    verbose: bool,
}

use std::fmt;
use std::fmt::write;

#[derive(Debug)]
struct Document {
    id: u32,
    title: String,
    status: DocStatus,
}

#[derive(Debug)]
enum DocStatus {
    Draft,
    Reviewed,
    Signed
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

fn main() {
    let args = Args::parse();

    println!("Starting DocuFlow on port: {}", args.port);

    if args.verbose {
        println!("Verbose mode is On. Looking for files...");
    }
}
