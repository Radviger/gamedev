#version 330

in vec3 pos;
in vec3 normal;
in vec4 color;
in vec2 uv;

out vec4 v_color;
out vec3 v_normal;
out vec2 v_uv;

uniform mat4 matrix;

void main() {
    v_color = color;
    v_normal = transpose(inverse(mat3(matrix))) * normal;
    v_uv = uv;
    gl_Position = matrix * vec4(pos, 1.0);
}