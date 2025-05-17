/*!
A [Piet](https://crates.io/crates/piet) backend for [Plotters](https://crates.io/crates/plotters). This lets you draw plots on a Piet render context.
*/

use plotters_backend::{BackendColor, BackendCoord, DrawingBackend, DrawingErrorKind};
use vello::{Scene, kurbo, peniko};

#[derive(Debug, PartialEq, Eq)]
pub struct Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "plotters-vello error")
    }
}

impl std::error::Error for Error {}

/// The Vello backend.
///
/// Note that the size of the Vello scene has to be specified here.
pub struct VelloBackend<'a> {
    size: (u32, u32),
    scene: &'a mut Scene,
}

impl std::fmt::Debug for VelloBackend<'_> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("VelloBackend")
            .field("size", &self.size)
            .finish()
    }
}

impl<'a> VelloBackend<'a> {
    pub fn new(size: (u32, u32), scene: &'a mut Scene) -> Self {
        Self { size, scene }
    }

    #[doc(hidden)]
    pub fn scene(&mut self) -> &mut Scene {
        self.scene
    }
}

impl DrawingBackend for VelloBackend<'_> {
    type ErrorType = Error;

    fn get_size(&self) -> (u32, u32) {
        self.size
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
        let x = point.0 as f64;
        let y = point.1 as f64;
        self.scene.fill(
            peniko::Fill::NonZero,
            kurbo::Affine::IDENTITY,
            plotters_color_to_peniko(&color),
            None,
            &kurbo::Rect::new(x, y, x + 1., y + 1.),
        );
        Ok(())
    }

    fn draw_line<S: plotters_backend::BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let from = plotters_point_to_kurbo_mid(from);
        let to = plotters_point_to_kurbo_mid(to);

        self.scene.stroke(
            &kurbo::Stroke::new(style.stroke_width() as f64).with_end_cap(kurbo::Cap::Square),
            kurbo::Affine::IDENTITY,
            plotters_color_to_peniko(&style.color()),
            None,
            &kurbo::Line::new(from, to),
        );
        Ok(())
    }

    fn draw_rect<S: plotters_backend::BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let color = plotters_color_to_peniko(&style.color());

        if fill {
            let upper_left = plotters_point_to_kurbo_corner(upper_left);
            let mut bottom_right = plotters_point_to_kurbo_corner(bottom_right);
            bottom_right.x += 1.;
            bottom_right.y += 1.;
            let rect = kurbo::Rect::new(upper_left.x, upper_left.y, bottom_right.x, bottom_right.y);

            self.scene.fill(
                peniko::Fill::NonZero,
                kurbo::Affine::IDENTITY,
                color,
                None,
                &rect,
            );
        } else {
            let upper_left = plotters_point_to_kurbo_mid(upper_left);
            let bottom_right = plotters_point_to_kurbo_mid(bottom_right);
            let rect = kurbo::Rect::new(upper_left.x, upper_left.y, bottom_right.x, bottom_right.y);

            self.scene.stroke(
                &kurbo::Stroke::new(style.stroke_width() as f64).with_end_cap(kurbo::Cap::Square),
                kurbo::Affine::IDENTITY,
                color,
                None,
                &rect,
            );
        }

        Ok(())
    }

    fn draw_path<S: plotters_backend::BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }

        let path: Vec<kurbo::PathEl> = plotters_path_to_kurbo(path).collect();

        self.scene.stroke(
            &kurbo::Stroke::new(style.stroke_width() as f64).with_end_cap(kurbo::Cap::Square),
            kurbo::Affine::IDENTITY,
            plotters_color_to_peniko(&style.color()),
            None,
            &kurbo::BezPath::from_vec(path),
        );
        Ok(())
    }

    fn draw_circle<S: plotters_backend::BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let center = plotters_point_to_kurbo_mid(center);
        let color = plotters_color_to_peniko(&style.color());
        let circle = kurbo::Circle::new(center, radius as f64);

        if fill {
            self.scene.fill(
                peniko::Fill::NonZero,
                kurbo::Affine::IDENTITY,
                color,
                None,
                &circle,
            );
        } else {
            self.scene.stroke(
                &kurbo::Stroke::new(style.stroke_width() as f64).with_end_cap(kurbo::Cap::Square),
                kurbo::Affine::IDENTITY,
                color,
                None,
                &circle,
            );
        }
        Ok(())
    }

    fn fill_polygon<S: plotters_backend::BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }

        let path: Vec<kurbo::PathEl> = plotters_path_to_kurbo(vert)
            .chain(std::iter::once(kurbo::PathEl::ClosePath))
            .collect();
        self.scene.fill(
            peniko::Fill::NonZero,
            kurbo::Affine::IDENTITY,
            plotters_color_to_peniko(&style.color()),
            None,
            &kurbo::BezPath::from_vec(path),
        );

        Ok(())
    }

    fn blit_bitmap(
        &mut self,
        pos: BackendCoord,
        (iw, ih): (u32, u32),
        src: &[u8],
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let data = src.to_vec();
        let image = peniko::Image::new(data.into(), peniko::ImageFormat::Rgba8, iw, ih);
        let transform = kurbo::Affine::translate((pos.0 as f64, pos.1 as f64));
        self.scene.draw_image(&image, transform);
        Ok(())
    }
}

