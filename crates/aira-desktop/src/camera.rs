//! Webcam frame capture for invite QR scan (QUEUE #133).

use anyhow::{Context, Result};
use image::{GrayImage, Luma};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;

/// Open the default system camera for QR scanning.
pub struct InviteQrCamera {
    camera: Camera,
}

impl InviteQrCamera {
    pub fn open_default() -> Result<Self> {
        let index = CameraIndex::Index(0);
        let requested =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
        let mut camera = Camera::new(index, requested).context("open default camera (index 0)")?;
        camera.open_stream().context("start camera stream")?;
        Ok(Self { camera })
    }

    /// Grab one frame as grayscale for `rqrr` decode.
    pub fn grab_luma_frame(&mut self) -> Result<GrayImage> {
        let frame = self.camera.frame().context("read camera frame")?;
        let decoded = frame
            .decode_image::<RgbFormat>()
            .context("decode RGB frame")?;
        let (w, h) = (decoded.width(), decoded.height());
        let raw = decoded.as_raw();
        let mut luma = GrayImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize * 3;
                let v = ((raw[i] as u16 + raw[i + 1] as u16 + raw[i + 2] as u16) / 3) as u8;
                luma.put_pixel(x, y, Luma([v]));
            }
        }
        Ok(luma)
    }

    pub fn stop(&mut self) {
        let _ = self.camera.stop_stream();
    }
}

impl Drop for InviteQrCamera {
    fn drop(&mut self) {
        self.stop();
    }
}
