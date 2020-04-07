#version 460

layout(set = 0, binding = 0, std140) uniform CameraMatrices {
  mat4 projection;
  mat4 view;
  mat4 projection_view;
};

layout(set = 0, binding = 1, std430) buffer Positions { vec3 positions[]; };
layout(set = 0, binding = 2, std430) buffer Sizes { vec3 sizes[]; };
layout(set = 0, binding = 3, std430) buffer Colors { vec3 colors[]; };

const vec3 vertices[8] = {
    vec3( 1.0, 1.0, 1.0),
    vec3( 1.0, 1.0,-1.0),
    vec3( 1.0,-1.0,-1.0),
    vec3(-1.0,-1.0,-1.0),
    vec3(-1.0,-1.0, 1.0),
    vec3(-1.0, 1.0, 1.0),
    vec3(-1.0, 1.0,-1.0),
    vec3( 1.0,-1.0, 1.0)
};

const int[36] indices = {
  0, 2, 1,
  0, 3, 2,

  1,2,6,
  6,5,1,

  4,5,6,
  6,7,4,

  2,3,6,
  6,3,7,

  0,7,3,
  0,4,7,

  0,1,5,
  0,5,4
};

void main(void)
{
  const vec3 center = positions[gl_InstanceIndex];
  const vec3 scale = sizes[gl_InstanceIndex];

  gl_Position = projection_view * vec4((center + vertices[indices[gl_VertexIndex % 24]] * 0.5 * scale), 1.0);
}