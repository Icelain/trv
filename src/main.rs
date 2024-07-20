mod controllers;
mod env;
mod process;
mod server;
mod whisper_pool;

#[tokio::main]
async fn main() {
    let opts = env::get_opts();
    server::start(opts).await;
}
