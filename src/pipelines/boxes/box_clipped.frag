#version 460

layout(set = 0, binding = 0, std140) uniform CameraMatrices {
  mat4 projection;
  mat4 view;
  mat4 projection_view;
};

layout(set = 0, binding = 1, std140) uniform ClipPlane {
    vec4 position;
    vec4 normal;
} clip;

layout(location = 0) in vec3 vs_ws_position;

void main(void)
{	
    // vec3 p0 = vec3(0, 0, 0);
    // vec3 n0 = vec3(1, 0, 0);

    if (dot(vs_ws_position - clip.position, clip.normal) < 0.0) {
        discard;
    }

	out_color = vec4(0.0, 0.0, 0.0, 1.0);
}