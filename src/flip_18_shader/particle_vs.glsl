#version 300 es

layout(std140) uniform _22_8
{
    float _m0;
} _8;

layout(std140) uniform _23_9
{
    vec2 _m0;
} _9;

layout(std140) uniform _24_10
{
    int _m0;
} _10;

layout(location = 0) in vec2 _4;
layout(location = 1) in vec3 _5;
out vec3 _6;
flat out int _7;

void main()
{
    gl_Position = vec4(_4.x * (2.0 / _9._m0.x) + (-1.0), _4.y * (2.0 / _9._m0.y) + (-1.0), 0.0, 1.0);
    gl_PointSize = _8._m0;
    _6 = _5;
    _7 = _10._m0;
}

