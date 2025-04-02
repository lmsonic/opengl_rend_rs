#version 330

uniform float fragLoopDuration;
uniform float time;

const vec4 firstColor = vec4(1.0,1.0,1.0,1.0);
const vec4 secondColor = vec4(0.0,1.0,1.0,1.0);

out vec4 outputColor;
void main()
{

   float loopTime = mod(time,fragLoopDuration);
   float currLerp = loopTime/fragLoopDuration;
   outputColor = mix(firstColor,secondColor,currLerp);;
}