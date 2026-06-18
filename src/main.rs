mod db;
mod routes;
mod vdf;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect_with(
            "sqlite:vdp.db"
                .parse::<sqlx::sqlite::SqliteConnectOptions>()
                .unwrap()
                .create_if_missing(true),
        )
        .await
        .expect("failed to open database");

    db::init(&pool).await;
    seed_demos(&pool).await;

    let app = Router::new()
        .route("/", get(routes::board))
        .route("/playground", get(routes::playground_form).post(routes::playground_run))
        .route("/submit", get(routes::submit_form).post(routes::submit_post))
        .route("/verify/:id", get(routes::verify_message))
        .route("/about", get(routes::about_page))
        .route("/source", get(routes::source_page))
        .route("/guide", get(routes::guide_page))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind");
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.expect("server error");
}

async fn seed_demos(pool: &SqlitePool) {
    if db::count(pool).await > 0 {
        return;
    }

    // Valid demo: low T, legitimate proof
    println!("Seeding demo proofs — please wait...");
    let proof = tokio::task::spawn_blocking(|| vdf::generate("Hello World", vdf::DEFAULT_T))
        .await
        .expect("spawn failed")
        .expect("VDF failed");
    db::insert(pool, "Hello World", &hex::encode(&proof), vdf::DEFAULT_T, true, true).await;

    // Invalid demo: high T, fabricated proof
    db::insert(
        pool,
        "I am the divine. Hear me.",
        "deadbeefcafebabe0000000000000000deadbeefcafebabe0000000000000000deadbeefcafebabe0000000000000000deadbeefcafebabe0000000000000000",
        1_000_000,
        true,
        false,
    )
    .await;

    println!("Seeded.");
}
