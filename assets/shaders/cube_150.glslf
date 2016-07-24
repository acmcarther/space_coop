#version 150 core

out vec4 o_color;

// TODO
in vec2 v_TexCoord;
in vec3 o_normal;
in vec3 o_toCamera;
in vec3 o_fragVert;

uniform mat4 u_objToWorld;
uniform vec4 u_lightPos;
uniform sampler2D t_Color;
//noperspective varying vec3 dist;

void main() {
    float amb = 0.2;

    vec4 tex = texture(t_Color, v_TexCoord);
    float blend = dot(v_TexCoord-vec2(0.5,0.5), v_TexCoord-vec2(0.5,0.5));
    vec4 tex_color = mix(tex, vec4(0.0,0.0,0.0,0.0), blend*1.0);

    float attenuation;

    vec3 o_toLight;
    if (u_lightPos.w == 0.0) {
      attenuation = 1.0;
      o_toLight = normalize(u_lightPos.xyz);
    } else {
      vec3 surfacePos = vec3(u_objToWorld * vec4(o_fragVert, 1));
      float distance = length(u_lightPos.xyz - surfacePos);
      o_toLight = normalize(u_lightPos.xyz - surfacePos);
      attenuation = 1.0 / (1.0 * pow(distance, 1));
    }
    vec3 L = normalize(o_toLight.xyz);
    vec3 V = normalize(o_toCamera);
    vec3 N = normalize(o_normal);

    vec4 Iamb = amb *vec4(0.2, 0.2, 0.2, 1.0);
    float diffuseTerm = clamp(dot(N.xyz, L.xyz), 0.0, 1.0);
    vec4 Idiff = (vec4(0.7, 0.7, 0.7, 1.0) * diffuseTerm) * attenuation;

    vec3 H = normalize(L + V);
    float shouldRenderSpec = sign(clamp(dot(N.xyz, L.xyz), 0.0, 1.0));
    vec4 Ispec = (shouldRenderSpec * vec4(0.5, 0.5, 0.5, 1.0) * pow(dot(N,H), 64)) * attenuation;

    o_color = Iamb + Idiff + Ispec;
}
