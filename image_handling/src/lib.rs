use image::{RgbImage};
use std::fmt;
use serde::{Serialize, Deserialize};
use reqwest::blocking::multipart;
use ax_multipart::Multipart;

// Put the data into a struct so it can be serialized and send.
// Less byte usage than sending the multipart with various headers.
#[derive(Serialize, Deserialize, Debug)]
pub struct CameraPacket {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub data: Vec<u8>,
}

// Enum to represent the supported image types.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    YUYV,
    MJPEG,
}
// Error enum for image conversions
#[derive(Debug)]
pub enum ImageUnpackError {
    BufferLengthMismatch { expected: usize, actual: usize },
    InvalidData,
}

impl fmt::Display for ImageUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BufferLengthMismatch { expected, actual } =>
                write!(f, "Expected {} bytes, but got {}", expected, actual),
            Self::InvalidData => write!(f, "The image data was malformed"),
        }
    }
}

impl std::error::Error for ImageUnpackError {}

// Convert YUVU to RGB. 
pub fn yuyv_to_rgb(height: u32, width: u32, yuyv_data: &[u8]) -> Result<RgbImage, ImageUnpackError> {
    let expected_size = (height * width * 2) as usize;
    if yuyv_data.len() != expected_size {
        return Err(ImageUnpackError::BufferLengthMismatch {
            expected: expected_size,
            actual: yuyv_data.len()
        });
    }
    let mut rgb_data = Vec::with_capacity(((width*height*3)) as usize);
    
    for chunk in yuyv_data.chunks_exact(4) {
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
    RgbImage::from_raw(width, height, rgb_data)
        .ok_or(ImageUnpackError::InvalidData)
}

// send the image to the destination service.
pub fn send_image(
    width: u32,
    height: u32,
    format: ImageFormat,
    raw_bytes: Vec<u8>,
    file_name: String,
    // 'http://<server>:port'
    service_addr: String,
) {
    client = reqwest::blocking::Client::new();

    let packet = CameraPacket {
        width=width,
        height=height,
        format=format,
        data=raw_bytes,
    }
    let url = service_addr.push("/upload_image")

    client.post("{service_addr}/upload_image")
        .body(form)
        .send()
        .unwrap();
}

// Convert the bytes into rgb data and saves.
// dest_dir should not include a trailing slash.
pub fn handle_image_post(body: Bytes, dest_dir: &str) {
    let packet: CameraPacket = bincode::deserialize(&body).unwrap();
    match packet.format {
        ImageFormat::YUYV => rgb_data = yuyv_to_rgb(
            height: packet.height,
            width: packet.width,
            yuyv_data: &packet.data,
        );
        ImageFormat::MJPEG => rgb_data = mjpeg_to_rgb(
            height: packet.height,
            width: packet.width,
            yuyv_data: &packet.data,
        );
    }
    let path = format!("{dest_dir}/{name}_{timestamp}.png");
    rgb_data.save(path);
}
