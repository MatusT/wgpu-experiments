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

const float triSide = 3.464;
const float triHalfSide = triSide / 2.0;

void main(void)
{
	// const vec4 world_position = positions[gl_VertexIndex / 3];

  // vec3 viewPos = (view * world_position).xyz;
	// vec3 viewDir = normalize(viewPos);
	// viewPos -= viewDir;

	// // create plane looking towards viewer
	// const vec3 right = cross(vec3(0.0, 1.0, 0.0), viewDir);
	// const vec3 up = cross(viewDir, right.xyz);
	
  // vec3 offset;
  // if (gl_VertexIndex % 3 == 0) {
  //   offset = -right * 1.73205 - up;
	// } else if (gl_VertexIndex % 3 == 1) {
  //   offset = right * 1.73205 - up;
	// } else {
  //   offset = up * 2;
	// }

  // gl_Position = projection * vec4(viewPos + offset, 1.0);
}
