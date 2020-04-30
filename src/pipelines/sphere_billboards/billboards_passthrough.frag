#version 460

layout(location = 0) vec4 out_color;

void main(void)
{	
	out_color = vec4(normal, 1.0);
}