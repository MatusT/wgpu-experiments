#version 460

layout(set = 0, binding = 0, std140) uniform CameraMatrices {
  mat4 projection;
  mat4 view;
  mat4 projection_view;
};

layout(location = 0) in vec2 uv;
layout(location = 1) in vec4 position_clip_space;

layout(location = 0) out vec4 out_color;

void main(void)
{
	const float lensqr = dot(uv, uv);
	if (lensqr > 0.25) {
		discard;
	}	
	
	const float z = sqrt(1.0 - lensqr);
	const vec3 normal = normalize(vec3(uv.x, uv.y, z)) * 0.5 + 0.5;
	
	// // ===================== Depth Adjustment =====================	
	const float offset = 1 - z;
	const vec4 fragPosClip = position_clip_space - projection[2] * offset;
	gl_FragDepth = fragPosClip.z / fragPosClip.w;
	
	out_color = vec4(normal, 1.0);
}