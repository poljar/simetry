#[cfg(windows)]
#[tokio::main]
async fn main() {
    use simetry::truck_simulator;
    use std::time::Duration;

    let data = truck_simulator::Client::connect(Duration::from_secs(1))
        .await
        .next_sim_state()
        .await
        .unwrap();
    println!("{:#?}", data);
}

#[cfg(unix)]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("This example only works on Windows")
}
