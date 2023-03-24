#version 330 core

uniform vec3 light_pos;
uniform vec3 uni_color;

out vec4 final_color;


in vec2 tex_coord;
in vec3 norm;
in vec3 frag_pos;


void main() {
    vec3 n = normalize(norm);
    vec3 light_dir = normalize(light_pos - frag_pos);

    float diff = max(dot(n, light_dir), 0.0);
    vec3 diffuse = diff * vec3(1.0, 1.0, 1.0);
    vec3 result = diffuse * uni_color;

    final_color = vec4(diff,diff,diff,1.0);
}