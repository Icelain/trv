mod controllers;
mod env;
mod process;
mod server;
mod startup;
mod whisper_pool;

#[tokio::main]
async fn main() {
    let opts = env::get_opts();

    startup::check_and_fix(&opts);
    server::start(&opts).await;
}
