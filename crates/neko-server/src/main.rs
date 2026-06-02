#[tokio::main]
async fn main() -> anyhow::Result<()> {
    neko_server::run_from_config().await
}
