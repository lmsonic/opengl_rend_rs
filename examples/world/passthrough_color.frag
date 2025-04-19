#version 330

uniform vec4 baseColor;
smooth in vec4 fragColor;

out vec4 outputColor;
void main()
{
    outputColor = fragColor;
}
