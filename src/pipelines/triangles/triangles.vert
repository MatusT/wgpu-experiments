#version 460

layout(set = 0, binding = 0, std140) uniform CameraMatrices {
  mat4 projection;
  mat4 view;
  mat4 projection_view;
};

layout(location = 0) in vec4 in_position;

out gl_PerVertex { vec4 gl_Position; };

void main() {
  gl_Position = projection_view * vec4(in_position.x, in_position.y, in_position.z, 1.0); 
}
