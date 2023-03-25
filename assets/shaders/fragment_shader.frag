#version 330 core

#define DirectionalPointLight 0x00000001
#define PointPointLight       0x00000002

#define M_PI 3.1415926535897932384626433832795


struct PointLight {
    vec4 position;
    vec4 color; //Alpha controls intensity
};

struct DirectionalLight {
    vec4 direction;
    vec4 color; //Alpha controls intensity
};



uniform vec3 albedo;
uniform float metallic;
uniform float roughness;
uniform float ao;
uniform vec3 V_P;
uniform PointLight lights[2];
uniform DirectionalLight directional_light;


out vec4 final_color;


in vec2 tex_coord;
in vec3 norm;
in vec3 frag_pos;
in vec3 view_dir;

vec3 CalcDirLight(DirectionalLight light, vec3 normal)
{
    //Calculate directional light from the values in directional light struct
    vec3 light_dir = normalize(light.direction.xyz);
    vec3 halfway_dir = normalize(light_dir + view_dir);
    vec3 diffuse = light.color.rgb * max(dot(normal, light_dir), 0) * albedo;
    vec3 specular = vec3(0.0);
    vec3 ambient = vec3(1.0);
    if (dot(normal, light_dir) > 0.0) {
        specular = light.color.rgb * pow(max(dot(normal, halfway_dir), 0.0), 32.0);
    }
    return diffuse;
}

vec3 CalcPointLight(PointLight light, vec3 normal)
{
    //Calculate point light from the values in point light struct
    vec3 light_dir = normalize(light.position.xyz - frag_pos);
    vec3 halfway_dir = normalize(light_dir + view_dir);
    float distance = length(light.position.xyz - frag_pos);
    float attenuation = 1.0 / (distance * distance);
    vec3 diffuse = light.color.rgb * max(dot(normal, light_dir), 0.10) * albedo;
    vec3 specular = vec3(0.0);
    vec3 ambient = vec3(0.1);
    if (dot(normal, light_dir) > 0.0) {
        specular = light.color.rgb * pow(max(dot(normal, halfway_dir), 0.0), 32.0);
    }
    return (ambient + (diffuse + specular) * attenuation) * light.color.a;
} 

void main() {
    vec3 N = normalize(norm); 
    vec3 V = normalize(V_P - frag_pos);

    //Loop through all lights
    vec3 result = vec3(0.0);
    // for (int i = 0; i < 2; i++) {
    //     result += CalcPointLight(lights[i], N);
    // }
    result += CalcDirLight(directional_light, N);

    final_color = vec4(result, 1.0) ;
}