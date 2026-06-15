#version 330 core

in vec3 vertex_color;
in vec2 texture_coordinate;
in mat3 tangent_basis;

uniform sampler2D u_texture;
uniform sampler2D u_normal_map;
uniform float u_texture_blend;

out vec4 fragment_color;

void main()
{
    vec4 side_color = vec4(vertex_color, 1.0);
    vec4 texture_color = texture(u_texture, texture_coordinate);
    vec3 normal_sample = texture(u_normal_map, texture_coordinate).rgb * 2.0 - 1.0;
    vec3 normal = normalize(tangent_basis * normal_sample);
    vec3 light_direction = normalize(vec3(0.35, 0.75, 0.55));
    float diffuse = max(dot(normal, light_direction), 0.0);
    float lighting = 0.24 + diffuse * 0.76;
    fragment_color = mix(side_color, texture_color, u_texture_blend) * vec4(vec3(lighting), 1.0);
}
