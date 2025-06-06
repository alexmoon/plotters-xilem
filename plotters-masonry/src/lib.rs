/*!

Use [Plotters](https://crates.io/crates/plotters) to draw plots in [Masonry](https://crates.io/crates/masonry).

All the features of plotters should just work. Additionally, transparency is also supported, i.e. you don't
have to fill the background with a solid colour as is usually done in plotters examples, the background can
instead just be whatever background colour is given through masonry.

You'll mainly need [`Plot::new`] from this crate.

# Example

For more complete examples see [the GitHub repo](https://github.com/alexmoon/plotters-xilem)

```rust
# use druid::{Widget, WindowDesc, AppLauncher};
# use plotters_druid::Plot;
# use plotters::prelude::*;
# #[derive(Clone, druid::Data)]
# struct AppState;
fn build_plot_widget() -> impl Widget<AppState> {
    Plot::new(|(width, height), data: &AppState, root| {
        root.fill(&WHITE).unwrap();
        let mut chart = ChartBuilder::on(&root)
            .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)
            .unwrap();

        // see the plotters documentation on how to use `chart`
    })
}

# fn main() {
let main_window = WindowDesc::new(build_plot_widget());
# }
```

# Limitations

It's currently not possible to propagate errors that might be returned from the plotters API. Right now
this means you'll probably have to use `.unwrap()` a lot in the closure that you pass to [`Plot::new`],
or alternatively just log it and shrug.

The possible errors in there mostly come from the drawing backend, e.g. cairo / direct2d / whatever piet
uses on your platform. Just directly propagating these in the widget's draw function doesn't really make
sense because it's not clear what masonry is supposed to do with these errors. Ideally we'd probably change
something in the data to notify the rest of the application of the error. If anyone has a good suggestion
for a possible API feel free to open an issue.

*/

use std::borrow::Cow;
use std::cell::RefCell;
use std::f64::consts::PI;

use masonry::core::PaintCtx;
use masonry::kurbo::{Affine, Vec2};
use masonry::parley;
use masonry::vello::Scene;
use plotters::prelude::*;
use plotters_backend::text_anchor::{HPos, VPos};
use plotters_backend::{BackendColor, BackendCoord, DrawingErrorKind};
use plotters_vello::VelloBackend;

mod widget;

pub use widget::*;

/// The Masonry backend.
///
/// Note that the size of the Masonry scene has to be specified here.
pub struct MasonryBackend<'a, 'b> {
    vello_backend: VelloBackend<'a>,
    ctx: RefCell<&'a mut PaintCtx<'b>>,
}

impl std::fmt::Debug for MasonryBackend<'_, '_> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("MasonryBackend")
            .field("size", &self.vello_backend.get_size())
            .finish()
    }
}

impl<'a, 'b> MasonryBackend<'a, 'b> {
    pub fn new(size: (u32, u32), scene: &'a mut Scene, ctx: &'a mut PaintCtx<'b>) -> Self {
        Self {
            vello_backend: VelloBackend::new(size, scene),
            ctx: RefCell::new(ctx),
        }
    }
}

impl DrawingBackend for MasonryBackend<'_, '_> {
    type ErrorType = plotters_vello::Error;

    fn get_size(&self) -> (u32, u32) {
        self.vello_backend.get_size()
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.vello_backend.draw_pixel(point, color)
    }

