#version 100
precision highp float;

// Distance along is passed to U
varying vec2 uv;

uniform vec4 _Time;

const float dimmest = 0.0;
const float speed = 3.0;

void main() {
    float brightness = pow(cos(3.14159 * (uv.x - mod(_Time.x * speed, 1.0))), 6.0) * (1.0 - dimmest) + dimmest;

    gl_FragColor = vec4(1.0, 1.0, 1.0, brightness);
}
