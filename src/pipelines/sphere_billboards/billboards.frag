#version 460

layout(set = 0, binding = 0, std140) uniform CameraMatrices {
  mat4 projection;
  mat4 view;
  mat4 projection_view;
  vec4 position;
};

// layout(early_fragment_tests) in;

layout(location = 0) in vec2 uv;
layout(location = 1) in vec4 position_clip_space;
layout(location = 2) in flat float scale;

layout(location = 0) out vec4 out_color;

void main(void)
{
	const float lensqr = dot(uv, uv);
	const float dst = scale * 0.5;
	if (length(uv) > dst) {
		discard;
	}	
	
	const float z = sqrt(dst - lensqr);
	const vec3 normal = normalize(vec3(uv.x, uv.y, z)) * 0.5 + 0.5;
	
	// Depth Adjustment
	const float offset = dst - z;
	const vec4 fragPosClip = position_clip_space - projection[2] * offset;
	gl_FragDepth = fragPosClip.z / fragPosClip.w;

	const float diffuse = max(dot(-normal, normalize(position.xyz)), 0.0);
	
	out_color = vec4(diffuse, diffuse, diffuse, 1.0);
	// out_color = vec4(0.3, 0.3, 0.3, 1.0);	
	// out_color = vec4(normalize(position.xyz) * 0.5 + 0.5, 1.0);
}