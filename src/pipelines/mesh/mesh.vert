#version 460

layout(set = 0, binding = 0, std140) uniform CameraMatrices {
  mat4 projection;
  mat4 view;
  mat4 projection_view;
};

layout(set = 0, binding = 1, std430) buffer InstancedPositions {
  vec4 position[];
};

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;

layout(location = 0) out vec3 out_normal;

out gl_PerVertex { vec4 gl_Position; };

void main() { 
  out_normal = in_normal;
  gl_Position = projection_view * (vec4(in_position, 1.0) + position[gl_InstanceIndex]); 
}
