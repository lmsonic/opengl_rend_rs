#version 330

layout(location = 0) in vec4 position;
layout(location = 1) in vec4 color;

layout(std140) uniform GlobalMatrices {
    mat4 cameraToClip;
    mat4 worldToCamera;
};

uniform mat4 modelToWorld;

smooth out vec4 fragColor;
void main()
{
    vec4 worldPos = modelToWorld * position;
    vec4 cameraPos = worldToCamera * worldPos;
    gl_Position = cameraToClip * cameraPos;
    fragColor = color;
}
