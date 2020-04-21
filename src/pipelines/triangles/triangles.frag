#version 460

layout(location = 0) out vec4 out_color;

layout(early_fragment_tests) in;

void main() {
    out_color = vec4(1.0, 1.0, 0.0, 1.0);
}
