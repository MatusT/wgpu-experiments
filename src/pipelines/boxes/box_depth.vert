#version 460

layout(set = 0, binding = 0, std140) uniform CameraMatrices {
  mat4 projection;
  mat4 view;
  mat4 projection_view;
};

layout(set = 0, binding = 1, std430) buffer ModelMatrices {
  mat4 model_matrices[];
};

const vec3 vertices[8] = {
    vec3(-0.5, -0.5,  0.5),
    vec3( 0.5, -0.5,  0.5),
    vec3( 0.5,  0.5,  0.5),
    vec3(-0.5,  0.5,  0.5),
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

layout(location = 0) out flat uint instance;

void main(void)
{
  const vec3 position_ws = vertices[indices[gl_VertexIndex % 36]];
  instance = gl_InstanceIndex;
  gl_Position = projection_view * model_matrices[gl_InstanceIndex] * vec4(position_ws, 1.0);
}