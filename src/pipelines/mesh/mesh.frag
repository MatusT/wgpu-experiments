#version 460

layout(location = 0) in vec3 out_normal;

layout(location = 0) out vec4 out_color;

layout(early_fragment_tests) in;

void main() {
    out_color = vec4(normalize(out_normal) * 0.5 + 0.5, 1.0);
}
