#version 330 core
layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

uniform mat4 MVP;
uniform mat4 V;
uniform mat4 M;
uniform vec3 V_P; //view position

out vec2 tex_coord;
out vec3 norm;
out vec3 frag_pos;
out vec3 view_pos;

void main() {


    tex_coord = uv;
    norm = normal;
    frag_pos = vec3(M * vec4(pos,1.0));
    view_pos = vec3(V * vec4(pos,1.0));


    gl_Position = MVP * vec4(pos,1.0);
}