#version 330

layout(location = 0) in vec4 position;
layout(location = 1) in vec4 color;

uniform mat4 modelToWorld;
uniform mat4 worldToCamera;
uniform mat4 cameraToClip;

void main()
{
    gl_Position = cameraToClip * worldToCamera * modelToWorld * position;
}
