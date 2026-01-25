use nexigo_lib;
use image_handling;

const CURRENT_FORMAT: image_handling::ImageFormat = image_handling::ImageFormat::YUYV;

// This edition just captures a single image,sends it, then exits.
fn main() {
    let camera = nexigo_lib::Camera::new(
        1280 as u32,
        720 as u32,
    );
    let dev_path = "/dev/video0";
    let yuyv_result = camera.take_picture(dev_path.to_string());
    let yuyv_shot = yuyv_result.expect("Failed to take picture");
    println!("Hello, world! We have a picture");
    image_handling::send_image(
       1280 as u32,
       720 as u32,
       CURRENT_FORMAT,
       yuyv_shot,
       String::from("test_shot"),
       String::from("192.168.4.22:3000"),
    );
}
