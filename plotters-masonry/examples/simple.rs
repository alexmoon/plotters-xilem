use masonry::core::{NewWidget, WidgetId};
use masonry::theme::default_property_set;
use masonry_winit::app::{AppDriver, DriverCtx, NewWindow};
use plotters::prelude::*;
use plotters_masonry::Plot;
use winit::dpi::LogicalSize;
use winit::window::Window;

struct Driver;

impl AppDriver for Driver {
    fn on_action(
        &mut self,
        _window_id: masonry_winit::app::WindowId,
        _ctx: &mut DriverCtx<'_, '_>,
        _widget_id: WidgetId,
        _action: masonry::core::ErasedAction,
    ) {
    }
}

fn main() {
    let plot = Plot::new(
        (),
        |_size, _, root| {
            // Code taken from the plotters example: https://github.com/38/plotters#quick-start
            root.fill(&WHITE).unwrap();
            let mut chart = ChartBuilder::on(root)
                .caption("y=x^2", ("sans-serif", 50).into_font())
                .margin(5)
                .margin_right(15)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)
                .unwrap();

            chart.configure_mesh().draw().unwrap();

            chart
                .draw_series(LineSeries::new(
                    (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
                    &RED,
                ))
                .unwrap()
                .label("y = x^2")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

            chart
                .configure_series_labels()
                .background_style(WHITE.mix(0.8))
                .border_style(BLACK)
                .draw()
                .unwrap();
        },
        "simple plot",
    );

    let window_size = LogicalSize::new(400., 400.);
    let window_attributes = Window::default_attributes()
        .with_title("Hello Plot!")
        .with_resizable(true)
        .with_min_inner_size(window_size);

    let new_window = NewWindow::new(window_attributes, NewWidget::new(plot).erased());

    masonry_winit::app::run(
        masonry_winit::app::EventLoop::with_user_event(),
        vec![new_window],
        Driver,
        default_property_set(),
    )
    .unwrap();
}
