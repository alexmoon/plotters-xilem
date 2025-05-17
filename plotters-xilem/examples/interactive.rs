use masonry::theme;
use plotters::prelude::*;
use plotters_xilem::plot;
use xilem::view::{
    Axis, CrossAxisAlignment, FlexExt, FlexSpacer, Label, MainAxisAlignment, button, flex, label,
    sized_box,
};
use xilem::winit::error::EventLoopError;
use xilem::{EventLoop, WidgetView, Xilem};

fn build_plot_view(mu: &mut f64) -> impl WidgetView<f64> + use<> {
    plot(
        *mu,
        |_size, data: &mut f64, root| {
            let μ = *data as f32;

            let res = 400;
            let font = FontDesc::new(FontFamily::SansSerif, 16., FontStyle::Normal);

            let mut chart = ChartBuilder::on(root)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .margin_right(10)
                .build_cartesian_2d(0.0..1_f32, 0.0..6_f32)
                .unwrap();

            chart
                .configure_mesh()
                .axis_style(RGBColor(28, 28, 28))
                .x_label_style(font.clone().with_color(WHITE))
                .y_label_style(font.clone().with_color(WHITE))
                .draw()
                .unwrap();

            for (σ, idx) in [0.32_f32, 0.56, 1., 1.78, 3.16].into_iter().zip(0..) {
                let fac = 1. / (σ * std::f32::consts::TAU.sqrt());
                let color = Palette99::pick(idx);

                let data = (0..res).map(|x| x as f32 / res as f32).map(|x| {
                    let y =
                        fac * (-(logit(x) - μ).powi(2) / (2. * σ.powi(2))).exp() / (x * (1. - x));
                    (x, y)
                });

                chart
                    .draw_series(LineSeries::new(data, &color))
                    .unwrap()
                    .label(format!("σ = {σ}"))
                    .legend(move |(x, y)| {
                        PathElement::new(
                            vec![(x, y), (x + 20, y)],
                            ShapeStyle::from(&color).stroke_width(2),
                        )
                    });
            }
            chart
                .configure_series_labels()
                .position(SeriesLabelPosition::UpperRight)
                .background_style(RGBColor(41, 41, 41))
                .border_style(RGBColor(28, 28, 28))
                .label_font(font.with_color(WHITE))
                .draw()
                .unwrap();
        },
        "Logit-Normal plot",
    )
}

/// A component to make a bigger than usual button
fn big_button(
    label: impl Into<Label>,
    callback: impl Fn(&mut f64) + Send + Sync + 'static,
) -> impl WidgetView<f64> {
    sized_box(button(label, callback)).width(40.).height(40.)
}

fn app_logic(mu: &mut f64) -> impl WidgetView<f64> + use<> {
    sized_box(flex((
        build_plot_view(mu).flex(1.),
        FlexSpacer::Fixed(5.),
        flex((
            FlexSpacer::Fixed(30.0),
            big_button("-", |mu| {
                *mu = (*mu - 0.1).max(-3.);
            }),
            FlexSpacer::Flex(1.0),
            label(format!("μ: {:.1}", mu)).text_size(32.).flex(5.0),
            FlexSpacer::Flex(1.0),
            big_button("+", |mu| {
                *mu = (*mu + 0.1).min(3.);
            }),
            FlexSpacer::Fixed(30.0),
        ))
        .direction(Axis::Horizontal)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .main_axis_alignment(MainAxisAlignment::Center)
        .must_fill_major_axis(true),
    )))
    .padding(10.)
}

fn main() -> Result<(), EventLoopError> {
    let app = Xilem::new(0.8, app_logic).background_color(theme::BACKGROUND_DARK);
    app.run_windowed(
        EventLoop::with_user_event(),
        "Logit-Normal Distribution".into(),
    )
}

fn logit(p: f32) -> f32 {
    (p / (1. - p)).ln()
}
