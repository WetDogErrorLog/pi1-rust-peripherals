use image::{RgbImage};
use std::{fmt, fs};
use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;
use chrono::Local;

// Put the data into a struct so it can be serialized and send.
// Less byte usage than sending the multipart with various headers.
#[derive(Serialize, Deserialize, Debug)]
pub struct CameraPacket {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub data: Vec<u8>,
    pub file_name_root: String,
    pub project_folder: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimelapseSessionConfig {
    pub service_addr: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimelapseLoopConfig {
    pub device_path: String,
    pub width: u32,
    pub height: u32,
    pub file_name_root: String,
    pub project_folder: String,
    pub image_format: ImageFormat,
    pub interval_minutes: u32,
    // pub camera_path
    // pub camera_method???
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

// send the image to the destination service.
pub fn send_image(
    service_addr: String,
    raw_bytes: Vec<u8>,
    loop_config: TimelapseLoopConfig,
) {
    println!("running send_image");
    let client = Client::new();

    let packet = CameraPacket {
        width: loop_config.width,
        height: loop_config.height,
        format: loop_config.image_format,
        data: raw_bytes,
        file_name_root: String::from(loop_config.file_name_root),
        project_folder: String::from(loop_config.project_folder),
    };
    let target_url = format!("http://{}/upload_image", service_addr);
    client.post(target_url)
        .json(&packet)
        .send()
        .expect("Failed to upload image to server");
}

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

pub fn mjpeg_to_rgb(height: u32, width: u32, mjpeg_data: &[u8]) -> Result<RgbImage, ImageUnpackError> {
    todo!("add support for jpeg conversion");
}

// Convert the bytes into rgb data and saves.
// dest_dir should not include a trailing slash.
pub fn handle_image_post(packet: CameraPacket, dest_dir: &str) -> std::io::Result<()> {
    let rgb_data = match packet.format {
        ImageFormat::YUYV => yuyv_to_rgb(
            packet.height,
            packet.width,
            &packet.data,
        ),
        ImageFormat::MJPEG => mjpeg_to_rgb(
            packet.height,
            packet.width,
            &packet.data,
        ),
    }
    .expect("failed to convert image for {file_root} of type {packet.format}");
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M").to_string();
    let file_root: &str = &packet.file_name_root;
    let project_folder: &str = &packet.project_folder;
    let path_to_project: &str = &format!("{dest_dir}/{project_folder}");
    println!("creating path_to_project: {path_to_project}");
    fs::create_dir_all(path_to_project);
    let path = format!("{path_to_project}/{file_root}_{timestamp}.png");
    let result = rgb_data.save(path);
    result.expect("failed to save the rgb image for {file_root}");
    Ok(())
}

