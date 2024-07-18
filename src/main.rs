mod controllers;
mod env;
mod server;
mod process;

#[tokio::main]
async fn main() {
    let opts = env::get_opts();
    server::start(opts).await;
}
