#version 150

uniform float time;

in vec2 f_texture_uv;
in vec3 f_normal;
in vec4 f_color;

out vec4 out_color;

void main()
{
    vec2 uv = f_texture_uv.xy;
    vec4 texture_color = vec4(9/255.0, 29/255.0, 1.0, 1.0);

    vec4 k = vec4(time)*0.4;
	k.xy = uv * 7.0;
    float val1 = length(0.5-fract(k.xyw*=mat3(vec3(-2.0,-1.0,0.0), vec3(3.0,-1.0,1.0), vec3(1.0,-1.0,-1.0))*0.5));
    float val2 = length(0.5-fract(k.xyw*=mat3(vec3(-2.0,-1.0,0.0), vec3(3.0,-1.0,1.0), vec3(1.0,-1.0,-1.0))*0.2));
    float val3 = length(0.5-fract(k.xyw*=mat3(vec3(-2.0,-1.0,0.0), vec3(3.0,-1.0,1.0), vec3(1.0,-1.0,-1.0))*0.5));
    vec4 color = vec4 ( pow(min(min(val1,val2),val3), 7.0) * 3.0)+texture_color;
    out_color = color;
}