#version 330

in vec3 pos;
in vec3 normal;
in vec4 color;

out vec4 v_color;
out vec3 v_normal;

uniform mat4 matrix;

void main() {
    v_color = color;
    v_normal = transpose(inverse(mat3(matrix))) * normal;
    gl_Position = matrix * vec4(pos, 1.0);
}