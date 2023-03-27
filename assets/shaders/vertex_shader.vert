#version 430 core


layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;
layout(location = 3) in vec3 tangent;
layout(location = 4) in vec3 bitangent;

uniform mat4 M,V,P;


out Vertex{
    vec3 pos;
    vec3 norm;
    vec2 uv;
    mat3 TBN;
} o;

void main() {
    gl_Position = P*V*M*vec4(pos,1);
    vec3 T = normalize(vec3(M * vec4(tangent,   0.0)));
    vec3 B = normalize(vec3(M * vec4(bitangent, 0.0)));
    vec3 N = normalize(vec3(M * vec4(normal,    0.0)));
    o.pos = pos;
    o.norm = normal;
    o.uv = uv;
    o.TBN = (mat3(T,B,N));
}