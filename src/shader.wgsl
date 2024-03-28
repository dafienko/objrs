// Vertex shader
struct MatrixUniform {
    data: mat4x4<f32>,
};

@group(0) @binding(0) 
var<uniform> camera: MatrixUniform;


struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexIn,
) -> VertexOutput {
    var out: VertexOutput;
    
    out.clip_position = camera.data * vec4<f32>(model.position.xyz, 1.0);
    out.normal = model.normal;
    
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let light = normalize(vec3<f32>(0.3, -1.0, 0.0));
	let dot = dot(-light, in.normal);
	let color = vec3<f32>(max(dot, 0.01));
    return vec4<f32>(color, 1.0);
}