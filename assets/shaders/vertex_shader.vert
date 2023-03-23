#version 330 core
layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 normal;

uniform mat4 MVP;
uniform mat4 V;
uniform mat4 M;
uniform vec3 light_pos;

out vec2 tex_coord;
out vec3 position_world;
out vec3 normal_camera;
out vec3 camera_dir;
out vec3 light_dir;

void main() {
    position_world = (M * vec4(pos,1.0)).xyz;
    normal_camera = (V * M * vec4(normal,0.0)).xyz;
    vec3 pos_camera = (V * M * vec4(pos,1.0)).xyz;
    camera_dir = vec3(0,0,0) - pos_camera;

    vec3 lightpos_camera = (V * vec4(light_pos,1.0)).xyz;
    light_dir = lightpos_camera + pos_camera;

    gl_Position = MVP * vec4(pos,1.0);

    tex_coord = uv;
}