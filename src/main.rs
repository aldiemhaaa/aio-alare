use std::net::SocketAddr;
use tokio::signal;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use inventory_system::{api::create_router, config::Config, infrastructure::create_pool};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 1. Inisialisasi logging terstruktur (tracing)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,inventory_system=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Memulai Inventory Management System Backend...");

    // 2. Baca konfigurasi lingkungan
    let config = Config::from_env()?;
    info!("Konfigurasi berhasil dimuat untuk environment: {}", config.environment);

    // 3. Inisialisasi Database Connection Pool
    let pool = create_pool(&config.database_url, config.database_max_connections).await?;
    info!("Koneksi PostgreSQL berhasil dibuat ke {}", config.database_url);

    // 4. Inisialisasi Router API
    let app = create_router(pool);

    // 5. Jalankan HTTP Server
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    info!("Server mendengarkan pada http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server berhasil dimatikan dengan aman (Graceful shutdown complete).");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Gagal menginstal handler sinyal Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Gagal menginstal handler sinyal SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Sinyal shutdown (Ctrl+C) diterima."),
        _ = terminate => info!("Sinyal shutdown (SIGTERM) diterima."),
    }
}
