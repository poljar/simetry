#[cfg(windows)]
#[tokio::main]
async fn main() {
    use simetry::assetto_corsa::Client;
    use std::time::Duration;

    let mut client = Client::connect(Duration::from_secs(1)).await;
    while let Some(sim_state) = client.next_sim_state().await {
        println!(
            "{} km/h @ {} RPM",
            sim_state.physics.speed_kmh, sim_state.physics.rpm,
        );
    }
}

#[cfg(unix)]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("This example only works on Windows")
}
