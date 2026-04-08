# pi1-rust-peripherals
Rust packages for the pi1v2 arm32 to interface with peripherals and upload to shared storage

# To build for the ARM32 old pi1
cross build --target arm-unknown-linux-gnueabihf --release

# Monitoring TCP
sudo tcpdump -i any port 3000

# TODO:
- [x] Add timelapse support to leave camera running
- [x] Add control logging (V4L2 values written to `controls_log.csv` alongside images)
- [ ] Add jpeg support
- [ ] Add second camera support with async requests
- [ ] Convert project to a rust workspace instead of all separate projects.

# Config

Each loop entry in `sample_config.json` supports an optional `controls` block for overriding V4L2 camera settings. Omit it to leave the camera on full auto.

```json
{
  "service_addr": "192.168.4.22:6260",
  "loops": [
    {
      "device_path": "/dev/video0",
      "width": 1280,
      "height": 720,
      "project_folder": "my_project",
      "file_name_root": "angle_1",
      "image_format": "YUYV",
      "interval_minutes": 1,
      "controls": {
        "exposure_auto": 1,
        "exposure_absolute": 150,
        "auto_white_balance": 0,
        "white_balance_temperature": 4000
      }
    }
  ]
}
```

All `controls` fields are optional. Omit the entire `controls` block (or any individual field) to leave that setting on auto. Available fields:

| Field | Notes |
|---|---|
| `brightness` | |
| `contrast` | |
| `saturation` | |
| `gain` | |
| `auto_white_balance` | `0` = off, `1` = on |
| `white_balance_temperature` | Kelvin — set `auto_white_balance: 0` first |
| `exposure_auto` | `0` = auto, `1` = manual |
| `exposure_absolute` | 100µs units — set `exposure_auto: 1` first |

To discover what controls your specific camera supports and their valid ranges, run `pi_nexigo_cam_helloworld` — it prints a full control table with min/max/current values on startup.
