struct Params {
    width: u32,
    height: u32,
}

@group(0)
@binding(0)
var<uniform> params: Params;

@group(0)
@binding(1)
var<storage, read> input_buffer: array<u32>; // this is used as both ininpudt and output for convenience

@group(0)
@binding(2)
var<storage, write> output_buffer: array<u32>; // this is used as both input and output for convenience

@group(0)
@binding(3)
var output_texture: texture_storage_2d<rgba32float, write>;

fn modulus(a: i32, b: i32) -> i32 {
    return ((a % b) + b) % b;
}

fn from_xy(x: u32, y: u32) -> u32 {
    return y * params.width + x;
}

var<private> max_life: u32 = 200;

fn get_at(position: vec3<u32>, x_mod: i32, y_mod: i32) -> u32 {
    let x = u32(modulus(i32(position.x) + x_mod, i32(params.width)));
    let y = u32(modulus(i32(position.y) + y_mod, i32(params.height)));
    let index = from_xy(x, y);
    let value = input_buffer[index];
    if value == u32(max_life) {
      return u32(1);  
    } else {
      return u32(0);
    }
}

@compute
@workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

    let index = from_xy(global_id.x, global_id.y);
    let old_value = input_buffer[index];

    var total: u32;
    total += get_at(global_id, -1, -1);
    total += get_at(global_id, 0, -1);
    total += get_at(global_id, 1, -1);
    total += get_at(global_id, -1, 0);
    total += get_at(global_id, 1, 0);
    total += get_at(global_id, -1, 1);
    total += get_at(global_id, 0, 1);
    total += get_at(global_id, 1, 1);

    var alive_rules = array(false, false, true, true, true, true, false, false, false);
    var dead_rules = array(false, false, false, false, true, true, true, true, true);

    var is_alive: bool;
    if old_value == u32(max_life) {
        is_alive = alive_rules[total];
    } else {
        is_alive = dead_rules[total];
    }

    var new_value: u32;
    if is_alive {
        new_value = u32(max_life);
    } else if old_value > u32(0) {
        new_value = old_value - u32(4);
    }

    output_buffer[index] = new_value;

    if new_value == u32(max_life) {
        textureStore(
          output_texture, 
          vec2<u32>(global_id.x, global_id.y), 
          vec4<f32>(
            0.0, 
            1.0, 
            1.0, 
            1.0
          )
        );
    } else {
        textureStore(
          output_texture, 
          vec2<u32>(global_id.x, global_id.y), 
          vec4<f32>(
            0.0, 
            0.0, 
            f32(new_value)/f32(max_life), 
            1.0
          )
        );
    }
}
