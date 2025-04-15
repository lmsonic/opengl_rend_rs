use std::path::Path;

struct MeshData {}

pub struct Mesh {
    mesh_data: MeshData,
}

impl Drop for Mesh {
    fn drop(&mut self) {}
}

impl Mesh {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            mesh_data: MeshData {},
        }
    }
    pub fn render(&mut self) {}
    pub fn render_mesh(&mut self, mesh_name: String) {}
    pub fn delete_objects(&mut self) {}
}
