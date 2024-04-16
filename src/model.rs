use wgpu::util::DeviceExt;
use tobj;
use cgmath::{InnerSpace, Vector3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	pub position: [f32; 3],
	pub normal: [f32; 3],
}

impl Vertex {
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }
	}
}

#[derive(Debug)]
pub struct BoundingBox {
	min: Vector3<f32>,
	max: Vector3<f32>,
}

impl BoundingBox {
	pub fn center(&self) -> Vector3<f32> {
		(self.min + self.max) / 2.0
	}

	pub fn diag(&self) -> f32 {
		(self.max - self.min).magnitude()
	}
}

pub struct Mesh {
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	pub bounding_box: BoundingBox,
	n: u32,
}

impl Mesh {
	pub fn from_obj(device: &wgpu::Device, filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
		let (models, _) = tobj::load_obj(filename, &tobj::GPU_LOAD_OPTIONS)?;

		let mut vertices: Vec<Vertex> = vec![];
		let mut indices: Vec<u32> = vec![];
		let mut off: u32 = 0;
		let mut min_bound = Vector3::new(0.0_f32, 0.0_f32, 0.0_f32);
		let mut max_bound = Vector3::new(0.0_f32, 0.0_f32, 0.0_f32);
		let mut first = true;
		for m in models.iter() {
			let mesh = &m.mesh;
			
			indices.extend(mesh.indices.iter().map(|i| (i + off) as u32));
			
			let n = mesh.positions.len() / 3;
			for i in 0..n {
				let pos = [mesh.positions[(i*3) as usize], mesh.positions[(i*3+1) as usize], mesh.positions[(i*3+2) as usize]];
				if first {
					first = false;
					min_bound = Vector3::new(pos[0], pos[1], pos[2]);
					max_bound = Vector3::new(pos[0], pos[1], pos[2]);
				} else {
					min_bound = Vector3::new(min_bound.x.min(pos[0]), min_bound.y.min(pos[1]), min_bound.z.min(pos[2]));
					max_bound = Vector3::new(max_bound.x.max(pos[0]), max_bound.y.max(pos[1]), max_bound.z.max(pos[2]));
				}

				vertices.push(Vertex {
					position: pos,
					normal: [mesh.normals[(i*3) as usize], mesh.normals[(i*3+1) as usize], mesh.normals[(i*3+2) as usize]],
				});
			}
			
			off += n as u32;
		}

		let vertex_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Vertex Buffer"),
				contents: bytemuck::cast_slice(vertices.as_slice()),
				usage: wgpu::BufferUsages::VERTEX,
			}
		);

		let index_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Index Buffer"),
				contents: bytemuck::cast_slice(indices.as_slice()),
				usage: wgpu::BufferUsages::INDEX,
			}
		);

		Ok(Self {
			vertex_buffer,
			index_buffer,
			bounding_box: BoundingBox{
				min: min_bound, 
				max: max_bound
			},
			n: indices.len() as u32,
		})
	}

	pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
		render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
		render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
		render_pass.draw_indexed(0..self.n, 0, 0..1);
	}
} 