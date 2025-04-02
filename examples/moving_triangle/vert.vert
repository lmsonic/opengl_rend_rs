#version 330

layout(location = 0) in vec4 position;
uniform float loopDuration;
uniform float time;

smooth out vec4 fragColor;
void main()
{
    float timeScale =  3.14159f * 2.0f / loopDuration;
    float loopTime = mod(time,loopDuration);
    vec4 totalOffset = vec4(cos(loopTime*timeScale)*0.5,sin(loopTime*timeScale)*0.5,0.0,0.0);
    gl_Position = position+totalOffset;
}