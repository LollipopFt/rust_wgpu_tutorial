// vertex shader
//store output of vertex shader
struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>
};
struct VertexOutput {
    // @builtin(position): value to use as clip position
    // (0, 0) is at top left of screen
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec3<f32>
};

// valid entry point for vertex shader
@vertex
// in_vertex_index calls its value from @builtin(vertex_index)
fn vs_main(model: VertexInput) -> VertexOutput {
    // declare value `out`
    var out: VertexOutput;
    // create values `x` & `y`
    out.color = model.color;
    out.clip_pos = vec4<f32>(model.pos, 1.);
    return out;
}

// fragment shader
@fragment
//@location(0): store returned vec4 in first colour target
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // set colour of current fragment to grey
    return vec4<f32>(in.color, 1.);
}