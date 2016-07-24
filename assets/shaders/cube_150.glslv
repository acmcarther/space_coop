#version 150 core

in vec3 a_Pos;
in vec3 a_Norm;
in vec2 a_TexCoord;

out vec2 v_TexCoord;
out vec3 o_normal;
out vec3 o_toCamera;
out vec3 o_fragVert;

uniform vec3 u_cameraPos;
uniform mat4 u_cameraPV;
uniform mat4 u_objToWorld;
uniform mat4 u_normToWorld;

void main() {
    v_TexCoord = a_TexCoord;

    vec4 worldPosition = u_objToWorld * vec4(a_Pos, 1.0);

    o_normal = normalize(u_normToWorld * vec4(a_Norm, 1.0)).xyz;
    o_toCamera = normalize(u_cameraPos - worldPosition.xyz);
    o_fragVert = a_Pos;

    gl_Position = u_cameraPV * u_objToWorld * vec4(a_Pos, 1.0);
}
