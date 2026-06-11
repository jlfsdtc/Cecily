use axum::Router;
use axum::middleware;
use kylin_api::{AppState, create_router, auth::auth_middleware};
use kylin_common::config::KylinConfig;
use kylin_metadata::{SqliteMetadataStore, MetadataStore};
use kylin_job::JobStore;
use kylin_query::QueryExecutor;
use kylin_storage::LocalStorageProvider;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load configuration
    let config = KylinConfig::from_env();
    tracing::info!("Starting Kylin server on {}:{}", config.server_host, config.server_port);

    // Initialize metadata store
    let metadata_store: Arc<dyn MetadataStore> = if config.metadata_db_url.starts_with("sqlite") {
        let store = SqliteMetadataStore::new(&config.metadata_db_url).await?;
        store.run_migrations().await?;
        Arc::new(store)
    } else {
        // For PostgreSQL, use PostgresMetadataStore
        // let store = PostgresMetadataStore::new(&config.metadata_db_url).await?;
        // store.run_migrations().await?;
        // Arc::new(store)
        anyhow::bail!("PostgreSQL not yet supported. Use sqlite:kylin.db");
    };

    // Initialize storage provider
    let storage = Arc::new(LocalStorageProvider::new(config.data_dir.clone()));

    // Initialize query executor
    let query_executor = Arc::new(QueryExecutor::new(metadata_store.clone(), storage));

    // Initialize job store (using in-memory store for now)
    let job_store = Arc::new(kylin_job::store::InMemoryJobStore::new());

    // Create application state
    let state = AppState {
        metadata_store: metadata_store.clone(),
        query_executor,
        job_store,
    };

    // Create API router
    let api_router = create_router(state)
        .layer(middleware::from_fn(auth_middleware));

    // Create main router with static file serving
    let app = Router::new()
        .merge(api_router)
        .nest_service("/", ServeDir::new("kystudio/dist"))
        .layer(CorsLayer::permissive());

    // Start server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);
    tracing::info!("API available at http://{}/api/", addr);
    tracing::info!("Frontend available at http://{}/", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
