use image_handling;
use serde_json;

// This edition just captures a single image,sends it, then exits.
#[tokio::main]
async fn main() {
    let config_data = std::fs::read_to_string("./sample_config.json")
        .expect("failed to read config to string");
    let session_config: image_handling::TimelapseSessionConfig = serde_json::from_str(&config_data)
        .expect("failed to unpack session config json");

    let mut task_set = tokio::task::JoinSet::new();

    for loop_config in session_config.loops {
        let service_addr = session_config.service_addr.clone();
        task_set.spawn(async move {
            image_handling::camera_timelapse_loop(
                service_addr,
                loop_config,
            ).await;
        });
    }

    while let Some(res) = task_set.join_next().await {
        if let Err(e) = res {
            eprintln!("Failed Camera Error: {:?}", e)
        }
    }
}
