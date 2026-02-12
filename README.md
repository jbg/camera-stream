# camera-stream

A cross-platform Rust library for streaming frames from cameras. Currently supports **macOS** via AVFoundation, with a trait-based architecture designed for future platform backends.

## Features

- **Device discovery** — enumerate cameras and query their supported formats (pixel format, size, frame rate ranges)
- **Zero-copy frame delivery** — frames are borrowed directly from the platform's pixel buffer within a callback scope
- **Configurable streams** — choose pixel format, size, and frame rate when opening a stream
- **Platform-specific extensions** — access advanced controls on macOS (focus, exposure, white balance, torch, zoom)
- **`no_std` support** — all core types and traits are available without `std` or `alloc`; only the platform backends require `std`

## Supported platforms

| Platform | Backend         | Status |
|----------|-----------------|--------|
| macOS    | AVFoundation    | ✅      |
| Linux    | —               | Planned |
| Windows  | —               | Planned |

## Quick start

Add to your `Cargo.toml`:

```toml
[dependencies]
camera-stream = "0.3"
```

### Example: capture frames

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use camera_stream::device::{CameraDevice, CameraManager};
use camera_stream::frame::Frame;
use camera_stream::stream::CameraStream;
use camera_stream::platform::macos::device::MacosCameraManager;

fn main() {
    let manager = MacosCameraManager::default();

    // Use the default camera
    let device = manager
        .default_device()
        .expect("failed to get default device")
        .expect("no default camera");

    println!("Using: {} ({})", device.name(), device.id());

    // Pick the first supported format
    let mut formats = device.supported_formats().expect("failed to get formats");
    let f = formats.next().expect("no supported formats");

    let config = camera_stream::StreamConfig {
        pixel_format: f.pixel_format,
        size: f.size,
        frame_rate: f.frame_rate_ranges().first().unwrap().max,
    };

    let mut stream = device.open(&config).expect("failed to open stream");

    let frame_count = Arc::new(AtomicU64::new(0));
    let counter = frame_count.clone();

    stream
        .start(move |frame| {
            let n = counter.fetch_add(1, Ordering::Relaxed) + 1;
            let planes = frame.planes();
            let total_bytes: usize = planes.iter().map(|p| p.data.len()).sum();
            println!(
                "Frame {}: {:?} {}x{} ts={:.3}s bytes={}",
                n,
                frame.pixel_format(),
                frame.size().width,
                frame.size().height,
                frame.timestamp().as_secs_f64(),
                total_bytes,
            );
        })
        .expect("failed to start stream");

    // Capture for 2 seconds
    std::thread::sleep(Duration::from_secs(2));

    stream.stop().expect("failed to stop stream");
    println!("Captured {} frames.", frame_count.load(Ordering::Relaxed));
}
```

Run the included example with:

```sh
cargo run --example capture
```

## Architecture

The library is built around three core traits:

| Trait | Purpose |
|-------|---------|
| `CameraManager` | Discover devices and get the default camera |
| `CameraDevice` | Inspect supported formats and open a stream |
| `CameraStream` | Start/stop streaming with a frame callback |

Frames are delivered through the `Frame` trait, which provides access to pixel format, size, timestamp, and per-plane image data.

All traits use `core::error::Error` bounds rather than `std::error::Error`, so they are usable in `no_std` environments. Methods that enumerate devices or formats return `impl Iterator` rather than `Vec`, avoiding heap allocation in the trait interface.

### Platform-specific extensions (macOS)

Import the `MacosCameraDeviceExt` trait from `camera_stream::platform::macos::ext` to access:

- **Focus** — query supported modes, set focus mode and point of interest
- **Exposure** — set mode, point of interest, and target bias
- **White balance** — set mode
- **Torch** — check availability and set mode
- **Zoom** — query max factor and set zoom level
- **Frame rate** — change active frame rate on a running device

All mutating operations acquire an `AVCaptureDevice` configuration lock automatically.

### Error handling

Platform errors preserve the native error objects (e.g. `NSError` on macOS) rather than eagerly converting to strings. Use `Display` (or `to_string()`) to get a human-readable description on demand.

## Pixel formats

| Variant | Description |
|---------|-------------|
| `Nv12` | YCbCr 4:2:0 biplanar (common macOS default) |
| `Yuyv` | YCbCr 4:2:2 packed |
| `Uyvy` | YCbCr 4:2:2 packed (alternate byte order) |
| `Bgra32` | 32-bit BGRA |
| `Jpeg` | JPEG compressed |

## Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | ✅ | Enables platform backends (macOS AVFoundation, etc.) |

Without `std`, all core types, traits (`CameraManager`, `CameraDevice`, `CameraStream`, `Frame`), and error types are still available — only the concrete platform implementations require `std`.

## Minimum Rust version

1.85 (edition 2024)

## License

Licensed under either of [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0) or [MIT License](http://opensource.org/licenses/MIT) at your option.
