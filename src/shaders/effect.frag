#version 330 core

in vec3 vertex_color;
in vec2 texture_coordinate;
in mat3 tangent_basis;
in vec3 world_position;
in vec3 world_normal;
in float effect_phase;

uniform sampler2D u_texture;
uniform sampler2D u_normal_map;
uniform float u_texture_blend;

out vec4 fragment_color;

void main()
{
    vec3 base_texture = texture(u_texture, texture_coordinate).rgb;
    vec3 normal_sample = texture(u_normal_map, texture_coordinate).rgb * 2.0 - 1.0;
    vec3 normal = normalize(tangent_basis * normal_sample);

    vec3 light_direction = normalize(vec3(-0.25, 0.85, 0.45));
    float diffuse = max(dot(normal, light_direction), 0.0);

    vec3 view_direction = normalize(vec3(0.0, 0.0, 1.0));
    float rim = pow(1.0 - max(dot(normalize(world_normal), view_direction), 0.0), 2.2);
    float stripes = 0.5 + 0.5 * sin(world_position.y * 24.0 + world_position.x * 8.0 + effect_phase * 80.0);

    vec3 material = mix(vertex_color, base_texture, u_texture_blend);
    vec3 neon = mix(vec3(0.05, 0.8, 1.0), vec3(0.9, 0.15, 1.0), stripes);
    vec3 color = material * (0.18 + diffuse * 0.58) + neon * (rim * 0.95 + stripes * 0.12);

    fragment_color = vec4(color, 1.0);
}
