use std::io;

use buoyant::{
    environment::DefaultEnvironment,
    layout::Layout,
    primitives::{Point, Size},
    render::{EmbeddedGraphicsRender, EmbeddedGraphicsView, Renderable},
    view::{
        HStack, LayoutExtensions, RenderExtensions, Text, VStack, ZStack,
        match_view::{Branch2, MatchView},
        padding::Edges,
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
use homepage::HomepageState;
use miyoo_io::{InputReader, MiyooButton, MiyooButtonEvent, MiyooFramebuffer};
use savelist::SavelistState;
use syncer_model::{commands::DaemonCommand, config::Config, platforms::Platform};
use tokio::{io::AsyncWriteExt, net::UnixStream};

mod components;
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
    let mut view = FullViewState::new(&mut cfg).await.unwrap();

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
                        FullViewState::Homepage(HomepageState::new(&mut cfg).await.unwrap())
                    }
                };
            }
            (MiyooButton::L | MiyooButton::Lz, MiyooButtonEvent::Pressed) => {
                view = match view {
                    FullViewState::Homepage(_) => {
                        FullViewState::SavesList(SavelistState::new(&mut cfg).await)
                    }
                    FullViewState::SavesList(_) => {
                        FullViewState::Homepage(HomepageState::new(&mut cfg).await.unwrap())
                    }
                };
            }
            _ => {}
        }
    }
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
    fn build_view(&self) -> impl EmbeddedGraphicsView<Rgb888> + Layout + '_;
}

pub enum FullViewState<'a> {
    Homepage(HomepageState<'a>),
    SavesList(SavelistState<'a>),
}

impl <'a> FullViewState<'a> {
    pub async fn new(cfg : &'a mut Config) -> Result<Self, anyhow::Error> {
        Ok(Self::Homepage(HomepageState::new(cfg).await?))
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
    fn build_view(&self) -> impl EmbeddedGraphicsView<Rgb888> + Layout + '_ {
        let (inner, tab_selection) = match self {
            FullViewState::SavesList(view) => {
                let inner = view.build_view();
                (MatchView::<Branch2<_, _>>::new(Branch2::Variant0(inner)), 0)
            }
            FullViewState::Homepage(view) => {
                let inner = view.build_view();
                (MatchView::<Branch2<_, _>>::new(Branch2::Variant1(inner)), 1)
            }
        };
        let tabs = HStack::new((
            header_tab("Home", tab_selection == 0),
            header_tab("Saves", tab_selection == 1),
        ))
        .flex_frame()
        .with_infinite_max_width();
        VStack::new((tabs, inner)).flex_frame()
    }
}

fn header_tab(label: &str, selected: bool) -> impl EmbeddedGraphicsView<Rgb888> + Clone {
    const UNSELECTED_TEXT_COLOR: Rgb888 = Rgb888::BLACK;
    const UNSELECTED_BACKGROUND_COLOR: Rgb888 = Rgb888::CSS_DARK_BLUE;
    const RECT_H_BORDER: u16 = 5;
    const TAB_LABEL_PADDING: u16 = 2;

    let txt_color = if selected {
        UNSELECTED_BACKGROUND_COLOR
    } else {
        UNSELECTED_TEXT_COLOR
    };
    let background_color = if selected {
        UNSELECTED_TEXT_COLOR
    } else {
        UNSELECTED_BACKGROUND_COLOR
    };

    let txt = Text::new(label, &FONT_24X32).foreground_color(txt_color);
    let background_rect = Rectangle.foreground_color(background_color);
    let buffer_rect = Rectangle
        .foreground_color(Rgb888::BLACK)
        .padding(Edges::Horizontal, RECT_H_BORDER);
    ZStack::new((buffer_rect, background_rect, txt))
        .flex_frame()
        .with_infinite_max_width()
        .with_min_height(FONT_24X32.character_size.height as u16 + 2 * TAB_LABEL_PADDING)
        .with_max_height(FONT_24X32.character_size.height as u16 + 2 * TAB_LABEL_PADDING)
}
