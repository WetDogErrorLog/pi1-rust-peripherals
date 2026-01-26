use nexigo_lib;
use image_handling;

const CURRENT_FORMAT: image_handling::ImageFormat = image_handling::ImageFormat::YUYV;

// This edition just captures a single image,sends it, then exits.
fn main() {
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
    let camera = nexigo_lib::Camera::new(
        example_loop_config.width,
        example_loop_config.height,
    );
    let yuyv_result = camera.take_picture(example_loop_config.device_path.clone());
    let yuyv_shot = yuyv_result.expect("Failed to take picture");
    println!("Hello, world! We have a picture");
    image_handling::send_image(
       example_session_config.service_addr,
       yuyv_shot,
       example_loop_config,
    );
}
