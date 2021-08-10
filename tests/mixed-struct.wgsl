// Mixed builtins and user-defined inputs.
struct MyInputs {
  [[location(0)]] x: vec4<f32>;
  [[builtin(front_facing)]] y: bool;
  [[location(1)]] z: u32;
};

struct MyOutputs {
  [[builtin(frag_depth)]] x: f32;
  [[location(0)]] y: vec4<f32>;
};

[[stage(fragment)]]
fn fragShader(in1: MyInputs) -> MyOutputs {
   return MyOutputs(1.0, in1.x);
}
