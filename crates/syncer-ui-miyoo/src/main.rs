use std::io;

use buoyant::{
    environment::DefaultEnvironment,
    layout::Layout,
    primitives::{Point, Size},
    render::{EmbeddedGraphicsRender, EmbeddedGraphicsView, Renderable},
    view::{
        match_view::{Branch2, MatchView}, padding::Edges, shape::Rectangle, HStack, LayoutExtensions, RenderExtensions, Text, ZStack
    },
};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Dimensions,
    pixelcolor::Rgb888,
    prelude::{RgbColor, WebColors},
};
use embedded_vintage_fonts::FONT_24X32;
use homepage::HomepageState;
use miyoo_io::{InputReader, MiyooButton, MiyooButtonEvent, MiyooFramebuffer};
use savelist::SavelistState;
use syncer_model::{commands::DaemonCommand, config::Config, platforms::Platform};
use tokio::{io::AsyncWriteExt, net::UnixStream};

mod daemon;
mod homepage;
mod miyoo_io;
mod savelist;
mod utils;

fn main() {
    verify_platform();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async_main())
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
    let mut cfg = Config::load_current_platform().await.unwrap();
    let mut view = FullViewState::new().await.unwrap();

    loop {
        let tree = view.build_view();
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
        drop(tree);
        drop(render_tree);

        let (btn, evt) = input.next_event().await.unwrap();
        match (btn, evt) {
            (MiyooButton::Up, MiyooButtonEvent::Pressed) => {
                view.up().await.unwrap();
            }
            (MiyooButton::Down, MiyooButtonEvent::Pressed) => {
                view.down().await.unwrap();
            }
            (MiyooButton::Left, MiyooButtonEvent::Pressed) => {
                view.left().await.unwrap();
            }
            (MiyooButton::Right, MiyooButtonEvent::Pressed) => {
                view.right().await.unwrap();
            }
            (MiyooButton::A, MiyooButtonEvent::Pressed) => {
                view.press().await.unwrap();
            }
            (MiyooButton::A, MiyooButtonEvent::Released) => {
                view.release().await.unwrap();
            }
            (MiyooButton::Menu, MiyooButtonEvent::Pressed) => {
                break;
            }
            (MiyooButton::R | MiyooButton::Rz, MiyooButtonEvent::Pressed) => {
                view = match view {
                    FullViewState::Homepage(_) => {
                        FullViewState::SavesList(SavelistState::new(&mut cfg).await)
                    }
                    FullViewState::SavesList(_) => {
                        FullViewState::Homepage(HomepageState::new().await.unwrap())
                    }
                };
            }
            (MiyooButton::L | MiyooButton::Lz, MiyooButtonEvent::Pressed) => {
                view = match view {
                    FullViewState::Homepage(_) => {
                        FullViewState::SavesList(SavelistState::new(&mut cfg).await)
                    }
                    FullViewState::SavesList(_) => {
                        FullViewState::Homepage(HomepageState::new().await.unwrap())
                    }
                };
            }
            _ => {}
        }
    }
}

fn labeled_checkbox<'a, S: AsRef<str> + Clone + 'a>(
    label: S,
    is_selected: bool,
    is_on: bool,
) -> impl EmbeddedGraphicsView<Rgb888> + Clone + 'a {
    const HEIGHT: u16 = 32;
    const PADDING: u16 = 4;
    HStack::new((
        Text::new(label, &FONT_24X32).foreground_color(Rgb888::BLACK),
        checkbox(is_selected, is_on),
    ))
    .flex_frame()
    .with_infinite_max_width()
    .with_min_height(HEIGHT)
    .with_max_height(HEIGHT)
    .padding(Edges::All, PADDING)
}

fn checkbox(is_selected: bool, is_on: bool) -> impl EmbeddedGraphicsView<Rgb888> + Layout + Clone {
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

fn button(
    text: &str,
    is_selected: bool,
    is_pressed: bool,
) -> impl EmbeddedGraphicsView<Rgb888> + Layout + Clone {
    const SELECTION_PADDING: u16 = 5;
    const HEIGHT: u16 = SELECTION_PADDING + FONT_24X32.character_size.height as u16;
    const PRESS_COLOR: Rgb888 = Rgb888::CSS_DARK_GREEN;
    const COLOR: Rgb888 = Rgb888::GREEN;
    const LABEL_COLOR: Rgb888 = Rgb888::BLACK;

    let lower_rect = Rectangle.foreground_color(Rgb888::BLACK);

    let padding = if is_selected { SELECTION_PADDING } else { 0 };
    let color = if is_pressed { PRESS_COLOR } else { COLOR };
    let upper_rect = Rectangle
        .foreground_color(color)
        .padding(Edges::All, padding);

    let label = Text::new(text, &FONT_24X32).foreground_color(LABEL_COLOR);

    ZStack::new((lower_rect, upper_rect, label))
        .flex_frame()
        .with_infinite_max_width()
        .with_max_height(HEIGHT)
        .with_min_height(HEIGHT)
        .with_ideal_height(HEIGHT)
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

pub trait ViewState {
    fn up(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        futures::future::ready(Ok(()))
    }
    fn down(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        futures::future::ready(Ok(()))
    }
    fn left(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        futures::future::ready(Ok(()))
    }
    fn right(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        futures::future::ready(Ok(()))
    }
    fn press(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        futures::future::ready(Ok(()))
    }
    fn release(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        futures::future::ready(Ok(()))
    }
    fn build_view(&self) -> impl EmbeddedGraphicsView<Rgb888> + Layout + Clone + '_;
}

pub enum FullViewState<'a> {
    Homepage(HomepageState),
    SavesList(SavelistState<'a>),
}

impl FullViewState<'_> {
    pub async fn new() -> Result<Self, anyhow::Error> {
        Ok(Self::Homepage(HomepageState::new().await?))
    }
}

impl ViewState for FullViewState<'_> {
    async fn up(&mut self) -> Result<(), anyhow::Error> {
        use FullViewState::*;
        match self {
            Homepage(view) => view.up().await,
            SavesList(view) => view.up().await,
        }
    }
    async fn down(&mut self) -> Result<(), anyhow::Error> {
        use FullViewState::*;
        match self {
            Homepage(view) => view.down().await,
            SavesList(view) => view.down().await,
        }
    }
    async fn left(&mut self) -> Result<(), anyhow::Error> {
        use FullViewState::*;
        match self {
            Homepage(view) => view.left().await,
            SavesList(view) => view.left().await,
        }
    }
    async fn right(&mut self) -> Result<(), anyhow::Error> {
        use FullViewState::*;
        match self {
            Homepage(view) => view.right().await,
            SavesList(view) => view.right().await,
        }
    }
    async fn press(&mut self) -> Result<(), anyhow::Error> {
        use FullViewState::*;
        match self {
            Homepage(view) => view.press().await,
            SavesList(view) => view.press().await,
        }
    }
    async fn release(&mut self) -> Result<(), anyhow::Error> {
        use FullViewState::*;
        match self {
            Homepage(view) => view.release().await,
            SavesList(view) => view.release().await,
        }
    }
    fn build_view(&self) -> impl EmbeddedGraphicsView<Rgb888> + Layout + Clone + '_ {
        match self {
            FullViewState::SavesList(view) => {
                let inner = view.build_view();
                MatchView::<Branch2<_, _>>::new(Branch2::Variant0(inner))
            }
            FullViewState::Homepage(view) => {
                let inner = view.build_view();
                MatchView::<Branch2<_, _>>::new(Branch2::Variant1(inner))
            }
        }
    }
}
