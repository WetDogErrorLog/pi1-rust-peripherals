# pi1-rust-peripherals
Rust packages for the pi1v2 arm32 to interface with peripherals and upload to shared storage

# To build for the ARM32 old pi1
cross build --target arm-unknown-linux-gnueabihf --release

# Monitoring TCP
sudo tcpdump -i any port 3000

# TODO:
- [ ] Add timelapse support to leave camera running
- [ ] Add jpeg support
- [ ] Add second camera support with async requests
- [ ] Add logging and monitoring
- [ ] Add a config template to fill out for quick project setups
- [ ] Convert project to a rust workspace instead of all separate projects.

An ideal config will list -
- One destination server with folder name
- One or more (camera_path, file_prefix, capture_rate) tuples.
