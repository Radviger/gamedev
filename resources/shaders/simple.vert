#version 330

in vec3 pos;
in vec2 uv;

out vec2 v_uv;

uniform mat4 matrix;

void main() {
    v_uv = uv;
    gl_Position = matrix * vec4(pos, 1.0);
}