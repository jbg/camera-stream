use std::panic::AssertUnwindSafe;
use std::sync::{Arc, Mutex};

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::runtime::ProtocolObject;
use objc2::{AllocAnyThread, DefinedClass, define_class, msg_send};
use objc2_av_foundation::{
    AVCaptureConnection, AVCaptureDevice, AVCaptureDeviceFormat, AVCaptureDeviceInput,
    AVCaptureOutput, AVCaptureSession, AVCaptureVideoDataOutput,
    AVCaptureVideoDataOutputSampleBufferDelegate,
};
use objc2_core_media::CMSampleBuffer;
use objc2_core_video::{
    CVPixelBufferLockBaseAddress, CVPixelBufferLockFlags, CVPixelBufferUnlockBaseAddress,
    kCVPixelBufferPixelFormatTypeKey,
};
use objc2_foundation::{NSDictionary, NSNumber, NSObjectProtocol, NSString};

use crate::error::{Error, PlatformError};
use crate::platform::macos::device::pixel_format_to_fourcc;
use crate::platform::macos::frame::{MacosFrame, MacosTimestamp};
use crate::stream::CameraStream;
use crate::types::StreamConfig;

/// Catch Objective-C exceptions and convert them to our Error type.
fn catch_objc<R>(f: impl FnOnce() -> R + std::panic::UnwindSafe) -> Result<R, Error> {
    objc2::exception::catch(f).map_err(|exception| {
        let msg = exception
            .map(|e| e.to_string())
            .unwrap_or_else(|| "unknown Objective-C exception".into());
        Error::Platform(PlatformError::Message(msg))
    })
}

