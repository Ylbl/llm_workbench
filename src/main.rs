#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    llm_workbench::init_tracing();

    let config = llm_workbench::AppConfig::from_env()?;
    let state = llm_workbench::AppState::from_config(config).await?;
    llm_workbench::serve(state).await?;

    Ok(())
}
