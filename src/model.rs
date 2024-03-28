use wgpu::util::DeviceExt;
use std::{fmt::Error, fs::File};
use std::io::BufReader;
use tobj;

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

const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0], normal: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], normal: [0.0, 1.0, 0.0] },
    Vertex { position: [0.5, -0.5, 0.0], normal: [0.0, 0.0, 1.0] },
];

const INDICES: &[u16] = &[
    0, 1, 2
];

pub struct Mesh {
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	n: u32,
}

impl Mesh {
	pub fn new(device: &wgpu::Device) -> Self {
		let vertex_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Vertex Buffer"),
				contents: bytemuck::cast_slice(VERTICES),
				usage: wgpu::BufferUsages::VERTEX,
			}
		);

		let index_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Index Buffer"),
				contents: bytemuck::cast_slice(INDICES),
				usage: wgpu::BufferUsages::INDEX,
			}
		);

		Self {
			vertex_buffer,
			index_buffer,
			n: INDICES.len() as u32,
		}
	}

	pub fn from_obj(device: &wgpu::Device) -> Result<Self, Box<dyn std::error::Error>> {
		let (models, materials) = tobj::load_obj("models/tree.obj", &tobj::GPU_LOAD_OPTIONS)?;

		let mut vertices: Vec<Vertex> = vec![];
		let mut indices: Vec<u16> = vec![];
		let mut off: u32 = 0;
		for m in models.iter() {
			let mesh = &m.mesh;
			
			indices.extend(mesh.indices.iter().map(|i| (i + off) as u16));
			
			let n = mesh.positions.len() / 3;
			for i in 0..n {
				vertices.push(Vertex {
					position: [mesh.positions[(i*3) as usize], mesh.positions[(i*3+1) as usize], mesh.positions[(i*3+2) as usize]],
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
			n: indices.len() as u32,
		})
	}

	pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
		render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
		render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
		render_pass.draw_indexed(0..self.n, 0, 0..1);
	}
} 