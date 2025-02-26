use std::{io, path::Path};

use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{Dimensions, DrawTarget, Point, RgbColor, Size},
    primitives::Rectangle,
    Pixel,
};
use evdev::{Device, EventStream, EventSummary, KeyCode};
use linuxfb::{Framebuffer, PixelLayout};
use tracing::warn;

/// Wrapper around the Miyoo Mini's framebuffer.
///
/// Also provides a second buffer for double-buffering, avoiding mid-frame
/// artifacts.
pub struct MiyooFramebuffer {
    buffer: Vec<u8>,
    fb: Framebuffer,
}

impl MiyooFramebuffer {
    /// Opens the any of the framebuffers we find on the system.
    ///
    /// The vast majority of cases only use a single framebuffer device and can
    /// therefore use this function.
    pub fn find_any() -> Result<Self, linuxfb::Error> {
        let path = Framebuffer::list()?
            .pop()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        Self::new(path)
    }
    pub fn new(path: impl AsRef<Path>) -> Result<Self, linuxfb::Error> {
        let fb = Framebuffer::new(path)?;
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&fb.map()?);
        Ok(Self { fb, buffer })
    }
    fn point_to_idx(&self, x: u32, y: u32) -> usize {
        let (w, h) = self.fb.get_size();
        let (w, h) = (w as usize, h as usize);
        let bpp = self.fb.get_bytes_per_pixel() as usize;
        let pixel_offset = (w - x as usize) + (h - y as usize) * w;
        pixel_offset * bpp
    }
    fn write_buffer(&mut self, x: u32, y: u32, buffer: [u8; 4]) {
        let bpp = self.fb.get_bytes_per_pixel();
        let start_idx = self.point_to_idx(x, y);
        let buffer = &buffer[buffer.len() - bpp as usize..];
        self.buffer[start_idx..start_idx + buffer.len()].copy_from_slice(buffer);
    }

    /// Flushes the current buffer to the screen.
    ///
    /// Must be called every frame to update the screen.
    pub fn flush(&mut self) -> Result<(), linuxfb::Error> {
        let mut fb = self.fb.map()?;
        fb.copy_from_slice(&self.buffer);
        Ok(())
    }
}

impl Dimensions for MiyooFramebuffer {
    fn bounding_box(&self) -> Rectangle {
        let (w, h) = self.fb.get_size();
        Rectangle::new(Point::zero(), Size::new(w, h))
    }
}

impl DrawTarget for MiyooFramebuffer {
    type Color = Rgb888;
    type Error = anyhow::Error;
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            let (w, h) = self.fb.get_size();
            if point.x < 0 || point.x > w as i32 || point.y < 0 || point.y > h as i32 {
                continue;
            }
            let x = point.x as u32;
            let y = point.y as u32;
            self.write_buffer(x, y, convert_color(color, self.fb.get_pixel_layout()));
        }
        Ok(())
    }
    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let converted = convert_color(color, self.fb.get_pixel_layout());
        let bpp = self.fb.get_bytes_per_pixel() as usize;
        let buffer = &converted[converted.len() - bpp..];
        let all_eq = buffer.iter().all(|bt| *bt == buffer[0]);
        if all_eq {
            self.buffer.fill(buffer[0]);
        } else {
            self.buffer = buffer.repeat(self.buffer.len() / buffer.len());
        }
        Ok(())
    }
}

fn convert_color(raw: Rgb888, target: PixelLayout) -> [u8; 4] {
    let pairs = [
        (raw.r(), target.red),
        (raw.g(), target.green),
        (raw.b(), target.blue),
        (0xFF, target.alpha),
    ];
    let mut retvl = 0u32;
    for (byte, layout) in pairs {
        if layout.length == 0 {
            continue;
        }
        let mut byte = byte >> (8 - layout.length);
        if layout.msb_right {
            byte = byte.reverse_bits();
        }
        retvl |= (byte as u32) << (24 - layout.offset);
    }
    retvl.to_be_bytes()
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum MiyooButton {
    A,
    B,
    X,
    Y,

    L,
    Lz,
    R,
    Rz,

    Up,
    Down,
    Left,
    Right,

    Start,
    Select,

    Menu,

    VolUp,
    VolDown,

    Power,
}
impl TryFrom<KeyCode> for MiyooButton {
    type Error = KeyCode;
    fn try_from(value: KeyCode) -> Result<Self, KeyCode> {
        use MiyooButton::*;

        match value {
            KeyCode::KEY_SPACE => Ok(A),
            KeyCode::KEY_LEFTCTRL => Ok(B),
            KeyCode::KEY_LEFTSHIFT => Ok(X),
            KeyCode::KEY_LEFTALT => Ok(Y),

            KeyCode::KEY_E => Ok(L),
            KeyCode::KEY_TAB => Ok(Lz),
            KeyCode::KEY_T => Ok(R),
            KeyCode::KEY_BACKSPACE => Ok(Rz),

            KeyCode::KEY_UP => Ok(Up),
            KeyCode::KEY_DOWN => Ok(Down),
            KeyCode::KEY_LEFT => Ok(Left),
            KeyCode::KEY_RIGHT => Ok(Right),

            KeyCode::KEY_ENTER => Ok(Start),
            KeyCode::KEY_RIGHTCTRL => Ok(Select),
            KeyCode::KEY_ESC => Ok(Menu),
            KeyCode::KEY_POWER => Ok(Power),

            KeyCode::KEY_VOLUMEUP => Ok(VolUp),
            KeyCode::KEY_VOLUMEDOWN => Ok(VolDown),
            other => Err(other),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MiyooButtonEvent {
    Pressed,
    Released,
}

pub struct InputReader {
    inner: EventStream,
}

impl InputReader {
    pub fn new() -> io::Result<Self> {
        let inner = Device::open("/dev/input/event0")?.into_event_stream()?;
        Ok(Self { inner })
    }
    pub async fn next_event(&mut self) -> io::Result<(MiyooButton, MiyooButtonEvent)> {
        loop {
            let raw = self.inner.next_event().await?;
            let EventSummary::Key(_, code, val) = raw.destructure() else {
                continue;
            };
            let button: MiyooButton = match code.try_into() {
                Ok(btn) => btn,
                Err(e) => {
                    warn!("Found unknown keycode: {e:?}");
                    continue;
                }
            };
            let event = match val {
                0 => MiyooButtonEvent::Released,
                1 => MiyooButtonEvent::Pressed,
                2 => {
                    // Key repeat; not sure what counts, but we'll treat it as a press for now.
                    MiyooButtonEvent::Pressed
                }
                other => {
                    warn!("Found unknown key event kind: {other}");
                    continue;
                }
            };
            return Ok((button, event));
        }
    }
}
