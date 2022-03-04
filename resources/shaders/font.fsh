#version 140

in vec2 f_texture_uv;

uniform vec4 color = vec4(0.0, 0.0, 0.0, 1.0);
uniform sampler2D tex;

void main() {
    float p = texture2D(tex, f_texture_uv).r;
    vec4 c = vec4(color.rgb, color.a * p);
    if (c.a <= 0.01) {
        discard;
    } else {
        gl_FragColor = c;
    }
}