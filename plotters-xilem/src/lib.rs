/*!

Use [Plotters](https://crates.io/crates/plotters) to draw plots in [Xilem](https://crates.io/crates/xilem).

All the features of plotters should just work. Additionally, transparency is also supportet, i.e. you don't
have to fill the background with a solid colour as is usually done in plotters examples, the background can
instead just be whatever background colour is given through Xilem.

Note that this is not directly a plotters backend in the sense described in
[plotters_backend](https://docs.rs/plotters-backend/latest/plotters_backend/), instead this uses
the plotters-masonry backend and wraps it in a struct that implements [`xilem_core::View`].

# Limitations

It's currently not possible to propagate errors that might be returned from the plotters API. Right now
this means you'll probably have to use `.unwrap()` a lot in the closure that you pass to [`Plot::new`],
or alternatively just log it and shrug.

*/

use masonry::core::ArcStr;
use plotters_masonry::{Plot as PlotWidget, PlotFn};
use xilem::core::{MessageResult, View, ViewMarker};
use xilem::{Pod, ViewCtx};

pub fn plot<Data>(data: Data, plot: PlotFn<Data>, alt_text: impl Into<ArcStr>) -> Plot<Data>
where
    Data: Clone + PartialEq,
{
    Plot {
        data,
        plot,
        alt_text: alt_text.into(),
    }
}

pub struct Plot<Data> {
    data: Data,
    plot: PlotFn<Data>,
    alt_text: ArcStr,
}

impl<Data> ViewMarker for Plot<Data> {}

impl<State, Data: Clone + PartialEq + 'static> View<State, (), ViewCtx> for Plot<Data> {
    type Element = Pod<PlotWidget<Data>>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx) -> (Self::Element, Self::ViewState) {
        let widget_pod = ctx.new_pod(PlotWidget::new(
            self.data.clone(),
            self.plot,
            self.alt_text.clone(),
        ));
        (widget_pod, ())
    }

    fn rebuild(
        &self,
        prev: &Self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        mut element: xilem::core::Mut<'_, Self::Element>,
    ) {
        if prev.data != self.data {
            PlotWidget::set_data(&mut element, self.data.clone());
        }
        if !std::ptr::fn_addr_eq(prev.plot, self.plot) {
            PlotWidget::set_plot(&mut element, self.plot);
        }
    }

    fn teardown(
        &self,
        _view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        element: xilem::core::Mut<'_, Self::Element>,
    ) {
        ctx.teardown_leaf(element);
    }

    fn message(
        &self,
        _view_state: &mut Self::ViewState,
        _id_path: &[xilem::core::ViewId],
        message: xilem::core::DynMessage,
        _app_state: &mut State,
    ) -> xilem::core::MessageResult<(), xilem::core::DynMessage> {
        tracing::error!(
            "Message arrived in Plot::message, but Plot doesn't consume any messages, this is a bug"
        );
        MessageResult::Stale(message)
    }
}
