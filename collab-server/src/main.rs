use collab_server::{AppState, db};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize database
    let db = db::init_db().await?;
    
    let state = AppState { db };

    // Build router
    let app = collab_server::create_app(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    tracing::info!("Server listening on http://0.0.0.0:8000");
    
    axum::serve(listener, app).await?;

    Ok(())
}
