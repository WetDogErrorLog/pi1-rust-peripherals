use nexigo_lib;
use image_handling;

// This edition just captures a single image,sends it, then exits.
#[tokio::main]
async fn main() {
    let example_session_config = image_handling::TimelapseSessionConfig{
        service_addr: String::from("192.168.4.22:3000"),
    };
    let example_loop_config = image_handling::TimelapseLoopConfig{
        device_path: String::from("/dev/video0"),
        width: 1280 as u32,
        height: 720 as u32,
        project_folder: String::from("config_tests"),
        file_name_root: String::from("basic_loop"),
        image_format: image_handling::ImageFormat::YUYV,
        interval_minutes: 1,
    };
    // TODO: when I test the configs, I can run two loops at different resolutions,
    // for an easy test case without adding more cameras.
    image_handling::camera_timelapse_loop(
        example_session_config.service_addr,
        example_loop_config,
    ).await;
}
