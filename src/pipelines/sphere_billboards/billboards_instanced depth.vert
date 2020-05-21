#version 460

layout(set = 0, binding = 0, std140) uniform CameraMatrices {
  mat4 projection;
  mat4 view;
  mat4 projection_view;
  vec4 camera_position;
};

layout(location = 0) in vec4 position;

layout(set = 0, binding = 1, std430) buffer ModelMatrices {
  mat4 model_matrices[];
};

const vec2 vertices[3] = {
    vec2(-0.43, -0.25),
    vec2(0.43, -0.25),
    vec2(0.0, 0.75),
};

void main(void) {
  const vec4 world_position = model_matrices[gl_InstanceIndex] * vec4(position.xyz, 1.0);
  const float scale = position.w;

  const vec2 vertex = scale * vertices[gl_VertexIndex % 3];  

  const vec3 CameraRight_worldspace = vec3(view[0][0], view[1][0], view[2][0]);
  const vec3 CameraUp_worldspace = vec3(view[0][1], view[1][1], view[2][1]);
  const vec4 position_worldspace = vec4(
      world_position.xyz +
      vertex.x * CameraRight_worldspace +
      vertex.y * CameraUp_worldspace, 1.0);
  
  gl_Position = projection_view * position_worldspace;
}
