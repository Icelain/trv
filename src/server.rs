use crate::{controllers, env};
use axum::{
    serve, Router,
    extract::DefaultBodyLimit,

};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub async fn start(opts: env::EnvOptions) {
    let router: Router<()> = Router::new();

    let router = apply_routing(router);
    let router = apply_middleware(router);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", opts.port))
        .await
        .expect("Could not create tcp listener");
    serve(listener, router).await.expect("axum failed");
}

fn apply_middleware(router: Router<()>) -> Router<()> {
    tracing_subscriber::fmt::init();

    let router = router.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::very_permissive())
            .layer(DefaultBodyLimit::max(2147483648))
    );

    return router;
}

fn apply_routing(router: Router<()>) -> Router<()> {
    let router = router
        .route("/", controllers::index())
        .route("/upload", controllers::upload_file());

    return router;
}
