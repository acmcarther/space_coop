#version 150 core

out vec4 o_color;

// TODO
in vec2 v_TexCoord;
in vec3 o_normal;
in vec3 o_toLight;
in vec3 o_toCamera;
uniform sampler2D t_Color;
//noperspective varying vec3 dist;

void main() {
    float amb = 0.2;

    vec4 tex = texture(t_Color, v_TexCoord);
    float blend = dot(v_TexCoord-vec2(0.5,0.5), v_TexCoord-vec2(0.5,0.5));
    vec4 tex_color = mix(tex, vec4(0.0,0.0,0.0,0.0), blend*1.0);

    float distance = length(o_toLight);
    distance = distance;
    vec3 L = normalize(o_toLight);
    vec3 V = normalize(o_toCamera);
    vec3 N = normalize(o_normal);

    vec4 Iamb = amb *vec4(0.6, 0.3, 0.0, 1.0);
    float diffuseTerm = clamp(dot(N.xyz, L.xyz), 0.0, 1.0);
    vec4 Idiff = (vec4(1, 0.5, 0.0, 1.0) * diffuseTerm) / distance;

    vec3 H = normalize(L + V);
    float shouldRenderSpec = sign(clamp(dot(N.xyz, L.xyz), 0.0, 1.0));
    vec4 Ispec = (shouldRenderSpec * vec4(0.0, 1.0, 0.0, 1.0) * pow(dot(N,H), 64)) / distance;

    o_color = Iamb + Idiff + Ispec;
}
