# OBJRS
OBJRS is a .obj file renderer written in Rust. OBJRS uses wgpu, which is a Rust library that is based on the WebGPU API. 

## Installation
1. Install Rust
2. Clone the repo
3. In the repo root, run `cargo run -- models/sponza.obj`  
  
On Windows, you can right click on any .obj file, select 'open with', and say to use the compiled .exe. Works best with pre-triangulated or geometry with no n-gons beyond quads.