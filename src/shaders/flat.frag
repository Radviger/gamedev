#version 330

in vec3 v_pos;
in vec4 v_color;
in vec3 v_normal;

out vec4 color;

const vec3 light_pos = vec3(0.7, 0.7, 0.7);

void main() {
    float brightness = dot(normalize(v_normal), normalize(light_pos));
    vec3 normal_color = v_color.rgb;
    vec3 dark_color = normal_color * 0.6;
    color = vec4(mix(dark_color, normal_color, brightness), 1.0);
}