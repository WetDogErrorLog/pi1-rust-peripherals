use nexigo_lib;

fn main() {
    let camera = nexigo_lib::Camera::new(
        1280 as u32,
        720 as u32,
    );
    let dev_path = "/dev/video0";
    let yuvu_shot = camera.take_picture(dev_path.to_string());
    println!("Hello, world!");
}
