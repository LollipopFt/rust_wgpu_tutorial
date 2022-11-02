// vertex shader
//store output of vertex shader
struct VertexOutput {
    // @builtin(position): value to use as clip position
    // (0, 0) is at top left of screen
    @builtin(position) clip_position: vec4<f32>
};

// valid entry point for vertex shader
@vertex
// in_vertex_index calls its value from @builtin(vertex_index)
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    // declare value `out`
    var out: VertexOutput;
    // create values `x` & `y`
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y, 0., 1.);
    return out;
}

// fragment shader
@fragment
//@location(0): store returned vec4 in first colour target
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // set colour of current fragment to grey
    return vec4<f32>(0.5, 0.5, 0.5, 1.);
}