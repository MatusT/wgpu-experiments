#version 460

layout(set = 0, binding = 2, std430) buffer Fragments {
  float fragments[];
};

layout(early_fragment_tests) in;
layout(location = 0) in flat uint instance;

void main(void)
{
    fragments[instance] = 1;
}