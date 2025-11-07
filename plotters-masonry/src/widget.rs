use accesskit::{Node, Role};
use masonry::core::{ArcStr, NoAction, Widget, WidgetMut};
use masonry::kurbo;
use plotters::coord::Shift;
use plotters::prelude::*;
use smallvec::SmallVec;

use crate::MasonryBackend;

pub type PlotFn<Data> = fn((u32, u32), &mut Data, &DrawingArea<MasonryBackend, Shift>);

/// The type of a plot widget.
///
/// See [`Plot::new`] for information on how to construct this.
///
/// This implements [`masonry::core::Widget`] so it can be used like
/// any other widget type.
/// ```rust
/// # use druid::{Widget, WindowDesc, AppLauncher};
/// # use plotters_druid::Plot;
/// fn build_plot_widget() -> impl Widget<()> {
///     // ... construct and return widget using Plot::new()
///     # Plot::new(|_, _, _|{})
/// }
///
/// # fn main() {
/// let main_window = WindowDesc::new(build_plot_widget());
/// # }
/// ```
pub struct Plot<Data> {
    alt_text: ArcStr,
    data: Data,
    plot: PlotFn<Data>,
}

impl<Data: 'static> Plot<Data> {
    /// Create a plot widget
    ///
    /// This takes a function that should draw the plot using the normal plotters API.
    /// The function has access to the width and height of the plotting area and to a
    /// plotters [`DrawingArea`].
    ///
    /// ```rust
    /// # use plotters_masonry::Plot;
    /// # use plotters::prelude::*;
    /// Plot::new(|(width, height), root| {
    ///     root.fill(&WHITE).unwrap();
    ///     let mut chart = ChartBuilder::on(&root)
    ///         .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)
    ///         .unwrap();
    ///
    ///     // see the plotters documentation on how to use `chart`
    /// });
    /// ```
    pub fn new(data: Data, plot: PlotFn<Data>, alt_text: impl Into<ArcStr>) -> Self {
        Self {
            alt_text: alt_text.into(),
            data,
            plot,
        }
    }

    pub fn set_data(this: &mut WidgetMut<'_, Self>, new_data: Data) {
        this.widget.data = new_data;
        this.ctx.request_paint_only();
    }

    pub fn set_plot(this: &mut WidgetMut<'_, Self>, new_plot: PlotFn<Data>) {
        this.widget.plot = new_plot;
        this.ctx.request_paint_only();
    }
}

impl<Data: 'static> Widget for Plot<Data> {
    type Action = NoAction;

    fn register_children(&mut self, _ctx: &mut masonry::core::RegisterCtx) {}

    fn layout(
        &mut self,
        _ctx: &mut masonry::core::LayoutCtx,
        _props: &mut masonry::core::PropertiesMut<'_>,
        bc: &masonry::core::BoxConstraints,
    ) -> masonry::kurbo::Size {
        bc.max()
    }

    fn paint(
        &mut self,
        ctx: &mut masonry::core::PaintCtx,
        _props: &masonry::core::PropertiesRef<'_>,
        scene: &mut masonry::vello::Scene,
    ) {
        let kurbo::Size { width, height } = ctx.size();
        let size = (width as u32, height as u32);
        let backend = MasonryBackend::new(size, scene, ctx);

        (self.plot)(size, &mut self.data, &backend.into_drawing_area());
    }

    fn accessibility_role(&self) -> Role {
        Role::Image
    }

    fn accessibility(
        &mut self,
        _ctx: &mut masonry::core::AccessCtx,
        _props: &masonry::core::PropertiesRef<'_>,
        node: &mut Node,
    ) {
        node.set_value(&*self.alt_text);
    }

    fn children_ids(&self) -> SmallVec<[masonry::core::WidgetId; 16]> {
        SmallVec::new()
    }
}
