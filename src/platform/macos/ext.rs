use std::ffi::c_void;

use objc2_av_foundation::{AVCaptureDevice, AVCaptureExposureMode, AVCaptureFocusMode};
use objc2_core_foundation::CGPoint;

use crate::error::{Error, PlatformError};
use crate::platform::macos::device::MacosCameraDevice;
use crate::platform::macos::frame::MacosFrame;
use crate::types::FrameRate;

// Re-export platform-specific enums for convenience
pub use objc2_av_foundation::{
    AVCaptureExposureMode as MacosExposureMode, AVCaptureFocusMode as MacosFocusMode,
    AVCaptureTorchMode as MacosTorchMode, AVCaptureWhiteBalanceMode as MacosWhiteBalanceMode,
};

/// RAII guard for `AVCaptureDevice` configuration lock.
pub struct ConfigLockGuard<'a> {
    device: &'a AVCaptureDevice,
}

impl<'a> ConfigLockGuard<'a> {
    pub fn device(&self) -> &AVCaptureDevice {
        self.device
    }
}

impl<'a> Drop for ConfigLockGuard<'a> {
    fn drop(&mut self) {
        unsafe { self.device.unlockForConfiguration() };
    }
}

/// macOS-specific camera device controls.
pub trait MacosCameraDeviceExt {
    fn lock_for_configuration(&self) -> Result<ConfigLockGuard<'_>, Error>;

    // Focus
    fn focus_modes(&self) -> Vec<MacosFocusMode>;
    fn set_focus_mode(&self, mode: MacosFocusMode) -> Result<(), Error>;
    fn set_focus_point(&self, x: f64, y: f64) -> Result<(), Error>;

    // Exposure
    fn exposure_modes(&self) -> Vec<MacosExposureMode>;
    fn set_exposure_mode(&self, mode: MacosExposureMode) -> Result<(), Error>;
    fn set_exposure_point(&self, x: f64, y: f64) -> Result<(), Error>;
    fn set_exposure_target_bias(&self, bias: f32) -> Result<(), Error>;

    // White balance
    fn set_white_balance_mode(&self, mode: MacosWhiteBalanceMode) -> Result<(), Error>;

    // Torch
    fn has_torch(&self) -> bool;
    fn set_torch_mode(&self, mode: MacosTorchMode) -> Result<(), Error>;

    // Zoom
    fn max_zoom_factor(&self) -> f64;
    fn set_zoom_factor(&self, factor: f64) -> Result<(), Error>;

    // Active format / frame rate
    fn set_active_frame_rate(&self, rate: FrameRate) -> Result<(), Error>;
}

