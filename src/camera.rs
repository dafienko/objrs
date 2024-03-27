use cgmath::SquareMatrix;

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
}

impl Camera {
	pub fn view(&self) -> cgmath::Matrix4<f32> {
        return self.transform.invert().unwrap();
    }

    pub fn proj(&self) -> cgmath::Matrix4<f32> {
        return cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
    }

    pub fn view_proj(&self) -> cgmath::Matrix4<f32> {
        return self.proj() * self.view();
    }
}