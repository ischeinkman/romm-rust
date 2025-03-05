use core::cmp::max;
use std::marker::PhantomData;

use buoyant::{
    environment::LayoutEnvironment,
    layout::{HorizontalAlignment, Layout, LayoutDirection, ResolvedLayout},
    primitives::{Dimension, Dimensions, Point, ProposedDimension, ProposedDimensions},
    render::{AnimatedJoin, AnimationDomain, EmbeddedGraphicsRender, Renderable},
};
use embedded_graphics::prelude::{DrawTarget, PixelColor};

#[derive(Debug, Clone)]
struct ForEachDynEnvironment<'a, T> {
    inner_environment: &'a T,
}

impl<T: LayoutEnvironment> LayoutEnvironment for ForEachDynEnvironment<'_, T> {
    fn alignment(&self) -> buoyant::layout::Alignment {
        self.inner_environment.alignment()
    }

    fn layout_direction(&self) -> LayoutDirection {
        LayoutDirection::Vertical
    }

    fn app_time(&self) -> core::time::Duration {
        self.inner_environment.app_time()
    }
}

impl<'a, T: LayoutEnvironment> From<&'a T> for ForEachDynEnvironment<'a, T> {
    fn from(environment: &'a T) -> Self {
        Self {
            inner_environment: environment,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ForEachDyn<S, V>
where
    S: AsRef<[V]>,
{
    iter: S,
    alignment: HorizontalAlignment,
    spacing: u16,
    _phanton: PhantomData<V>,
}

impl<S, V> PartialEq for ForEachDyn<S, V>
where
    S: PartialEq + AsRef<[V]>,
{
    fn eq(&self, other: &Self) -> bool {
        match (self.alignment, other.alignment) {
            (HorizontalAlignment::Center, HorizontalAlignment::Center)
            | (HorizontalAlignment::Leading, HorizontalAlignment::Leading)
            | (HorizontalAlignment::Trailing, HorizontalAlignment::Trailing) => {
                self.iter == other.iter && self.spacing == other.spacing
            }
            _ => false,
        }
    }
}

impl <S, V> Eq for ForEachDyn<S, V> where S : Eq + AsRef<[V]> {}

impl<I: AsRef<[V]>, V: Layout> ForEachDyn<I, V> {
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            alignment: HorizontalAlignment::default(),
            spacing: 0,
            _phanton: PhantomData,
        }
    }

    #[must_use]
    pub const fn with_alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    #[must_use]
    pub const fn with_spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<I: AsRef<[V]>, V: Layout> Layout for ForEachDyn<I, V> {
    type Sublayout = Vec<ResolvedLayout<V::Sublayout>>;

    fn layout(
        &self,
        offer: &ProposedDimensions,
        env: &impl LayoutEnvironment,
    ) -> ResolvedLayout<Self::Sublayout> {
        let env = &ForEachDynEnvironment::from(env);
        let mut sublayouts: Vec<ResolvedLayout<V::Sublayout>> = Vec::new();

        let items = self.iter.as_ref();

        let mut subview_stages: Vec<(i8, bool)> = Vec::new();
        // fill sublayouts with an initial garbage layout
        for item in items.iter() {
            let view = item;
            sublayouts.push(view.layout(offer, env));
            subview_stages.push((view.priority(), view.is_empty()));
        }

        let layout_fn = |index: usize, offer: ProposedDimensions| {
            let layout = items[index].layout(&offer, env);
            let size = layout.resolved_size;
            sublayouts[index] = layout;
            size
        };

        let size = layout_dynamic(&subview_stages, *offer, self.spacing, layout_fn);
        ResolvedLayout {
            sublayouts,
            resolved_size: size,
        }
    }
}

impl<S: AsRef<[V]>, V: Renderable<C> + Layout, C> Renderable<C> for ForEachDyn<S, V> {
    type Renderables = VecWrapper<V::Renderables>;

    fn render_tree(
        &self,
        layout: &ResolvedLayout<Self::Sublayout>,
        origin: Point,
        env: &impl LayoutEnvironment,
    ) -> Self::Renderables {
        let env = &ForEachDynEnvironment::from(env);

        let mut accumulated_height = 0;
        let mut renderables = Vec::new();

        for (item_layout, item) in layout.sublayouts.iter().zip(self.iter.as_ref().iter()) {
            let aligned_origin = origin
                + Point::new(
                    self.alignment.align(
                        layout.resolved_size.width.into(),
                        item_layout.resolved_size.width.into(),
                    ),
                    accumulated_height,
                );
            let view = item;
            renderables.push(view.render_tree(item_layout, aligned_origin, env));

            if !view.is_empty() {
                let item_height: i16 = item_layout.resolved_size.height.into();
                accumulated_height += item_height;
            }
        }

        VecWrapper(renderables)
    }
}

fn layout_dynamic(
    subviews: &[(i8, bool)],
    offer: ProposedDimensions,
    spacing: u16,
    mut layout_fn: impl FnMut(usize, ProposedDimensions) -> Dimensions,
) -> Dimensions {
    let ProposedDimension::Exact(height) = offer.height else {
        let mut total_height: Dimension = 0.into();
        let mut max_width: Dimension = 0.into();
        let mut non_empty_views: u16 = 0;
        for (i, (_, is_empty)) in subviews.iter().enumerate() {
            let dimensions = layout_fn(i, offer);
            if *is_empty {
                continue;
            }

            total_height += dimensions.height;
            max_width = max(max_width, dimensions.width);
            non_empty_views += 1;
        }
        return Dimensions {
            width: max_width,
            height: total_height + spacing * (non_empty_views.saturating_sub(1)),
        };
    };

    // compute the "flexibility" of each view on the vertical axis and sort by decreasing
    // flexibility
    // Flexibility is defined as the difference between the responses to 0 and infinite height offers
    let mut flexibilities: Vec<Dimension> = vec![0.into(); subviews.len()];
    let mut num_empty_views = 0;
    let min_proposal = ProposedDimensions {
        width: offer.width,
        height: ProposedDimension::Exact(0),
    };

    let max_proposal = ProposedDimensions {
        width: offer.width,
        height: ProposedDimension::Infinite,
    };

    for index in 0..subviews.len() {
        let minimum_dimension = layout_fn(index, min_proposal);
        // skip any further work for empty views
        if subviews[index].1 {
            num_empty_views += 1;
            continue;
        }
        let maximum_dimension = layout_fn(index, max_proposal);
        flexibilities[index] = maximum_dimension.height - minimum_dimension.height;
    }

    let mut remaining_height = height
        .saturating_sub(spacing * (subviews.len().saturating_sub(num_empty_views + 1)) as u16);
    let mut last_priority_group: Option<i8> = None;
    let mut max_width: Dimension = 0.into();
    loop {
        // collect the unsized subviews with the max layout priority into a group
        let mut subviews_indecies: Vec<usize> = vec![0; subviews.len()];
        let mut max = i8::MIN;
        let mut slice_start: usize = 0;
        let mut slice_len: usize = 0;
        for (i, (priority, is_empty)) in subviews.iter().enumerate() {
            if last_priority_group.is_some_and(|p| p <= *priority) || *is_empty {
                continue;
            }
            match max.cmp(priority) {
                core::cmp::Ordering::Less => {
                    max = *priority;
                    slice_start = i;
                    slice_len = 1;
                    subviews_indecies[slice_start] = i;
                }
                core::cmp::Ordering::Equal => {
                    if slice_len == 0 {
                        slice_start = i;
                    }

                    subviews_indecies[slice_start + slice_len] = i;
                    slice_len += 1;
                }
                core::cmp::Ordering::Greater => {}
            }
        }
        last_priority_group = Some(max);

        if slice_len == 0 {
            break;
        }

        let group_indecies = &mut subviews_indecies[slice_start..slice_start + slice_len];
        group_indecies.sort_unstable_by_key(|&i| flexibilities[i]);

        let mut remaining_group_size = group_indecies.len() as u16;

        for index in group_indecies {
            let height_fraction =
                remaining_height / remaining_group_size + remaining_height % remaining_group_size;
            let size = layout_fn(
                *index,
                ProposedDimensions {
                    width: offer.width,
                    height: ProposedDimension::Exact(height_fraction),
                },
            );
            remaining_height = remaining_height.saturating_sub(size.height.into());
            remaining_group_size -= 1;
            max_width = max_width.max(size.width);
        }
    }

    Dimensions {
        width: max_width,
        height: (height.saturating_sub(remaining_height)).into(),
    }
}

pub struct VecWrapper<V>(Vec<V>);

impl<T: AnimatedJoin> AnimatedJoin for VecWrapper<T> {
    fn join(source: Self, target: Self, domain: &AnimationDomain) -> Self {
        Self(
            source
                .0
                .into_iter()
                .zip(target.0)
                .map(|(source, target)| T::join(source, target, domain))
                .collect(),
        )
    }
}

impl<Color: PixelColor, T: EmbeddedGraphicsRender<Color>> EmbeddedGraphicsRender<Color>
    for VecWrapper<T>
{
    fn render(
        &self,
        render_target: &mut impl DrawTarget<Color = Color>,
        style: &Color,
        offset: Point,
    ) {
        self.0
            .iter()
            .for_each(|item| item.render(render_target, style, offset));
    }

    fn render_animated(
        render_target: &mut impl DrawTarget<Color = Color>,
        source: &Self,
        target: &Self,
        style: &Color,
        offset: Point,
        domain: &AnimationDomain,
    ) {
        source
            .0
            .iter()
            .zip(target.0.iter())
            .for_each(|(source, target)| {
                T::render_animated(render_target, source, target, style, offset, domain);
            });
    }
}
