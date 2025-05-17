use std::fs::File;

use plotters::prelude::*;
use plotters_vello::VelloBackend;
use vello::Scene;
use vello::peniko::color::palette::css;
use vello::wgpu::{
    self, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, TexelCopyBufferInfo,
    TextureDescriptor, TextureFormat, TextureUsages,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let width = 1920;
    let height = 1080;

    // Create rendering context
    let mut context = vello::util::RenderContext::new();
    let device_id = context.device(None).await.unwrap();
    let device_handle = &mut context.devices[device_id];
    let device = &device_handle.device;
    let queue = &device_handle.queue;
    let mut renderer = vello::Renderer::new(device, Default::default())?;
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

    // Wrapping this in its own scope because we need to release the borrow on  `bitmap`
    // before we try to save the png at the end.
    let mut scene = Scene::new();
    {
        let vello_backend = VelloBackend::new((width, height), &mut scene);

        let root = vello_backend.into_drawing_area();

        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption("y=x^2", ("sans-serif", 50).into_font())
            .margin(5)
            .margin_right(15)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)?;

        chart.configure_mesh().draw()?;

        chart
            .draw_series(LineSeries::new(
                (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
                &RED,
            ))?
            .label("y = x^2")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

        chart
            .configure_series_labels()
            .background_style(WHITE)
            .border_style(BLACK)
            .draw()?;

        root.present()?;
    }

    renderer
        .render_to_texture(device, queue, &scene, &view, &render_params)
        .unwrap();

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
    vello::util::block_on_wgpu(device, receiver)??;

    // Remove row padding from the buffer
    let data = buf_slice.get_mapped_range();
    let mut result_unpadded = Vec::<u8>::with_capacity((width * height * 4).try_into()?);
    for row in 0..height {
        let start = (row * padded_byte_width).try_into()?;
        result_unpadded.extend(&data[start..start + (width * 4) as usize]);
    }

    let mut file = File::create("plot.png")?;
    let mut png_encoder = png::Encoder::new(&mut file, width, height);
    png_encoder.set_color(png::ColorType::Rgba);
    png_encoder.set_depth(png::BitDepth::Eight);
    let mut writer = png_encoder.write_header()?;
    writer.write_image_data(&result_unpadded)?;
    writer.finish()?;
    println!("Wrote result ({width}x{height}) to plot.png");

    Ok(())
}
