#version 330

layout(location = 0) in vec4 position;
layout(location = 1) in vec4 color;

uniform mat4 modelToCamera;
uniform mat4 cameraToClip;

smooth out vec4 fragColor;
void main()
{
    gl_Position = cameraToClip * modelToCamera * position;
    fragColor = color;
}
