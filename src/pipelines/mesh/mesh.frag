#version 460

layout(location = 0) in vec3 ws_position;
layout(location = 1) in vec3 out_normal;

layout(location = 0) out vec4 out_color;

layout(early_fragment_tests) in;

void main() {
    // vec3 clip_position = vec3(0.0, 0.0, -0.1);
    // vec3 clip_normal = vec3(0.0, 0.0, 1.0);
    // if (dot(ws_position.xyz - clip_position, clip_normal) < 0.0) {
    //     discard;
    // }

    // clip_position = vec3(0.0, 0.0, 0.1);
    // clip_normal = vec3(0.0, 0.0, -1.0);
    // if (dot(ws_position.xyz - clip_position, clip_normal) < 0.0) {
    //     discard;
    // }

    out_color = vec4(out_normal * 0.5 + 0.5, 1.0);
}
