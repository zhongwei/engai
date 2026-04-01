pub mod api;
pub mod tui;

pub async fn run_tui(server_url: &str) -> anyhow::Result<()> {
    let client = api::ApiClient::new(server_url.to_string());
    tui::run(client).await
}
