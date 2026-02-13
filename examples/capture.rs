use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use camera_stream::device::{CameraDevice, CameraManager};
use camera_stream::frame::{Frame, Timestamp};
use camera_stream::stream::CameraStream;

fn main() {
    #[cfg(target_os = "macos")]
    {
        use camera_stream::platform::macos::device::MacosCameraManager;

        let manager = MacosCameraManager::default();

        // Discover devices
        let devices: Vec<_> = manager
            .discover_devices()
            .expect("failed to discover devices")
            .collect();
        println!("Found {} camera(s):", devices.len());
        for (i, dev) in devices.iter().enumerate() {
            println!("  [{}] {} (id: {})", i, dev.name(), dev.id());
        }

        if devices.is_empty() {
            println!("No cameras found.");
            return;
        }

        // Use default device
        let device = manager
            .default_device()
            .expect("failed to get default device")
            .expect("no default camera");

        println!("\nUsing: {} ({})", device.name(), device.id());

        // Print supported formats
        let formats: Vec<_> = device
            .supported_formats()
            .expect("failed to get formats")
            .collect();
        println!("\nSupported formats ({} total):", formats.len());
        for (i, f) in formats.iter().take(10).enumerate() {
            println!(
                "  [{}] {:?} {}x{} ({} frame rate range(s))",
                i,
                f.pixel_format,
                f.size.width,
                f.size.height,
                f.frame_rate_ranges().len(),
            );
            for rr in f.frame_rate_ranges() {
                println!("       {:.1}-{:.1} fps", rr.min.as_f64(), rr.max.as_f64(),);
            }
        }
        if formats.len() > 10 {
            println!("  ... and {} more", formats.len() - 10);
        }

        // Pick first format or a reasonable default
        let config = if let Some(f) = formats.first() {
            let rate =
                f.frame_rate_ranges()
                    .first()
                    .map(|r| r.max)
                    .unwrap_or(camera_stream::Ratio {
                        numerator: 30000,
                        denominator: 1000,
                    });
            camera_stream::StreamConfig {
                pixel_format: f.pixel_format,
                size: f.size,
                frame_rate: rate,
            }
        } else {
            println!("No supported formats found.");
            return;
        };

        println!(
            "\nOpening with {:?} {}x{} @ {:.1} fps",
            config.pixel_format,
            config.size.width,
            config.size.height,
            config.frame_rate.as_f64(),
        );

        let mut stream = device.open(&config).expect("failed to open stream");

        let frame_count = Arc::new(AtomicU64::new(0));
        let count_clone = frame_count.clone();
        let target_frames: u64 = 60;

        stream
            .start(move |frame| {
                let n = count_clone.fetch_add(1, Ordering::Relaxed) + 1;
                let planes = frame.planes();
                let total_bytes: usize = planes.iter().map(|p| p.data.len()).sum();
                println!(
                    "Frame {}: {:?} {}x{} ts={:.3}s planes={} bytes={}",
                    n,
                    frame.pixel_format(),
                    frame.size().width,
                    frame.size().height,
                    frame.timestamp().as_secs_f64(),
                    planes.len(),
                    total_bytes,
                );
            })
            .expect("failed to start stream");

        // Wait until we've captured enough frames
        loop {
            std::thread::sleep(Duration::from_millis(100));
            if frame_count.load(Ordering::Relaxed) >= target_frames {
                break;
            }
        }

        stream.stop().expect("failed to stop stream");
        println!(
            "\nDone. Captured {} frames.",
            frame_count.load(Ordering::Relaxed)
        );
    }

    #[cfg(not(target_os = "macos"))]
    {
        println!("This example only works on macOS.");
    }
}
