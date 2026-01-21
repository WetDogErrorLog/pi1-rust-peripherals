use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::prelude::*;
use v4l::video::Capture;

pub struct Camera {
    pub width: u32,
    pub height: u32,
}


impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Camera {
            width: width,
            height: height
        }
    }

    pub fn take_picture(&self, dev_path: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut dev = Device::with_path(dev_path)?;
        let mut fmt = dev.format()
            .expect("failed to read the format on the camera");
        fmt.width = self.width;
        fmt.height = self.height;
        // TODO: Convert to a MJPEG for decreased transmission size?
        fmt.fourcc = v4l::FourCC::new(b"YUYV");
        
        // Important: capture the actual format the camera accepted
        let fmt = dev.set_format(&fmt)?;

        println!("Camera set to {}x{} ({})", fmt.width, fmt.height, fmt.fourcc);

        let mut stream = MmapStream::with_buffers(&mut dev, Type::VideoCapture, 4)?;
        // Use the trait method to get the next frame
        let (data, _metadata) = CaptureStream::next(&mut stream)?;
        
        Ok(data.to_vec())
    }
}
