use buoyant::{
    layout::Layout,
    render::EmbeddedGraphicsView,
    view::{
        HStack, LayoutExtensions, RenderExtensions, Text, ZStack, padding::Edges, shape::Rectangle,
    },
};
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{RgbColor, WebColors},
};
use embedded_vintage_fonts::FONT_24X32;

/// A [`checkbox`] with a label to the left. 
pub fn labeled_checkbox<'a, S: AsRef<str> + Clone + 'a>(
    label: S,
    is_selected: bool,
    is_on: bool,
) -> impl EmbeddedGraphicsView<Rgb888> + Clone + 'a {
    const HEIGHT: u16 = FONT_24X32.character_size.height as _;
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

/// A checkbox that can either be on or off as well as selected as the current
/// element.
pub fn checkbox(
    is_selected: bool,
    is_on: bool,
) -> impl EmbeddedGraphicsView<Rgb888> + Layout + Clone {
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

/// A button with text that can be selected & pressed. 
pub fn button(
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
