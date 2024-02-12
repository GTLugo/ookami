use wgpu::{Color, CommandEncoder};

use super::{create_render_pipeline, Pass};
use crate::renderer::{context::GraphicsContext, mesh::Mesh, render_data::Drawable, target::RenderTarget, Renderer};

pub struct SimplePass {
  pipeline: wgpu::RenderPipeline,
}

impl SimplePass {
  pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, render_target: &RenderTarget) -> Self {
    let shader = wgpu::include_wgsl!("../../../assets/shaders/shader.wgsl");

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Simple Pipeline Layout"),
      bind_group_layouts: &[],
      push_constant_ranges: &[],
    });

    let pipeline = create_render_pipeline(
      Some("Simple Pipeline"),
      device,
      &pipeline_layout,
      RenderTarget::RENDER_TARGET_FORMAT,
      None,
      &[],
      shader,
    );

    Self { pipeline }
  }
}

impl Pass for SimplePass {
  fn draw(
    &mut self,
    command_encoder: &mut CommandEncoder,
    render_target: &wgpu::TextureView,
    mesh: &Mesh,
  ) -> Result<(), crate::error::RendererError> {
    let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("Simple Pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: render_target,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Load,
          store: wgpu::StoreOp::Store,
        },
      })],
      depth_stencil_attachment: None,
      occlusion_query_set: None,
      timestamp_writes: None,
    });

    render_pass.set_pipeline(&self.pipeline);
    render_pass.draw(0..3, 0..1);

    Ok(())
  }

  fn resize(&mut self, device: &wgpu::Device, render_target: &RenderTarget) {}
}
