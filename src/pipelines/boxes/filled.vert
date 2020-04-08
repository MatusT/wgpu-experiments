#version 460

layout(set = 0, binding = 0, std140) uniform CameraMatrices {
  mat4 projection;
  mat4 view;
  mat4 projection_view;
};

layout(set = 0, binding = 1, std430) buffer Positions { vec4 positions[]; };
layout(set = 0, binding = 2, std430) buffer Sizes { vec4 sizes[]; };
layout(set = 0, binding = 3, std430) buffer Colors { vec4 colors[]; };

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

layout(location = 0) out vec3 vs_color;

void main(void)
{
  const vec3 center = positions[gl_InstanceIndex].xyz;
  const vec3 scale = sizes[gl_InstanceIndex].xyz;

  vs_color = colors[gl_InstanceIndex].rgb;
  gl_Position = projection_view * vec4((center + vertices[indices[gl_VertexIndex % 24]] * 0.5 * scale), 1.0);
}