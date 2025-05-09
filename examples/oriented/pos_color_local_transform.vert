#version 330

layout(location = 0) in vec4 position;
layout(location = 1) in vec4 color;

smooth out vec4 fragColor;

uniform mat4 cameraToClip;
uniform mat4 modelToCamera;

void main()
{
    gl_Position = cameraToClip * modelToCamera * position;
    fragColor = color;
}