type FrameCallback = Box<dyn FnMut(&MacosFrame<'_>) + Send + 'static>;

struct DelegateIvars {
    callback: Arc<Mutex<Option<FrameCallback>>>,
}

define_class!(
    #[unsafe(super(objc2_foundation::NSObject))]
    #[ivars = DelegateIvars]
    #[name = "CameraStreamSampleBufferDelegate"]
    struct SampleBufferDelegate;

    impl SampleBufferDelegate {
    }

    unsafe impl NSObjectProtocol for SampleBufferDelegate {}

    unsafe impl AVCaptureVideoDataOutputSampleBufferDelegate for SampleBufferDelegate {
        #[unsafe(method(captureOutput:didOutputSampleBuffer:fromConnection:))]
        #[allow(non_snake_case)]
        unsafe fn captureOutput_didOutputSampleBuffer_fromConnection(
            &self,
            _output: &AVCaptureOutput,
            sample_buffer: &CMSampleBuffer,
            _connection: &AVCaptureConnection,
        ) {
            // Get the pixel buffer from the sample buffer
            let pixel_buffer = match unsafe { sample_buffer.image_buffer() } {
                Some(pb) => pb,
                None => return,
            };

            // Get timestamp
            let cm_time = unsafe { sample_buffer.presentation_time_stamp() };
            let timestamp = MacosTimestamp {
                value: cm_time.value,
                timescale: cm_time.timescale,
                flags: cm_time.flags.0,
                epoch: cm_time.epoch,
            };

            // Lock, build frame, call callback, unlock
            let lock_flags = CVPixelBufferLockFlags::ReadOnly;
            unsafe {
                CVPixelBufferLockBaseAddress(&pixel_buffer, lock_flags);
            }

            let frame = unsafe { MacosFrame::from_locked_pixel_buffer(&pixel_buffer, timestamp) };

            if let Ok(mut guard) = self.ivars().callback.lock()
                && let Some(ref mut cb) = *guard {
                    cb(&frame);
                }

            unsafe {
                CVPixelBufferUnlockBaseAddress(&pixel_buffer, lock_flags);
            }
        }
    }
);

impl SampleBufferDelegate {
    fn new(callback: FrameCallback) -> Retained<Self> {
        let ivars = DelegateIvars {
            callback: Arc::new(Mutex::new(Some(callback))),
        };
        let obj = Self::alloc().set_ivars(ivars);
        unsafe { msg_send![super(obj), init] }
    }
}

/// macOS camera stream backed by `AVCaptureSession`.
pub struct MacosCameraStream {
    session: Retained<AVCaptureSession>,
    device: Retained<AVCaptureDevice>,
    output: Retained<AVCaptureVideoDataOutput>,
    delegate: Option<Retained<SampleBufferDelegate>>,
    /// True while the device config lock is held (between open and start).
    config_locked: bool,
    running: bool,
}

impl MacosCameraStream {
    pub(crate) fn new(
        device: Retained<AVCaptureDevice>,
        config: &StreamConfig,
    ) -> Result<Self, Error> {
        let session = unsafe { AVCaptureSession::new() };

        // Create device input
        let input = unsafe { AVCaptureDeviceInput::deviceInputWithDevice_error(&device) }
            .map_err(|e| Error::Platform(PlatformError::Message(e.to_string())))?;

        // Create video data output
        let output = unsafe { AVCaptureVideoDataOutput::new() };

        // Tell the output to deliver frames in the requested pixel format
        // rather than its own default (which is typically UYVY).
        let target_fourcc = pixel_format_to_fourcc(&config.pixel_format);
        unsafe {
            let key: &NSString = std::mem::transmute::<&objc2_core_foundation::CFString, &NSString>(
                kCVPixelBufferPixelFormatTypeKey,
            );
            let value = NSNumber::new_u32(target_fourcc);
            let settings: Retained<NSDictionary<NSString, AnyObject>> =
                NSDictionary::dictionaryWithObject_forKey(&value, ProtocolObject::from_ref(key));
            output.setVideoSettings(Some(&settings));
        }

        // Find matching format before configuring the session
        let formats = unsafe { device.formats() };
        let mut matched_format: Option<Retained<AVCaptureDeviceFormat>> = None;

        for format in formats.iter() {
            let desc = unsafe { format.formatDescription() };
            let sub_type = unsafe { desc.media_sub_type() };
            let dims = unsafe { objc2_core_media::CMVideoFormatDescriptionGetDimensions(&desc) };

            if sub_type == target_fourcc
                && dims.width as u32 == config.size.width
                && dims.height as u32 == config.size.height
            {
                matched_format = Some(format.clone());
                break;
            }
        }

        let matched = matched_format.ok_or(Error::UnsupportedFormat)?;

        let frame_duration = objc2_core_media::CMTime {
            value: config.frame_rate.denominator as i64,
            timescale: config.frame_rate.numerator as i32,
            flags: objc2_core_media::CMTimeFlags(1), // kCMTimeFlags_Valid
            epoch: 0,
        };

        catch_objc(AssertUnwindSafe(|| unsafe {
            session.beginConfiguration();

            // Add input
            if !session.canAddInput(&input) {
                session.commitConfiguration();
                return Err(Error::Platform(PlatformError::Message(
                    "cannot add input to session".into(),
                )));
            }
            session.addInput(&input);

            // Add output
            if !session.canAddOutput(&output) {
                session.commitConfiguration();
                return Err(Error::Platform(PlatformError::Message(
                    "cannot add output to session".into(),
                )));
            }
            session.addOutput(&output);

            session.commitConfiguration();
            Ok::<(), Error>(())
        }))??;

        // Lock the device for configuration and set the active format.
        // The lock is intentionally held across startRunning() — if we
        // unlock before startRunning the session's preset overrides
        // our format choice.
        unsafe { device.lockForConfiguration() }
            .map_err(|e| Error::Platform(PlatformError::Message(e.to_string())))?;

        catch_objc(AssertUnwindSafe(|| unsafe {
            device.setActiveFormat(&matched);
            device.setActiveVideoMinFrameDuration(frame_duration);
            device.setActiveVideoMaxFrameDuration(frame_duration);
        }))?;

        Ok(MacosCameraStream {
            session,
            device,
            output,
            delegate: None,
            config_locked: true,
            running: false,
        })
    }
}

impl CameraStream for MacosCameraStream {
    type Frame<'a> = MacosFrame<'a>;
    type Error = Error;

    fn start<F>(&mut self, callback: F) -> Result<(), Self::Error>
    where
        F: FnMut(&Self::Frame<'_>) + Send + 'static,
    {
        if self.running {
            return Err(Error::AlreadyStarted);
        }

        let delegate = SampleBufferDelegate::new(Box::new(callback));

        let queue = dispatch2::DispatchQueue::new(
            "camera-stream.callback",
            dispatch2::DispatchQueueAttr::SERIAL,
        );

        unsafe {
            self.output.setSampleBufferDelegate_queue(
                Some(ProtocolObject::from_ref(&*delegate)),
                Some(&queue),
            );
        }

        self.delegate = Some(delegate);

        catch_objc(AssertUnwindSafe(|| unsafe { self.session.startRunning() }))?;
        self.running = true;

        // Now that the session is running with our format, release the
        // device config lock.
        if self.config_locked {
            unsafe { self.device.unlockForConfiguration() };
            self.config_locked = false;
        }

        Ok(())
    }

    fn stop(&mut self) -> Result<(), Self::Error> {
        if !self.running {
            return Err(Error::NotStarted);
        }

        unsafe { self.session.stopRunning() };

        unsafe {
            self.output.setSampleBufferDelegate_queue(None, None);
        }

        // Clear the callback
        if let Some(ref delegate) = self.delegate
            && let Ok(mut guard) = delegate.ivars().callback.lock()
        {
            *guard = None;
        }
        self.delegate = None;
        self.running = false;

        Ok(())
    }
}

impl Drop for MacosCameraStream {
    fn drop(&mut self) {
        if self.running {
            let _ = self.stop();
        }
        if self.config_locked {
            unsafe { self.device.unlockForConfiguration() };
            self.config_locked = false;
        }
    }
}
