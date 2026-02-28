mod autotag;
mod server;
mod sync;
mod util;

#[tokio::main]
async fn main() {
    server::serve(util::Configuration::open().unwrap()).await
}
