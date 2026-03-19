mod app;
mod routes;

use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = app::build_app().await?;

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("cornerback running at http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;

    Ok(())
}
