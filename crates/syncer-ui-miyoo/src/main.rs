use std::{
    env, io,
    ops::{ControlFlow, Deref},
    sync::Arc,
};

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
use futures::{FutureExt, TryFutureExt, future::Either, pin_mut};
use socketproto::DaemonSocket;
use tokio::sync::RwLock;
use tracing::{debug, error, level_filters::LevelFilter};

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
    view.render_view(&mut fb).unwrap();

    loop {
        let mapped_evt = {
            /*
            We need to separate out the code that polls for button events and view triggers from
            the code that handles them because right now the `.await` call on the combined future
            of these events won't actually drop the future until the end of the scope its in. To
            get around this we only actually generate the futures within a sub-scope and then
            translate their results into a single `Result<Option<Button, ButtonEvent>>` before the
            scope completes. This drops the future at the end of the sub-scope, allowing us to then
            use & update `view` as needed.
            */
            let input_fut = input.next_event().map_err(anyhow::Error::from).fuse();
            pin_mut!(input_fut);
            let trigger_fut = view.trigger_redraw().fuse();
            pin_mut!(trigger_fut);
            let res = futures::future::select(input_fut, trigger_fut).await;

            match res {
                Either::Left((Err(e), _)) | Either::Right((Err(e), _)) => Err(e),
                Either::Left((Ok((btn, evt)), _)) => Ok(Some((btn, evt))),
                Either::Right((Ok(()), _)) => Ok(None),
            }
        };
        match mapped_evt {
            Ok(Some((btn, evt))) => {
                view.handle_event(btn, evt).await.unwrap();
                view.render_view(&mut fb).unwrap();
            }
            Ok(None) => {
                view.render_view(&mut fb).unwrap();
            }
            Err(e) => {
                error!("Error waiting for redraw event: {e:?}");
            }
        }
    }
}

pub trait ViewState {
    /// Handle the up arrow.
    fn up(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        futures::future::ready(Ok(()))
    }
    /// Handle the down arrow.
    fn down(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        futures::future::ready(Ok(()))
    }
    /// Handle the left arrow.
    fn left(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        futures::future::ready(Ok(()))
    }
    /// Handle the right arrow.
    fn right(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        futures::future::ready(Ok(()))
    }
    /// Handle the `L` and `Lz` buttons.
    fn l(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        futures::future::ready(Ok(()))
    }
    /// Handle the `R` and `Rz` buttons.
    fn r(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        futures::future::ready(Ok(()))
    }
    /// Handle an `A` button press.
    ///
    /// Whether the current item triggers on press vs release depends on
    /// the screen and selection.
    fn press(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        futures::future::ready(Ok(()))
    }
    /// Handle an `A` button release.
    ///
    /// Whether the current item triggers on press vs release depends on
    /// the screen and selection.
    fn release(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        futures::future::ready(Ok(()))
    }
    /// Handle an `B` button press
    fn back(&mut self) -> impl Future<Output = Result<ControlFlow<(), ()>, anyhow::Error>> + '_ {
        futures::future::ready(Ok(ControlFlow::Continue(())))
    }
    /// Handles a single button event.
    ///
    /// The default implementation just forwards to the relevant trait methods.
    fn handle_event(
        &mut self,
        btn: MiyooButton,
        evt: MiyooButtonEvent,
    ) -> impl Future<Output = Result<ControlFlow<(), ()>, anyhow::Error>> + '_ {
        async move {
            let res = match (btn, evt) {
                (MiyooButton::Up, MiyooButtonEvent::Pressed) => {
                    self.up().await?;
                    ControlFlow::Continue(())
                }
                (MiyooButton::Down, MiyooButtonEvent::Pressed) => {
                    self.down().await?;
                    ControlFlow::Continue(())
                }
                (MiyooButton::Left, MiyooButtonEvent::Pressed) => {
                    self.left().await?;
                    ControlFlow::Continue(())
                }
                (MiyooButton::Right, MiyooButtonEvent::Pressed) => {
                    self.right().await?;
                    ControlFlow::Continue(())
                }
                (MiyooButton::A, MiyooButtonEvent::Pressed) => {
                    self.press().await?;
                    ControlFlow::Continue(())
                }
                (MiyooButton::A, MiyooButtonEvent::Released) => {
                    self.release().await?;
                    ControlFlow::Continue(())
                }
                (MiyooButton::R | MiyooButton::Rz, MiyooButtonEvent::Pressed) => {
                    self.r().await?;
                    ControlFlow::Continue(())
                }
                (MiyooButton::L | MiyooButton::Lz, MiyooButtonEvent::Pressed) => {
                    self.l().await?;
                    ControlFlow::Continue(())
                }
                (MiyooButton::Menu, MiyooButtonEvent::Pressed) => ControlFlow::Break(()),
                (MiyooButton::B, MiyooButtonEvent::Pressed) => self.back().await?,
                _ => ControlFlow::Continue(()),
            };
            Ok(res)
        }
    }

    /// Constructs the UI view for this [`ViewState`].
    fn build_view(&self) -> impl EmbeddedGraphicsView<Rgb888> + Layout + '_;

    /// Renders the output of [`ViewState::build_view`] to the given framebuffer.
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
    /// If this future completes then the output of [`ViewState::build_view`]
    /// has changed and we need a new call to [`ViewState::render_view`].
    fn trigger_redraw(&mut self) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        futures::future::pending()
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
    async fn l(&mut self) -> Result<(), anyhow::Error> {
        *self = match self {
            FullViewState::Homepage(state) => {
                FullViewState::SavesList(SavelistState::new(state.cfg.clone()).await)
            }
            FullViewState::SavesList(state) => {
                FullViewState::Homepage(HomepageState::new(state.cfg.clone()).await?)
            }
        };
        Ok(())
    }
    async fn r(&mut self) -> Result<(), anyhow::Error> {
        *self = match self {
            FullViewState::Homepage(state) => {
                FullViewState::SavesList(SavelistState::new(state.cfg.clone()).await)
            }
            FullViewState::SavesList(state) => {
                FullViewState::Homepage(HomepageState::new(state.cfg.clone()).await?)
            }
        };
        Ok(())
    }
    async fn back(&mut self) -> Result<ControlFlow<(), ()>, anyhow::Error> {
        match self {
            FullViewState::Homepage(_) => Ok(ControlFlow::Break(())),
            FullViewState::SavesList(state) => {
                *self = FullViewState::Homepage(HomepageState::new(state.cfg.clone()).await?);
                Ok(ControlFlow::Continue(()))
            }
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
    async fn trigger_redraw(&mut self) -> Result<(), anyhow::Error> {
        use FullViewState::*;
        match self {
            Homepage(view) => view.trigger_redraw().await,
            SavesList(view) => view.trigger_redraw().await,
        }
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
