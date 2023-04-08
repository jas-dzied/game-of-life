// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let h: f32 = hsv.x * 6.0f;
    let s: f32 = hsv.y;
    let v: f32 = hsv.z;

    let w: i32 = i32(h);
    let f: f32 = h - f32(w);
    let p: f32 = v * (1.0f - s);
    let q: f32 = v * (1.0f - (s * f));
    let t: f32 = v * (1.0f - (s * (1.0f - f)));

    var r: f32;
    var g: f32;
    var b: f32;

    switch (w) {
        case 0: { r = v; g = t; b = p; }
        case 1: { r = q; g = v; b = p; }
        case 2: { r = p; g = v; b = t; }
        case 3: { r = p; g = q; b = v; }
        case 4: { r = t; g = p; b = v; }
        case 5: { r = v; g = p; b = q; }
        default: {  }
    }

    return vec3<f32>(r, g, b);
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let life = textureSample(t_diffuse, s_diffuse, in.tex_coords).x;
    let new_color = hsv_to_rgb(vec3<f32>(life / 1.1, 1.0, 1.0));
    return vec4<f32>(new_color, 1.0);
}
