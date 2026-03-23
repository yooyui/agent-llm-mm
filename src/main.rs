use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    agent_llm_mm::support::tracing::init_tracing();
    Ok(())
}
