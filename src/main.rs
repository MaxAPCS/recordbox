mod config;
mod server;
mod sync;
mod tagser;

#[tokio::main]
async fn main() {
    match config::Configuration::open() {
        Ok(configuration) => server::serve(configuration).await,
        Err(e) => eprintln!("{}", e),
    }
}

// TEST SCRIPT curl -X POST http://127.0.0.1:4000/addtrack -d "{\"provider\": \"youtube\", \"id\": \"fXYWY1Q87j8\"}"  -H "Content-Type: application/json" --http2-prior-knowledge
