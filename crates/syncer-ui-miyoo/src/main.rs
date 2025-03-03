use std::io;

use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    mono_font::MonoTextStyle,
    pixelcolor::Rgb888,
    prelude::{Point, RgbColor},
    text::Text,
};
use embedded_vintage_fonts::FONT_24X32;
use miyoo_io::{InputReader, MiyooFramebuffer};
use syncer_model::{commands::DaemonCommand, platforms::Platform};
use tokio::{io::AsyncWriteExt, net::UnixStream};

mod miyoo_io;

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async_main())
}

async fn async_main() {
    let mut fb = MiyooFramebuffer::find_any().unwrap();
    fb.clear(Rgb888::BLACK).unwrap();
    fb.flush().unwrap();
    let mut input = InputReader::new().unwrap();
    loop {
        let (btn, evt) = input.next_event().await.unwrap();
        let txt = format!("{btn:?} {evt:?}");
        let txt = Text::new(
            &txt,
            Point::new(0, 0),
            MonoTextStyle::new(&FONT_24X32, Rgb888::GREEN),
        );
        txt.draw(&mut fb).unwrap();
        fb.flush().unwrap();
    }
}

async fn install_daemon() -> Result<(), anyhow::Error> {
    Err(anyhow::anyhow!("TODO"))
}
async fn uninstall_daemon() -> Result<(), anyhow::Error> {
    Err(anyhow::anyhow!("TODO"))
}

pub struct DaemonSocket (UnixStream);

impl DaemonSocket {
    pub async fn new() -> Result<Self, io::Error> {
        let platform = Platform::get();
        let stream = UnixStream::connect(platform.socket_path()).await?;
        Ok(Self(stream))
    }
    pub async fn send(&mut self, command : DaemonCommand) -> Result<(), io::Error> {
        let command = command.serialize();
        self.0.write_all(command.as_bytes()).await
    }
}