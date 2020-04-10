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
    // front
    vec3(-0.5, -0.5,  0.5),
    vec3( 0.5, -0.5,  0.5),
    vec3( 0.5,  0.5,  0.5),
    vec3(-0.5,  0.5,  0.5),
    // back
    vec3(-0.5, -0.5, -0.5),
    vec3( 0.5, -0.5, -0.5),
    vec3( 0.5,  0.5, -0.5),
    vec3(-0.5,  0.5, -0.5)
};

const int indices[36] = {
		// front
		0, 1, 2,
		2, 3, 0,
		// right
		1, 5, 6,
		6, 2, 1,
		// back
		7, 6, 5,
		5, 4, 7,
		// left
		4, 0, 3,
		3, 7, 4,
		// bottom
		4, 5, 1,
		1, 0, 4,
		// top
		3, 2, 6,
    6, 7, 3
};

layout(location = 0) out vec3 vs_color;

void main(void)
{
  const vec3 center = positions[gl_InstanceIndex].xyz;
  const vec3 scale = sizes[gl_InstanceIndex].xyz;

  vs_color = colors[gl_InstanceIndex].rgb;
  gl_Position = projection_view * vec4((center + vertices[indices[gl_VertexIndex % 36]] * scale), 1.0);
}