#[doc(hidden)]
pub fn plotters_color_to_peniko(col: &BackendColor) -> peniko::Color {
    peniko::Color::from_rgba8(col.rgb.0, col.rgb.1, col.rgb.2, (col.alpha * 256.) as u8)
}

#[doc(hidden)]
pub fn plotters_point_to_kurbo_mid((x, y): BackendCoord) -> kurbo::Point {
    kurbo::Point {
        x: x as f64 + 0.5,
        y: y as f64 + 0.5,
    }
}

#[doc(hidden)]
pub fn plotters_point_to_kurbo_corner((x, y): BackendCoord) -> kurbo::Point {
    kurbo::Point {
        x: x as f64,
        y: y as f64,
    }
}

/// This is basically just an iterator map that applies a different function on
/// the first item as on the later items.
/// We need this because the piet direct2d backend doesn't like it if a path
/// consists entirely of `LineTo` entries, it requires the first entry to be
/// a `MoveTo` entry.
struct PlottersPathToKurbo<I> {
    iter: I,
    first: bool,
}

impl<I> PlottersPathToKurbo<I> {
    fn new(path: I) -> PlottersPathToKurbo<I> {
        PlottersPathToKurbo {
            iter: path,
            first: true,
        }
    }
}

impl<I> Iterator for PlottersPathToKurbo<I>
where
    I: Iterator<Item = BackendCoord>,
{
    type Item = kurbo::PathEl;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|point| {
            let point = plotters_point_to_kurbo_mid(point);

            if self.first {
                self.first = false;
                kurbo::PathEl::MoveTo(point)
            } else {
                kurbo::PathEl::LineTo(point)
            }
        })
    }
}

fn plotters_path_to_kurbo(
    path: impl IntoIterator<Item = BackendCoord>,
) -> impl Iterator<Item = kurbo::PathEl> {
    PlottersPathToKurbo::new(path.into_iter())
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotters::prelude::*;
    use vello::peniko::color::palette::css;
    use vello::wgpu::{
        BufferDescriptor, BufferUsages, CommandEncoderDescriptor, TexelCopyBufferInfo,
        TextureDescriptor, TextureFormat, TextureUsages,
    };
    use vello::{Scene, wgpu};

    #[tokio::test]
    async fn fill_root_white() {
        let width = 3;
        let height = 2;

        // Create rendering context
        let mut context = vello::util::RenderContext::new();
        let device_id = context.device(None).await.unwrap();
        let device_handle = &mut context.devices[device_id];
        let device = &device_handle.device;
        let queue = &device_handle.queue;
        let mut renderer = vello::Renderer::new(device, Default::default()).unwrap();
        let render_params = vello::RenderParams {
            base_color: css::BLACK,
            width,
            height,
            antialiasing_method: vello::AaConfig::Area,
        };
        let size = vello::wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let target = device.create_texture(&TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());

        // Create the scene
        {
            let mut scene = Scene::new();

            let vello_backend = VelloBackend {
                size: (width, height),
                scene: &mut scene,
            };

            let root = vello_backend.into_drawing_area();
            root.fill(&WHITE).unwrap();

            renderer
                .render_to_texture(device, queue, &scene, &view, &render_params)
                .unwrap();
        }

        // Copy the texture to host memory
        let padded_byte_width = (width * 4).next_multiple_of(256);
        let buffer_size = padded_byte_width as u64 * height as u64;
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("val"),
            size: buffer_size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Copy out buffer"),
        });
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_byte_width),
                    rows_per_image: None,
                },
            },
            size,
        );
        queue.submit([encoder.finish()]);
        let buf_slice = buffer.slice(..);

        // Wait for the copy to complete
        let (sender, receiver) = tokio::sync::oneshot::channel();
        buf_slice.map_async(wgpu::MapMode::Read, |res| sender.send(res).unwrap());
        vello::util::block_on_wgpu(device, receiver)
            .unwrap()
            .unwrap();

        // Remove row padding from the buffer
        let data = buf_slice.get_mapped_range();
        let mut result_unpadded =
            Vec::<u8>::with_capacity((width * height * 4).try_into().unwrap());
        for row in 0..height {
            let start = (row * padded_byte_width).try_into().unwrap();
            result_unpadded.extend(&data[start..start + (width * 4) as usize]);
        }

        assert_eq!(&result_unpadded, &[255; 6 * 4]);
    }

    #[test]
    fn test_plotters_path_to_kurbo() {
        let path = vec![(1, 2), (3, 4), (5, 6)];

        let kurbo_path: Vec<kurbo::PathEl> = plotters_path_to_kurbo(path).collect();

        assert_eq!(
            kurbo_path,
            vec![
                kurbo::PathEl::MoveTo(kurbo::Point { x: 1.5, y: 2.5 }),
                kurbo::PathEl::LineTo(kurbo::Point { x: 3.5, y: 4.5 }),
                kurbo::PathEl::LineTo(kurbo::Point { x: 5.5, y: 6.5 }),
            ]
        );
    }
}
