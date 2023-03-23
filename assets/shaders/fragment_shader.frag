#version 330 core

uniform vec3 sun_pos;
uniform vec3 uni_color;

out vec3 final_color;

in vec2 tex_coord;
in vec3 position_world;
in vec3 normal_camera;
in vec3 camera_dir;
in vec3 light_dir;

void main() {
    vec3 specular_color = vec3(1.0,1.0,1.0);


    //Should put these as uniforms or make some sort of light struct
    vec3 light_color = vec3(1.0,1.0,1.0);
    float light_intensity = 200.0;
    float light_distance = length(sun_pos - position_world);

    vec3 n = normalize(normal_camera);
    vec3 l = normalize(light_dir);

    float cos_theta = clamp(dot(n,l),0.01,1);

    vec3 E = normalize(camera_dir);
    vec3 R = reflect(-l,n);

    float cos_alpha = clamp(dot(E,R),0.01,1);

    final_color = 
    uni_color * light_color * light_intensity * cos_theta / (light_distance * light_distance) +
    specular_color * light_color * light_intensity * pow(cos_alpha, 5) / (light_distance * light_distance);

}