use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::prelude::*;
use v4l::video::Capture;
use image::{RgbImage, Rgb};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "/dev/video0";
    let mut dev = Device::with_path(path)?;

    let mut fmt = dev.format()?;
    fmt.width = 640;
    fmt.height = 480;
    fmt.fourcc = v4l::FourCC::new(b"YUYV");
    // Important: capture the actual format the camera accepted
    let fmt = dev.set_format(&fmt)?;

    println!("Camera set to {}x{} ({})", fmt.width, fmt.height, fmt.fourcc);

    let mut stream = MmapStream::with_buffers(&mut dev, Type::VideoCapture, 4)?;
    // Use the trait method to get the next frame
    let (data, _metadata) = v4l::io::traits::CaptureStream::next(&mut stream)?;

    println!("YUYV Frame captured! Size: {} bytes", data.len());

    // --- NEW CONVERSION LOGIC ---
    let mut rgb_data = Vec::with_capacity((fmt.width * fmt.height * 3) as usize);

    // YUYV sends 2 pixels in 4 bytes: [Y1, U, Y2, V]
    for chunk in data.chunks_exact(4) {
        let y1 = chunk[0] as f32;
        let u  = chunk[1] as f32 - 128.0;
        let y2 = chunk[2] as f32;
        let v  = chunk[3] as f32 - 128.0;

        // Pixel 1
        rgb_data.push((y1 + 1.402 * v).clamp(0.0, 255.0) as u8);
        rgb_data.push((y1 - 0.344136 * u - 0.714136 * v).clamp(0.0, 255.0) as u8);
        rgb_data.push((y1 + 1.772 * u).clamp(0.0, 255.0) as u8);

        // Pixel 2
        rgb_data.push((y2 + 1.402 * v).clamp(0.0, 255.0) as u8);
        rgb_data.push((y2 - 0.344136 * u - 0.714136 * v).clamp(0.0, 255.0) as u8);
        rgb_data.push((y2 + 1.772 * u).clamp(0.0, 255.0) as u8);
    }
    // ----------------------------

    let img = RgbImage::from_raw(fmt.width, fmt.height, rgb_data)
        .ok_or("Failed to create image buffer")?;

    img.save("snapshot.png")?;

    println!("Saved to 'snapshot.png'");
    Ok(())
}
