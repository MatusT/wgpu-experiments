#version 460

layout(set = 0, binding = 0, std140) uniform CameraMatrices {
  mat4 projection;
  mat4 view;
  mat4 projection_view;
};

layout(location = 0) out vec4 out_color;

void main(void)
{	
	out_color = vec4(1.0, 0.0, 0.0, 1.0);
}