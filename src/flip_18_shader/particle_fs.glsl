#version 300 es
precision mediump float;
precision highp int;

in highp vec3 _13;
flat in int _14;
layout(location = 0) out highp vec4 _15;

void main()
{
    bool _99;
    if (_14 == 1)
    {
        highp float _88 = 0.5 - gl_PointCoord.x;
        highp float _90 = 0.5 - gl_PointCoord.y;
        _99 = ((_88 * _88 + (_90 * _90)) > 0.25) ? false : true;
    }
    else
    {
        _99 = true;
    }
    if (_99)
    {
        _15 = vec4(_13, 1.0);
    }
    else
    {
    }
}

