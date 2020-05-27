#version 460

#extension GL_NV_mesh_shader: require

layout(location = 0) out vec4 color;

void main(void)
{
	color = vec4(0.5, 0.5, 0.5, 1.0);
}