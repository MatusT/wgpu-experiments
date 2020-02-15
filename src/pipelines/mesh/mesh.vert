#version 460

layout(binding = 0, std140) uniform CameraMatrices {
  mat4 projection;
  mat4 view;
};

layout(location = 0) in vec3 in_position;

out gl_PerVertex { vec4 gl_Position; };

void main() { gl_Position = projection * view * vec4(in_position, 1.0); }
