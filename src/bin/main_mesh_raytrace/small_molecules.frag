#version 460

#extension GL_NV_mesh_shader: require

layout(set = 0, binding = 0, std140) uniform CameraMatrices {
    mat4 projection;
    mat4 view;
	mat4 projection_view;
	vec4 camera_position;
} camera;

layout (set = 0, binding = 1, std140) uniform MoleculeBuffer {
	vec4 positions[64];
	vec4 aabb_scale;
	uint count;
} molecule;

layout(location = 0) in vec3 view_position;
layout(location = 1) in perprimitiveNV vec3 positions[64];

layout(location = 0) out vec4 color;

bool raySphereIntersect(vec3 rd, vec3 s0, float sr) {
    // - r0: ray origin
    // - rd: normalized ray direction
    // - s0: sphere center
    // - sr: sphere radius
    // - Returns distance from r0 to first intersecion with sphere,
    //   or -1.0 if no intersection.
    float a = dot(rd, rd);
    vec3 s0_r0 = -s0;
    float b = 2.0 * dot(rd, s0_r0);
    float c = dot(s0_r0, s0_r0) - (sr * sr);

    if (b*b - 4.0*a*c < 0.0) {
        return false;
    }

    return true;
}

// bool slabs(vec3 p0, vec3 p1, vec3 invRaydir) {
// 	vec3 t0 = p0 * invRaydir;
// 	vec3 t1 = p1 * invRaydir;
// 	vec3 tmin = min(t0, t1); 
// 	vec3 tmax = max(t0, t1);
// 	return max(tmin) <= min(tmax);
// }

void main(void)
{
	bool d = true;
	for(uint i = 0; i < 4; i++) {
		if (raySphereIntersect(normalize(view_position), positions[i].xyz, 0.5)) {
			d = false;
			break;
		}
	}
	if (d) {
    	discard;
	} else {
		color = vec4(0.5, 0.5, 0.5, 1.0);
	}
}