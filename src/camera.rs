use cgmath::{Deg, InnerSpace, Matrix4, One, Rad, SquareMatrix, Vector2, Vector3, Vector4};
use winit::{dpi::PhysicalPosition, event::*};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MatrixUniform {
    data: [[f32; 4]; 4],
}

impl MatrixUniform {
	pub fn new() -> Self {
        Self {
            data: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn from_Matrix4(mat: cgmath::Matrix4<f32>) -> Self {
        Self {
            data: mat.into()
        }
    }
}

pub struct Camera {
    pub transform: cgmath::Matrix4<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
	pub zoom: f32,

	dragging: bool,
	last_cursorpos: PhysicalPosition<f64>,
	delta: [f32; 2],
	zoom_factor: f32,

	forward: f32,
	backward: f32,
	left: f32,
	right: f32,
	up: f32,
	down: f32,
}

impl Camera {
	const SPEED: f32 = 0.1;
	const ZOOM_SENSITIVITY: f32 = 0.9;
	const TRACKPAD_ZOOM_SENSITIVITY: f32 = 0.5;
	const LOOK_SENSITIVITY: f32 = 0.35;

	pub fn new(transform: cgmath::Matrix4<f32>, aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
		Camera {
			transform: transform,
			aspect: aspect,
			fovy: fovy,
			znear: znear,
			zfar: zfar,
			zoom: 2.0,

			dragging: false,
			last_cursorpos: PhysicalPosition { x: 0.0, y: 0.0 },
			delta: [0.0, 0.0],
			zoom_factor: 1.0,

			forward: 0.0,
			backward: 0.0,
			left: 0.0,
			right: 0.0,
			up: 0.0,
			down: 0.0,
		}
	}

	pub fn view(&self) -> cgmath::Matrix4<f32> {
        return self.transform.invert().unwrap();
    }

    pub fn proj(&self) -> cgmath::Matrix4<f32> {
        return cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
    }

    pub fn view_proj(&self) -> cgmath::Matrix4<f32> {
        return self.proj() * self.view();
    }

	pub fn input(&mut self, event: &WindowEvent) {
		match event {
			WindowEvent::KeyboardInput { input, .. } => {
				if let Some(keycode) = input.virtual_keycode {
					let value = match input.state {
						ElementState::Pressed => 1.0,
						ElementState::Released => 0.0,
					};

					match keycode {
						VirtualKeyCode::W => { self.forward = value },
						VirtualKeyCode::S => { self.backward = value },
						VirtualKeyCode::A => { self.left = value },
						VirtualKeyCode::D => { self.right = value },
						VirtualKeyCode::E => { self.up = value },
						VirtualKeyCode::Q => { self.down = value },
						_ => {}
					}
				}
			},
			WindowEvent::MouseInput { state, button, .. } => {
				match button {
					MouseButton::Left => {
						self.dragging = match state {
							ElementState::Pressed => true,
							ElementState::Released => false,
						};
					},
					_ => {}
				}
			},
			WindowEvent::MouseWheel { delta, .. } => {
				match delta {
					MouseScrollDelta::LineDelta(_, y) => {
						self.zoom_factor *= Self::ZOOM_SENSITIVITY.powf(*y);
					},
					MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
						self.delta = [
							self.delta[0] + *x as f32, 
							self.delta[1] + *y as f32
						];
					},
					_ => {}
				}
			},
			WindowEvent::TouchpadMagnify { delta, .. } => {
				self.zoom_factor *= Self::TRACKPAD_ZOOM_SENSITIVITY.powf(*delta as f32);
			}
			WindowEvent::CursorMoved { position, .. } => {
				if self.dragging {
					self.delta = [
						self.delta[0] + (position.x - self.last_cursorpos.x) as f32,
						self.delta[1] + (position.y - self.last_cursorpos.y) as f32,
					];
				}
				self.last_cursorpos = *position;
			},
			_ => {}
		}
	}

	pub fn update(&mut self) {
		let x = self.right - self.left;
		let y = self.up - self.down;
		let z = self.backward - self.forward;
		
		self.transform = self.transform * Matrix4::from_translation((0.0, 0.0, -self.zoom * self.zoom_factor).into());
		self.zoom *= self.zoom_factor;
		let pos = self.transform.w;
		self.transform.w = Vector4::new(0.0, 0.0, 0.0, 1.0);
		
		let yaw = Matrix4::from_angle_y(Deg { 0: -self.delta[0] * Self::LOOK_SENSITIVITY });
		self.transform = yaw * self.transform;
		
		let tilt = Matrix4::from_angle_x(Deg { 0: -self.delta[1] * Self::LOOK_SENSITIVITY });
		self.transform = self.transform * tilt;
		
		self.transform = self.transform * Matrix4::from_translation((0.0, 0.0, self.zoom * self.zoom_factor).into());
		self.transform = self.transform * Matrix4::from_translation(Vector3::new(x, y, z) * Self::SPEED);
		self.transform.w += Vector4::new(pos.x, pos.y, pos.z, 0.0);
		
		self.zoom_factor = 1.0;
		self.delta = [0.0, 0.0];
	}
}