#[cfg(windows)]
#[tokio::main]
async fn main() {
    use simetry::raceroom_racing_experience::Client;
    use std::time::Duration;

    let mut client = Client::connect(Duration::from_secs(1)).await;
    println!("{:#?}", client.next_sim_state().await);
}

#[cfg(unix)]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("This example only works on Windows")
}
