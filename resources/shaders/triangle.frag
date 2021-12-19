#version 150

in vec4 v_color;
in vec3 v_pos;
out vec4 color;

uniform float time;

const float PI = 3.14159265358979323846264338327950288;
const float TAU = 6.28318530717958647692528676655900577;

void main() {
    vec2 p = v_pos.xy;
    float angle = atan(p.y, p.x);
    float wt = angle + time;
    float r = (sin(wt) + 1.0) / 2.0;
    float g = (sin(wt + TAU / 3.0) + 1.0) / 2.0;
    float b = (sin(wt + 2.0 * TAU / 3.0) + 1.0) / 2.0;
    color = vec4(r, g, b, 1.0);
}