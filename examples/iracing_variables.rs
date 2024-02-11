#[cfg(windows)]
#[tokio::main]
async fn main() {
    use simetry::iracing::Client;
    use std::time::Duration;

    let mut client = Client::connect(Duration::from_secs(1)).await;
    let sim_state = client.next_sim_state().await.unwrap();
    let variables = sim_state.variables();
    println!("{variables:#?}");
}

#[cfg(unix)]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("This example only works on Windows")
}
