#version 330 core

uniform vec3 light_pos;
uniform vec3 uni_color;
uniform vec3 light_color;

out vec4 final_color;


in vec2 tex_coord;
in vec3 norm;
in vec3 frag_pos;
in vec3 view_dir;


void main() {
    float ambient_strength = 0;
    vec3 ambient = ambient_strength * light_color;
    vec3 n = normalize(norm);
    vec3 light_dir = normalize(light_pos - frag_pos);

    float diff = max(dot(n, light_dir), 0.0);
    vec3 diffuse = diff * light_color;
    vec3 result = (ambient+diffuse) * uni_color;

    final_color = vec4(result, 1.0);
}