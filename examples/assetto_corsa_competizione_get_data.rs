#[cfg(windows)]
#[tokio::main]
async fn main() {
    use simetry::assetto_corsa_competizione::Client;
    use std::time::Duration;

    let mut client = Client::connect(Duration::from_secs(1)).await;
    println!("{:#?}", client.static_data());
    while let Some(sim_state) = client.next_sim_state().await {
        println!("{:#?}", sim_state);
    }
}

#[cfg(unix)]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("This example only works on Windows")
}
