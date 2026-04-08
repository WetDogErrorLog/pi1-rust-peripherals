use serde::{Deserialize, Serialize};
use v4l::buffer::Type;
use v4l::control::{Control, Flags, Value};
use v4l::io::traits::CaptureStream;
use v4l::prelude::*;
use v4l::video::Capture;

// V4L2 control IDs (0x00980900 base for user controls, 0x009A0900 for camera class)
const CID_BRIGHTNESS: u32 = 9_963_776;
const CID_CONTRAST: u32 = 9_963_777;
const CID_SATURATION: u32 = 9_963_778;
const CID_GAIN: u32 = 9_963_795;
const CID_AUTO_WHITE_BALANCE: u32 = 9_963_788;
const CID_WHITE_BALANCE_TEMP: u32 = 9_963_802;
const CID_GAMMA: u32 = 9_963_792;
const CID_SHARPNESS: u32 = 9_963_803;
const CID_BACKLIGHT_COMPENSATION: u32 = 9_963_804;
const CID_EXPOSURE_AUTO: u32 = 10_094_849;
const CID_EXPOSURE_ABSOLUTE: u32 = 10_094_850;

/// Optional V4L2 control overrides. Any field left as `None` is not touched
/// and the camera keeps its current (often auto-selected) value.
///
/// Order-sensitive fields:
///   - Set `exposure_auto = 1` (manual) before `exposure_absolute`
///   - Set `auto_white_balance = 0` (off) before `white_balance_temperature`
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CameraControls {
    pub brightness: Option<i64>,
    pub contrast: Option<i64>,
    pub saturation: Option<i64>,
    pub gain: Option<i64>,
    /// 0 = off (manual), 1 = on (auto)
    pub auto_white_balance: Option<i64>,
    /// Color temperature in Kelvin — only effective when auto_white_balance = 0
    pub white_balance_temperature: Option<i64>,
    pub gamma: Option<i64>,
    pub sharpness: Option<i64>,
    pub backlight_compensation: Option<i64>,
    /// 0 = auto, 1 = manual
    pub exposure_auto: Option<i64>,
    /// Exposure time in 100µs units — only effective when exposure_auto = 1
    pub exposure_absolute: Option<i64>,
}

/// A single control value as read back from the device after capture.
pub struct ControlSnapshot {
    pub name: String,
    pub id: u32,
    pub value: Value,
}

pub struct Camera {
    pub width: u32,
    pub height: u32,
    /// Frames to capture and discard before the real shot, giving auto-exposure
    /// and auto-white-balance time to converge. At ~5fps, 20 frames ≈ 4 seconds.
    pub warmup_frames: u32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Camera { width, height, warmup_frames: 20 }
    }

    /// Capture one frame from `dev_path`.
    ///
    /// If `controls` is provided, those values are applied after format negotiation
    /// and before streaming.
    ///
    /// Returns:
    ///   - `Vec<u8>` — raw YUYV bytes for the captured frame
    ///   - `Vec<ControlSnapshot>` — every readable control value as it stood at
    ///     capture time, including auto-selected values when no overrides were given.
    ///     Useful for logging what the camera actually used.
    pub fn take_picture(
        &self,
        dev_path: String,
        controls: Option<&CameraControls>,
    ) -> Result<(Vec<u8>, Vec<ControlSnapshot>), Box<dyn std::error::Error>> {
        let mut dev = Device::with_path(dev_path)?;

        let mut fmt = dev.format().expect("failed to read camera format");
        fmt.width = self.width;
        fmt.height = self.height;
        fmt.fourcc = v4l::FourCC::new(b"YUYV");
        let fmt = dev.set_format(&fmt)?;
        println!("Camera set to {}x{} ({})", fmt.width, fmt.height, fmt.fourcc);

        if let Some(ctrl) = controls {
            apply_controls(&dev, ctrl)?;
        }

        let mut stream = MmapStream::with_buffers(&mut dev, Type::VideoCapture, 4)?;

        // Discard warmup frames so auto-exposure and auto-white-balance have time
        // to converge before the real capture. At 5fps, 20 frames ≈ 4 seconds.
        for _ in 0..self.warmup_frames {
            CaptureStream::next(&mut stream)?;
        }

        let (data, _metadata) = CaptureStream::next(&mut stream)?;
        let image_bytes = data.to_vec();
        drop(stream);

        let snapshots = read_all_controls(&dev);

        Ok((image_bytes, snapshots))
    }
}

/// Print every control the device exposes — name, valid range, and current value.
/// Run this once to discover what your camera supports.
pub fn list_controls(dev_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dev = Device::with_path(dev_path)?;
    let descriptions = dev.query_controls()?;

    println!("{:<40} {:>12} {:>8} {:>8} {:>8} {:>8}  current",
        "name", "id", "min", "max", "step", "default");
    println!("{}", "-".repeat(100));

    for desc in descriptions {
        let rw = if desc.flags.contains(Flags::READ_ONLY) { "RO" } else { "RW" };
        let current = dev.control(desc.id)
            .ok()
            .map(|c| format_value(&c.value))
            .unwrap_or_else(|| "?".to_string());
        println!(
            "[{}] {:<38} {:>12} {:>8} {:>8} {:>8} {:>8}  {}",
            rw, desc.name, desc.id,
            desc.minimum, desc.maximum, desc.step, desc.default,
            current,
        );
    }

    Ok(())
}

fn apply_controls(dev: &Device, controls: &CameraControls) -> Result<(), Box<dyn std::error::Error>> {
    // Apply mode switches first so dependent controls take effect
    let ordered: &[(u32, Option<i64>)] = &[
        (CID_EXPOSURE_AUTO,        controls.exposure_auto),
        (CID_AUTO_WHITE_BALANCE,   controls.auto_white_balance),
        (CID_EXPOSURE_ABSOLUTE,    controls.exposure_absolute),
        (CID_WHITE_BALANCE_TEMP,   controls.white_balance_temperature),
        (CID_BRIGHTNESS,           controls.brightness),
        (CID_CONTRAST,             controls.contrast),
        (CID_SATURATION,           controls.saturation),
        (CID_GAIN,                 controls.gain),
        (CID_GAMMA,                controls.gamma),
        (CID_SHARPNESS,            controls.sharpness),
        (CID_BACKLIGHT_COMPENSATION, controls.backlight_compensation),
    ];
    for &(id, maybe_val) in ordered {
        if let Some(val) = maybe_val {
            dev.set_control(Control { id, value: Value::Integer(val) })?;
        }
    }
    Ok(())
}

fn read_all_controls(dev: &Device) -> Vec<ControlSnapshot> {
    dev.query_controls()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|desc| {
            dev.control(desc.id).ok().map(|c| ControlSnapshot {
                name: desc.name,
                id: c.id,
                value: c.value,
            })
        })
        .collect()
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Integer(n) => n.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::String(s)  => s.clone(),
        _                 => format!("{:?}", value),
    }
}
