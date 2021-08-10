#version 310 es

precision highp float;
precision highp int;

struct MyInputs {
    vec4 x;
    bool y;
    uint z;
};

struct MyOutputs {
    float x;
    vec4 y;
};

layout(location = 0) smooth in vec4 _vs2fs_location0;
layout(location = 1) flat in uint _vs2fs_location1;
layout(location = 0) out vec4 _fs2p_location0;

void main() {
    MyInputs in1_ = MyInputs(_vs2fs_location0, gl_FrontFacing, _vs2fs_location1);
    MyOutputs _tmp_return = MyOutputs(1.0, in1_.x);
    gl_FragDepth = _tmp_return.x;
    _fs2p_location0 = _tmp_return.y;
    return;
}

