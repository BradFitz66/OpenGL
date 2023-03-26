#version 430 core

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

uniform PointLight lights[2];
uniform DirectionalLight directional_light;
uniform sampler2D tex;

in Vertex{
    vec3 pos;
    vec3 norm;
    vec2 uv;
} i;

layout (location = 0) out vec4 color;

void main(){
    color = vec4(texture(tex,i.uv).xyz,1.0);
}