    fn draw_line<S: plotters_backend::BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.vello_backend.draw_line(from, to, style)
    }

    fn draw_rect<S: plotters_backend::BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.vello_backend
            .draw_rect(upper_left, bottom_right, style, fill)
    }

    fn draw_path<S: plotters_backend::BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.vello_backend.draw_path(path, style)
    }

    fn draw_circle<S: plotters_backend::BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.vello_backend.draw_circle(center, radius, style, fill)
    }

    fn fill_polygon<S: plotters_backend::BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.vello_backend.draw_path(vert, style)
    }

    fn draw_text<TStyle: plotters_backend::BackendTextStyle>(
        &mut self,
        text: &str,
        style: &TStyle,
        pos: BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        use masonry::core::render_text;
        use parley::{FontStack, FontWeight, GenericFamily, PlainEditor, StyleProperty};

        let mut editor = PlainEditor::new(style.size() as f32);
        editor.set_text(text);

        let styles = editor.edit_styles();
        let family = match style.family() {
            FontFamily::Serif => parley::FontFamily::Generic(GenericFamily::UiSerif),
            FontFamily::SansSerif => parley::FontFamily::Generic(GenericFamily::UiSansSerif),
            FontFamily::Monospace => parley::FontFamily::Generic(GenericFamily::UiMonospace),
            FontFamily::Name(name) => parley::FontFamily::Named(Cow::Owned(name.to_owned())),
        };
        styles.insert(StyleProperty::FontStack(FontStack::Single(family)));
        styles.insert(match style.style() {
            FontStyle::Normal => StyleProperty::FontStyle(parley::FontStyle::Normal),
            FontStyle::Oblique => StyleProperty::FontStyle(parley::FontStyle::Oblique(None)),
            FontStyle::Italic => StyleProperty::FontStyle(parley::FontStyle::Italic),
            FontStyle::Bold => StyleProperty::FontWeight(FontWeight::BOLD),
        });

        let mut ctx = self.ctx.borrow_mut();
        let (font_ctx, layout_ctx) = ctx.text_contexts();
        let layout = editor.layout(font_ctx, layout_ctx);
        let (width, height) = (f64::from(layout.full_width()), f64::from(layout.height()));

        // Center on the origin
        let transform = Affine::translate(Vec2::new(-width / 2., -height / 2.));

        // Rotate based on FontTransform
        let (width, height, transform) = match style.transform() {
            FontTransform::None => (width, height, transform),
            FontTransform::Rotate90 => (height, width, transform.then_rotate(PI / 2.)),
            FontTransform::Rotate180 => (width, height, transform.then_rotate(PI)),
            FontTransform::Rotate270 => (height, width, transform.then_rotate(-PI / 2.)),
        };

        // Move the anchor to the origin
        let transform = match style.anchor().h_pos {
            HPos::Left => transform.then_translate(Vec2::new(width / 2., 0.)),
            HPos::Center => transform,
            HPos::Right => transform.then_translate(Vec2::new(-width / 2., 0.)),
        };
        let transform = match style.anchor().v_pos {
            VPos::Top => transform.then_translate(Vec2::new(0., height / 2.)),
            VPos::Center => transform,
            VPos::Bottom => transform.then_translate(Vec2::new(0., -height / 2.)),
        };

        // Move to pos
        let transform = transform.then_translate(Vec2::new(f64::from(pos.0), f64::from(pos.1)));

        let color = plotters_vello::plotters_color_to_peniko(&style.color());
        render_text(
            self.vello_backend.scene(),
            transform,
            layout,
            &[color.into()],
            true,
        );

        Ok(())
    }

    fn estimate_text_size<TStyle: plotters_backend::BackendTextStyle>(
        &self,
        text: &str,
        style: &TStyle,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        use parley::{FontStack, FontWeight, GenericFamily, PlainEditor, StyleProperty};

        let mut editor = PlainEditor::new(style.size() as f32);
        editor.set_text(text);

        let styles = editor.edit_styles();
        let family = match style.family() {
            FontFamily::Serif => parley::FontFamily::Generic(GenericFamily::UiSerif),
            FontFamily::SansSerif => parley::FontFamily::Generic(GenericFamily::UiSansSerif),
            FontFamily::Monospace => parley::FontFamily::Generic(GenericFamily::UiMonospace),
            FontFamily::Name(name) => parley::FontFamily::Named(Cow::Owned(name.to_owned())),
        };
        styles.insert(StyleProperty::FontStack(FontStack::Single(family)));
        styles.insert(match style.style() {
            FontStyle::Normal => StyleProperty::FontStyle(parley::FontStyle::Normal),
            FontStyle::Oblique => StyleProperty::FontStyle(parley::FontStyle::Oblique(None)),
            FontStyle::Italic => StyleProperty::FontStyle(parley::FontStyle::Italic),
            FontStyle::Bold => StyleProperty::FontWeight(FontWeight::BOLD),
        });

        let mut ctx = self.ctx.borrow_mut();
        let (font_ctx, layout_ctx) = ctx.text_contexts();
        let layout = editor.layout(font_ctx, layout_ctx);
        let (width, height) = (
            layout.full_width().ceil() as u32,
            layout.height().ceil() as u32,
        );

        Ok(match style.transform() {
            FontTransform::None | FontTransform::Rotate180 => (width, height),
            FontTransform::Rotate90 | FontTransform::Rotate270 => (height, width),
        })
    }

    fn blit_bitmap(
        &mut self,
        pos: BackendCoord,
        size: (u32, u32),
        src: &[u8],
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.vello_backend.blit_bitmap(pos, size, src)
    }
}
