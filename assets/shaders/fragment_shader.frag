//This was used as a reference: https://github.com/adrianderstroff/pbr/tree/master/assets/shaders/pbr/simple

#version 430 core

#define DirectionalPointLight 0x00000001
#define PointPointLight       0x00000002

#define M_PI 3.1415926535897932384626433832795


///////////////////////////////////////////////////////////////////////////////////////////
// Uniforms                                                                              //                       
///////////////////////////////////////////////////////////////////////////////////////////
uniform vec3 camera_pos;
uniform vec3 albedo;
uniform float roughness;
uniform float metallic;

//Texture maps
uniform sampler2D diffuse_map;
uniform sampler2D roughness_map;
uniform sampler2D normal_map;
uniform sampler2D metallic_map;


in Vertex{
    vec3 pos;
    vec3 norm;
    vec2 uv;
    mat3 TBN;
} i;

struct pbr_material {
    vec3  albedo;
    float metallic;
    float roughness;
    vec3  f0;
    float a;
    float k;
};

struct micro_surface {
    vec3 n;
    vec3 l;
    vec3 v;
    vec3 h;
};



pbr_material make_pbr_material() {
    pbr_material mat;

    mat.albedo=albedo;
    mat.metallic=metallic*texture(metallic_map,i.uv*4).r;
    mat.roughness=roughness*texture(roughness_map,i.uv*4).r;
    mat.f0=mix(vec3(0.04),mat.albedo,mat.metallic);
    mat.a=mat.roughness*mat.roughness;
    mat.k=((mat.roughness+1) * (mat.roughness+1))/8;

    return mat;
}

micro_surface make_micro_surface(pbr_material mat, vec3 pos, vec3 normal) {
    micro_surface ms;

    ms.n=normalize(normal);
    ms.v=normalize(camera_pos-pos);
    ms.l=normalize(vec3(0.0,1.0,1.0));//Hardcoded light direction for now
    ms.h=normalize(ms.l+ms.v);

    return ms;
}

vec3 fresnel_schlick(pbr_material mat, micro_surface ms) {
    float vdotn = max(dot(ms.v, ms.n), 0.0);
    return mat.f0 + (1.0 - mat.f0) * pow(1.0 - vdotn, 5.0);
}

float normal_distribution_ggx(micro_surface ms, float a) {
    float a2 = a * a;
    float NdotH = max(dot(ms.n, ms.h), 0.0);
    float NdotH2 = NdotH * NdotH;

    float nom = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = M_PI * denom * denom;

    return nom / denom;
}

float geometry_smith(micro_surface ms, float a, float k) {
    float NdotV = max(dot(ms.n, ms.v), 0.0);
    float NdotL = max(dot(ms.n, ms.l), 0.0);
    float ggx1 = NdotL/(NdotL * (1.0 - k) + k);
    float ggx2 = NdotV/(NdotV * (1.0 - k) + k);

    return (ggx1 * ggx2);
}

vec3 specular(pbr_material mat, micro_surface ms){
    vec3 v = ms.v;
    vec3 n = ms.n;
    vec3 l = ms.l;
    vec3 h = ms.h;

    float d = normal_distribution_ggx(ms, mat.a);
    float g = geometry_smith(ms, mat.a, mat.k);
    vec3 f = fresnel_schlick(mat, ms);

    float ndotl = max(dot(n,l),0.0);
    float ndotv = max(dot(n,v),0.0);
    float denom = max(4.0*ndotl*ndotv,0.0001);

    return (f * d * g) / denom;
}



vec3 brdf(pbr_material mat, micro_surface ms) {
    vec3 Ks = fresnel_schlick(mat, ms);
    vec3 kD = (vec3(1.0)-Ks) * (1.0-mat.metallic);

    vec3 diffuse_color = mix(mat.albedo / M_PI,vec3(0),mat.metallic);
    vec3 specular_color = specular(mat, ms);

    return M_PI * kD * diffuse_color + mat.metallic * specular_color;
}



out vec4 o_color;

void main(){
    vec3 normal = normalize(texture(normal_map,i.uv*4).rgb*2.0-1.0);
    normal = normalize(i.TBN*normal);
    //normal = i.norm;
    pbr_material mat = make_pbr_material();
    micro_surface ms = make_micro_surface(mat,i.pos,normal);

    float NdotL = max(dot(ms.n,ms.l),0.0);
    vec3 Lo = M_PI * brdf(mat,ms) * NdotL * vec3(1.0,1.0,1.0);
    vec3 ambient = vec3(0.1)*mat.albedo;
    vec3 color_HDR = ambient + Lo;
    vec3 final = color_HDR * texture(diffuse_map,i.uv*4).rgb;
    o_color = vec4(final,1.0);
}