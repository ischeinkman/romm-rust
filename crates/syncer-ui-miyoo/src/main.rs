use std::{env, io, ops::Deref, sync::Arc};

use anyhow::Context;
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
use socketproto::DaemonSocket;
use tokio::sync::RwLock;
use tracing::{debug, level_filters::LevelFilter};

use syncer_model::{
    commands::{DaemonCommand, DaemonCommandBody},
    config::Config,
    platforms::Platform,
};

mod components;
mod daemon;
mod homepage;
use homepage::HomepageState;
mod miyoo_io;
use miyoo_io::{InputReader, MiyooButton, MiyooButtonEvent, MiyooFramebuffer};
mod savelist;
mod socketproto;
use savelist::SavelistState;
use tracing_subscriber::{EnvFilter, FmtSubscriber, util::SubscriberInitExt as _};
mod utils;

fn main() {
    verify_platform();
    init_logger();
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

fn init_logger() {
    let trace_env = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .with_env_var("ROM_SYNC_LOG")
        .from_env()
        .unwrap();
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(trace_env)
        .with_file(true)
        .with_line_number(true);
    let no_color = env::var_os("NO_COLOR").is_some_and(|s| !s.eq_ignore_ascii_case("0"))
        || env::var_os("ROM_SYNC_UI_NO_COLOR").is_some_and(|s| !s.eq_ignore_ascii_case("0"));
    let json_log =
        env::var_os("ROM_SYNC_UI_LOG_JSON").is_some_and(|s| !s.eq_ignore_ascii_case("0"));
    let subscriber = subscriber.with_ansi(!no_color);
    if json_log {
        subscriber.json().finish().init();
    } else {
        subscriber.finish().init();
    }
}

async fn async_main() {
    let mut fb = MiyooFramebuffer::find_any().unwrap();
    fb.clear(Rgb888::BLACK).unwrap();
    fb.flush().unwrap();
    let mut input = InputReader::new().unwrap();
    let cfg = ApplicationState::new().await.unwrap();
    let mut view = FullViewState::new(cfg.clone()).await.unwrap();

    loop {
        view.render_view(&mut fb).unwrap();
        let (btn, evt) = input.next_event().await.unwrap();
        match (btn, evt) {
            (MiyooButton::Menu, MiyooButtonEvent::Pressed) => {
                break;
            }
            (MiyooButton::B, MiyooButtonEvent::Pressed) => {
                view = match view {
                    FullViewState::Homepage(_) => {
                        return;
                    }
                    _ => FullViewState::Homepage(HomepageState::new(cfg.clone()).await.unwrap()),
                };
            }
            (MiyooButton::R | MiyooButton::Rz, MiyooButtonEvent::Pressed) => {
                view = match view {
                    FullViewState::Homepage(_) => {
                        FullViewState::SavesList(SavelistState::new(cfg.clone()).await)
                    }
                    FullViewState::SavesList(_) => {
                        FullViewState::Homepage(HomepageState::new(cfg.clone()).await.unwrap())
                    }
                };
            }
            (MiyooButton::L | MiyooButton::Lz, MiyooButtonEvent::Pressed) => {
                view = match view {
                    FullViewState::Homepage(_) => {
                        FullViewState::SavesList(SavelistState::new(cfg.clone()).await)
                    }
                    FullViewState::SavesList(_) => {
                        FullViewState::Homepage(HomepageState::new(cfg.clone()).await.unwrap())
                    }
                };
            }
            (btn, evt) => {
                view.handle_event(btn, evt).await.unwrap();
            }
        }
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
    fn handle_event(
        &mut self,
        btn: MiyooButton,
        evt: MiyooButtonEvent,
    ) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        async move {
            match (btn, evt) {
                (MiyooButton::Up, MiyooButtonEvent::Pressed) => {
                    self.up().await?;
                }
                (MiyooButton::Down, MiyooButtonEvent::Pressed) => {
                    self.down().await?;
                }
                (MiyooButton::Left, MiyooButtonEvent::Pressed) => {
                    self.left().await?;
                }
                (MiyooButton::Right, MiyooButtonEvent::Pressed) => {
                    self.right().await?;
                }
                (MiyooButton::A, MiyooButtonEvent::Pressed) => {
                    self.press().await?;
                }
                (MiyooButton::A, MiyooButtonEvent::Released) => {
                    self.release().await?;
                }
                _ => {}
            }
            Ok(())
        }
    }
    fn build_view(&self) -> impl EmbeddedGraphicsView<Rgb888> + Layout + '_;
    fn render_view(&self, fb: &mut MiyooFramebuffer) -> Result<(), anyhow::Error> {
        let tree = self.build_view();
        let display_rect = fb.bounding_box();
        let render_tree = tree.render_tree(
            &tree.layout(
                &Size::from(display_rect.size).into(),
                &DefaultEnvironment::default(),
            ),
            Point::zero(),
            &DefaultEnvironment::default(),
        );
        fb.clear(Rgb888::CSS_DARK_BLUE)?;
        render_tree.render(fb, &Rgb888::CSS_DARK_BLUE, Point::zero());
        fb.flush()?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ApplicationState {
    cfg: Arc<RwLock<Config>>,
    socket: DaemonSocket,
}

impl ApplicationState {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let cfg = Config::load_current_platform()
            .await
            .context("Error loading config")?;
        let socket = DaemonSocket::new()
            .await
            .context("Error opening daemon socket")?;
        Ok(Self {
            cfg: Arc::new(RwLock::new(cfg)),
            socket,
        })
    }
    pub async fn config(&self) -> impl Deref<Target = Config> {
        self.cfg.read().await
    }
    pub async fn modify_and_save_cfg<F, R>(&self, cb: F) -> Result<R, anyhow::Error>
    where
        F: for<'a> AsyncFnOnce(&'a mut Config) -> R,
    {
        let mut lock = self.cfg.write().await;
        let res = cb(&mut lock).await;
        lock.save_current_platform().await?;
        let socket_res = self
            .socket
            .send(&DaemonCommand::new(DaemonCommandBody::DoSync))
            .await;
        match socket_res {
            Ok(()) => Ok(res),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                debug!("Attempted sync while daemon isn't running.");
                Ok(res)
            }
            Err(e) => Err(e.into()),
        }
    }
}

pub enum FullViewState {
    Homepage(HomepageState),
    SavesList(SavelistState),
}

impl FullViewState {
    pub async fn new(cfg: ApplicationState) -> Result<Self, anyhow::Error> {
        Ok(Self::Homepage(HomepageState::new(cfg).await?))
    }
}

impl ViewState for FullViewState {
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
