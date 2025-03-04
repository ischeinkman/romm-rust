use std::io;

use buoyant::{
    environment::DefaultEnvironment,
    layout::Layout,
    primitives::{Point, Size},
    render::{EmbeddedGraphicsRender, EmbeddedGraphicsView, Renderable},
    view::{
        ForEach, HStack, LayoutExtensions, RenderExtensions, Spacer, Text, ZStack, padding::Edges,
        shape::Rectangle,
    },
};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Dimensions,
    pixelcolor::Rgb888,
    prelude::{RgbColor, WebColors},
};
use embedded_vintage_fonts::FONT_24X32;
use miyoo_io::{InputReader, MiyooButton, MiyooButtonEvent, MiyooFramebuffer};
use syncer_model::{commands::DaemonCommand, platforms::Platform};
use tokio::{io::AsyncWriteExt, net::UnixStream};

mod daemon;
mod miyoo_io;

fn main() {
    verify_platform();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async_main())
}

fn with_idx<T, const N: usize>(base: [T; N]) -> [(usize, T); N] {
    let mut idx = 0;
    base.map(|itm| {
        let nxt = idx;
        idx += 1;
        (nxt, itm)
    })
}

fn verify_platform() {
    let platform = Platform::get();
    if platform != Platform::MiyooMiniOnion {
        panic!("Can only run on Onion OS on the Miyoo Mini, but we're on {platform:?}");
    }
}

async fn async_main() {
    let mut fb = MiyooFramebuffer::find_any().unwrap();
    fb.clear(Rgb888::BLACK).unwrap();
    fb.flush().unwrap();
    let mut input = InputReader::new().unwrap();
    let buttons = with_idx(["B1", "B2"]);
    let mut selection = 0;
    let mut ons = buttons.map(|_| false);

    loop {
        let tree = ForEach::<2, _, _, _>::new(buttons, |&(idx, lbl)| {
            println!("BTN: {lbl} {idx} {:?} {:?}", idx == selection, ons[idx]);
            labeled_checkbox(lbl, idx == selection, ons[idx])
        })
        .flex_frame()
        .with_infinite_max_width();
        let display_rect = fb.bounding_box();
        let render_tree = tree.render_tree(
            &tree.layout(
                &Size::from(display_rect.size).into(),
                &DefaultEnvironment::default(),
            ),
            Point::zero(),
            &DefaultEnvironment::default(),
        );
        fb.clear(Rgb888::CSS_DARK_BLUE).unwrap();
        render_tree.render(&mut fb, &Rgb888::CSS_DARK_BLUE, Point::zero());
        fb.flush().unwrap();

        let (btn, evt) = input.next_event().await.unwrap();
        match (btn, evt) {
            (MiyooButton::Up, MiyooButtonEvent::Pressed) => {
                selection = selection.saturating_sub(1);
            }
            (MiyooButton::Down, MiyooButtonEvent::Pressed) => {
                selection = (selection + 1).min(buttons.len() - 1);
            }
            (MiyooButton::A, MiyooButtonEvent::Released) => {
                ons[selection] = !ons[selection];
            }
            (MiyooButton::Menu, MiyooButtonEvent::Pressed) => {
                break;
            }
            _ => {}
        }
        fb.flush().unwrap();
    }
}

fn labeled_checkbox(
    label: &str,
    is_selected: bool,
    is_on: bool,
) -> impl EmbeddedGraphicsView<Rgb888> + Layout + '_ {
    const HEIGHT: u16 = 32;
    const PADDING: u16 = 4;
    HStack::new((
        Text::new(label, &FONT_24X32).foreground_color(Rgb888::BLACK),
        Spacer::default(),
        checkbox(is_selected, is_on),
    ))
    .frame()
    .with_height(HEIGHT)
    .padding(Edges::All, PADDING)
}

fn checkbox(is_selected: bool, is_on: bool) -> impl EmbeddedGraphicsView<Rgb888> + Layout {
    const WIDTH: u16 = 20;
    const HEIGHT: u16 = 20;
    const SELECTION_PADDING: u16 = 5;

    const ON_COLOR: Rgb888 = Rgb888::GREEN;
    const OFF_COLOR: Rgb888 = Rgb888::CSS_DIM_GRAY;

    let lower_rect = Rectangle.foreground_color(Rgb888::BLACK);

    let padding = if is_selected { SELECTION_PADDING } else { 0 };
    let color = if is_on { ON_COLOR } else { OFF_COLOR };
    let upper_rect = Rectangle
        .foreground_color(color)
        .padding(Edges::All, padding);
    ZStack::new((lower_rect, upper_rect))
        .frame()
        .with_width(WIDTH)
        .with_height(HEIGHT)
        .geometry_group()
}

pub struct DaemonSocket(UnixStream);

impl DaemonSocket {
    pub async fn new() -> Result<Self, io::Error> {
        let platform = Platform::get();
        let stream = UnixStream::connect(platform.socket_path()).await?;
        Ok(Self(stream))
    }
    pub async fn send(&mut self, command: DaemonCommand) -> Result<(), io::Error> {
        let command = command.serialize();
        self.0.write_all(command.as_bytes()).await
    }
}
