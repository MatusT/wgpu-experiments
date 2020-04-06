#version 460

layout(set = 0, binding = 0, std140) uniform CameraMatrices {
  mat4 projection;
  mat4 view;
  mat4 projection_view;
};

layout(set = 0, binding = 1, std430) buffer InstancedPositions {
  vec4 positions[];
};

layout(location = 0) out VSOUTPUT
{
  vec2 uv;
	vec3 viewPos;
	vec4 clipPos;
} OUTPUT;

const float triSide = 3.464;
const float triHalfSide = triSide / 2.0;

void main(void)
{
	vec4 world_position = positions[gl_VertexIndex / 3];
  world_position.w = 1.0;

  vec3 viewPos = (view * world_position).xyz;
	vec3 viewDir = normalize(viewPos);
	viewPos -= viewDir;

	// create plane looking towards viewer
	const vec3 right = vec4(cross(vec3(0.0, 1.0, 0.0), viewDir), 0).xyz;
	const vec3 up = vec4(cross(viewDir, right.xyz), 0).xyz;
	
  if (gl_VertexIndex % 3 == 0) {
    vec3 offset = -right * 1.73205 - up;
    OUTPUT.uv = vec2(-triHalfSide, -1.0);
    OUTPUT.viewPos = vec3(viewPos + offset);
    OUTPUT.clipPos = projection * vec4(viewPos + offset, 1.0);
    gl_Position = OUTPUT.clipPos;
	} else if (gl_VertexIndex % 3 == 1) {
    vec3 offset = right * 1.73205 - up;
    OUTPUT.uv = vec2(triHalfSide, -1.0);
    OUTPUT.viewPos = vec3(viewPos + offset);
    OUTPUT.clipPos = projection * vec4(viewPos + offset, 1.0);
    gl_Position = OUTPUT.clipPos;
	} else {
    vec3 offset = up * 2;
    OUTPUT.uv = vec2(0.0, 2.0);
    OUTPUT.viewPos = vec3(viewPos + offset);
    OUTPUT.clipPos = projection * vec4(viewPos + offset, 1.0);
    gl_Position = OUTPUT.clipPos;
	}
}