impl MacosCameraDeviceExt for MacosCameraDevice {
    fn lock_for_configuration(&self) -> Result<ConfigLockGuard<'_>, Error> {
        unsafe { self.device.lockForConfiguration() }
            .map_err(|e| Error::Platform(PlatformError::Message(e.to_string())))?;
        Ok(ConfigLockGuard {
            device: &self.device,
        })
    }

    fn focus_modes(&self) -> Vec<MacosFocusMode> {
        let mut modes = Vec::new();
        let candidates = [
            AVCaptureFocusMode(0), // Locked
            AVCaptureFocusMode(1), // AutoFocus
            AVCaptureFocusMode(2), // ContinuousAutoFocus
        ];
        for mode in &candidates {
            if unsafe { self.device.isFocusModeSupported(*mode) } {
                modes.push(*mode);
            }
        }
        modes
    }

    fn set_focus_mode(&self, mode: MacosFocusMode) -> Result<(), Error> {
        let _guard = self.lock_for_configuration()?;
        unsafe { self.device.setFocusMode(mode) };
        Ok(())
    }

    fn set_focus_point(&self, x: f64, y: f64) -> Result<(), Error> {
        if !unsafe { self.device.isFocusPointOfInterestSupported() } {
            return Err(Error::Platform(PlatformError::Message(
                "focus point of interest not supported".into(),
            )));
        }
        let _guard = self.lock_for_configuration()?;
        unsafe {
            self.device.setFocusPointOfInterest(CGPoint { x, y });
        }
        Ok(())
    }

    fn exposure_modes(&self) -> Vec<MacosExposureMode> {
        let mut modes = Vec::new();
        let candidates = [
            AVCaptureExposureMode(0), // Locked
            AVCaptureExposureMode(1), // AutoExpose
            AVCaptureExposureMode(2), // ContinuousAutoExposure
            AVCaptureExposureMode(3), // Custom
        ];
        for mode in &candidates {
            if unsafe { self.device.isExposureModeSupported(*mode) } {
                modes.push(*mode);
            }
        }
        modes
    }

    fn set_exposure_mode(&self, mode: MacosExposureMode) -> Result<(), Error> {
        let _guard = self.lock_for_configuration()?;
        unsafe { self.device.setExposureMode(mode) };
        Ok(())
    }

    fn set_exposure_point(&self, x: f64, y: f64) -> Result<(), Error> {
        if !unsafe { self.device.isExposurePointOfInterestSupported() } {
            return Err(Error::Platform(PlatformError::Message(
                "exposure point of interest not supported".into(),
            )));
        }
        let _guard = self.lock_for_configuration()?;
        unsafe {
            self.device.setExposurePointOfInterest(CGPoint { x, y });
        }
        Ok(())
    }

    fn set_exposure_target_bias(&self, bias: f32) -> Result<(), Error> {
        let _guard = self.lock_for_configuration()?;
        unsafe {
            self.device
                .setExposureTargetBias_completionHandler(bias, None);
        }
        Ok(())
    }

    fn set_white_balance_mode(&self, mode: MacosWhiteBalanceMode) -> Result<(), Error> {
        if !unsafe { self.device.isWhiteBalanceModeSupported(mode) } {
            return Err(Error::Platform(PlatformError::Message(
                "white balance mode not supported".into(),
            )));
        }
        let _guard = self.lock_for_configuration()?;
        unsafe { self.device.setWhiteBalanceMode(mode) };
        Ok(())
    }

    fn has_torch(&self) -> bool {
        unsafe { self.device.hasTorch() }
    }

    fn set_torch_mode(&self, mode: MacosTorchMode) -> Result<(), Error> {
        if !unsafe { self.device.isTorchModeSupported(mode) } {
            return Err(Error::Platform(PlatformError::Message(
                "torch mode not supported".into(),
            )));
        }
        let _guard = self.lock_for_configuration()?;
        unsafe { self.device.setTorchMode(mode) };
        Ok(())
    }

    fn max_zoom_factor(&self) -> f64 {
        unsafe { self.device.maxAvailableVideoZoomFactor() }
    }

    fn set_zoom_factor(&self, factor: f64) -> Result<(), Error> {
        let _guard = self.lock_for_configuration()?;
        unsafe { self.device.setVideoZoomFactor(factor) };
        Ok(())
    }

    fn set_active_frame_rate(&self, rate: FrameRate) -> Result<(), Error> {
        let _guard = self.lock_for_configuration()?;
        let duration = objc2_core_media::CMTime {
            value: rate.denominator as i64,
            timescale: rate.numerator as i32,
            flags: objc2_core_media::CMTimeFlags(1),
            epoch: 0,
        };
        unsafe { self.device.setActiveVideoMinFrameDuration(duration) };
        unsafe { self.device.setActiveVideoMaxFrameDuration(duration) };
        Ok(())
    }
}

/// macOS-specific frame data.
pub trait MacosFrameExt {
    fn sample_buffer_ptr(&self) -> *const c_void;
}

impl MacosFrameExt for MacosFrame<'_> {
    fn sample_buffer_ptr(&self) -> *const c_void {
        self.pixel_buffer_ptr()
    }
}
