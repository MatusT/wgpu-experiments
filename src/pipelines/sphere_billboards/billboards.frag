#version 460

layout(set = 0, binding = 0, std140) uniform CameraMatrices {
  mat4 projection;
  mat4 view;
  mat4 projection_view;
};

layout(location = 0) in VSOUTPUT
{
	vec2 uv;
	vec3 viewPos;
	vec4 clipPos;
} INPUT;

layout(location = 0) out vec4 out_color;

// conservative depth extension for performance
// layout (depth_greater) out float gl_FragDepth;

void main(void)
{
	const float lensqr = dot(INPUT.uv, INPUT.uv);
	if (lensqr > 1.0) {
		discard;
	}
	
	const float z = sqrt(1.0 - lensqr);
	
	// compute normal
	const vec3 normal = normalize(vec3(INPUT.uv.x, INPUT.uv.y, z)) * 0.5 + 0.5;
	
	vec3 fragViewPos = INPUT.viewPos;
	fragViewPos.z += z;
	
	// ===================== Depth Adjustment =====================	
	const float offset = 1 - z;
	const vec4 fragPosClip = INPUT.clipPos - projection[2] * offset;
	
	const float z_d = fragPosClip.z / fragPosClip.w;
	gl_FragDepth = z_d;
	
	out_color = vec4(normal, 1.0);
}