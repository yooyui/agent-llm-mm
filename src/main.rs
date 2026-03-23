use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    agent_llm_mm::run().await
}
