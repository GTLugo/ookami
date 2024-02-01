// pub static SHADERS: OnceLock<ShaderStore> = OnceLock::new();

use std::{collections::HashMap, path::PathBuf};

use foxy_utils::types::handle::Handle;

use super::{
  stage::{compute::Compute, fragment::Fragment, geometry::Geometry, mesh::Mesh, vertex::Vertex, StageInfo},
  Shader,
};
use crate::vulkan::device::Device;

#[allow(dead_code)]
pub struct ShaderStore {
  device: Device,
  vertex_shaders: HashMap<PathBuf, Handle<Shader<Vertex>>>,
  fragment_shaders: HashMap<PathBuf, Handle<Shader<Fragment>>>,
  compute_shaders: HashMap<PathBuf, Handle<Shader<Compute>>>,
  geometry_shaders: HashMap<PathBuf, Handle<Shader<Geometry>>>,
  mesh_shaders: HashMap<PathBuf, Handle<Shader<Mesh>>>,
}

impl ShaderStore {
  pub fn delete(&mut self) {
    for shader in self.vertex_shaders.values_mut() {
      shader.get_mut().delete();
    }
    for shader in self.fragment_shaders.values_mut() {
      shader.get_mut().delete();
    }
    for shader in self.compute_shaders.values_mut() {
      shader.get_mut().delete();
    }
    for shader in self.geometry_shaders.values_mut() {
      shader.get_mut().delete();
    }
    for shader in self.mesh_shaders.values_mut() {
      shader.get_mut().delete();
    }
  }
}

impl ShaderStore {
  pub const SHADER_ASSET_DIR: &'static str = "assets/shaders";
  pub const SHADER_CACHE_DIR: &'static str = "tmp/shaders";

  pub fn new(device: Device) -> Self {
    Self {
      device,
      vertex_shaders: Default::default(),
      fragment_shaders: Default::default(),
      compute_shaders: Default::default(),
      geometry_shaders: Default::default(),
      mesh_shaders: Default::default(),
    }
  }

  pub fn get_vertex<P: Into<PathBuf>>(&mut self, path: P) -> Handle<Shader<Vertex>> {
    Self::get_shader(&self.device, &mut self.vertex_shaders, path)
  }

  pub fn get_fragment<P: Into<PathBuf>>(&mut self, path: P) -> Handle<Shader<Fragment>> {
    Self::get_shader(&self.device, &mut self.fragment_shaders, path)
  }

  pub fn get_compute<P: Into<PathBuf>>(&mut self, path: P) -> Handle<Shader<Compute>> {
    Self::get_shader(&self.device, &mut self.compute_shaders, path)
  }

  pub fn get_geometry<P: Into<PathBuf>>(&mut self, path: P) -> Handle<Shader<Geometry>> {
    Self::get_shader(&self.device, &mut self.geometry_shaders, path)
  }

  pub fn get_mesh<P: Into<PathBuf>>(&mut self, path: P) -> Handle<Shader<Mesh>> {
    Self::get_shader(&self.device, &mut self.mesh_shaders, path)
  }

  fn get_shader<Stage: StageInfo + Clone, P: Into<PathBuf>>(
    device: &Device,
    shader_map: &mut HashMap<PathBuf, Handle<Shader<Stage>>>,
    path: P,
  ) -> Handle<Shader<Stage>> {
    let path: PathBuf = path.into();
    match shader_map.get(&path).cloned() {
      Some(shader) => shader.clone(),
      None => {
        let shader = Handle::new(Shader::new(device.clone(), path.clone()));
        shader_map.insert(path, shader.clone());
        shader
      }
    }
  }
}
