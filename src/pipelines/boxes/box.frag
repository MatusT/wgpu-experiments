#version 460

layout(set = 0, binding = 0, std140) uniform CameraMatrices {
  mat4 projection;
  mat4 view;
  mat4 projection_view;
};

layout(location = 0) in vec3 vs_ws_position;
layout(location = 1) in vec3 vs_color;

layout(location = 0) out vec4 out_color;

void main(void)
{	
    // vec3 p0 = vec3(0, 0, 0);
    // vec3 n0 = vec3(1, 0, 0);

    // if (dot(vs_ws_position - p0, n0) < 0.0) {
    //     discard;
    // }

	out_color = vec4(vs_color, 1.0);
}