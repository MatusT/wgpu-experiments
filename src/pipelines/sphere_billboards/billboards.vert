#version 460

layout(set = 0, binding = 0, std140) uniform CameraMatrices {
  mat4 projection;
  mat4 view;
  mat4 projection_view;
};

layout(set = 0, binding = 1, std430) buffer InstancedPositions {
  vec4 positions[];
};

// layout(location = 0) out VSOUTPUT
// {
//   vec2 uv;
// 	vec3 viewPos;
// 	vec4 clipPos;
// } OUTPUT;

// const float triSide = 3.464;
// const float triHalfSide = triSide / 2.0;

// const vec2 vertices[3] = {
//     vec2(-0.86, -0.5),
//     vec2(0.86, -0.5),
//     vec2(0.0, 0.74),
// };

layout(location = 0) out vec2 uv;
layout(location = 1) out vec4 position_clip_space;

const vec2 vertices[6] = {
    vec2(-0.5, -0.5),
    vec2(0.5, -0.5),
    vec2(-0.5, 0.5),
    vec2(-0.5, 0.5),
    vec2(0.5, -0.5),
    vec2(0.5, 0.5),
};

void main(void) {
  const vec4 world_position = positions[gl_VertexIndex / 6];
  const vec2 vertex = vertices[gl_VertexIndex % 6];

  const vec3 CameraRight_worldspace = vec3(view[0][0], view[1][0], view[2][0]);
  const vec3 CameraUp_worldspace = vec3(view[0][1], view[1][1], view[2][1]);
  const vec4 position_worldspace = vec4(
      world_position.xyz +
      vertex.x * CameraRight_worldspace +
      vertex.y * CameraUp_worldspace, 1.0);

  uv = vertex;
  position_clip_space = projection_view * position_worldspace;    
  gl_Position = position_clip_space;
}
