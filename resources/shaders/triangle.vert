#version 150

in vec3 pos;
in vec4 color;
out vec3 v_pos;
out vec4 v_color;

uniform mat4 matrix;
uniform float time;

void main() {
    vec4 p = matrix * vec4(pos, 1.0);
    gl_Position = p;
    v_pos = pos.xyz;
    v_color = color;
}