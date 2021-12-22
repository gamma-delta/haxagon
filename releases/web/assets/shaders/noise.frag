#version 100
precision highp float;

varying vec2 uv;
varying vec4 color;
uniform vec4 _Time;

float rand(vec2 co) {
    return fract(sin(dot(mod(co.xy, 1000.0), vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    vec3 random = vec3(rand(color.rg * uv * _Time.x * 0.5), rand(uv.yx / color.br * _Time.x * 0.9), rand(color.gb * uv.xy * _Time.x * 10.0));
    gl_FragColor = vec4(random * color.rgb, color.a);
}
