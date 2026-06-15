#version 330 core

layout (location = 0) in vec3 a_position;
layout (location = 1) in vec3 a_color;
layout (location = 2) in vec2 a_uv;
layout (location = 3) in vec3 a_normal;
layout (location = 4) in vec3 a_tangent;
layout (location = 5) in vec3 a_bitangent;

uniform mat4 u_mvp;
uniform mat4 u_model;
uniform float u_texture_blend;

out vec3 vertex_color;
out vec2 texture_coordinate;
out mat3 tangent_basis;
out vec3 world_position;
out vec3 world_normal;
out float effect_phase;

void main()
{
    float wave = sin(a_position.y * 14.0 + a_position.x * 5.0 + u_texture_blend * 6.28318) * 0.025;
    vec3 displaced = a_position + normalize(a_normal) * wave;
    vec4 world = u_model * vec4(displaced, 1.0);

    gl_Position = u_mvp * vec4(displaced, 1.0);
    vertex_color = a_color;
    texture_coordinate = a_uv + vec2(wave * 1.8, -wave * 1.2);

    mat3 normal_matrix = mat3(u_model);
    tangent_basis = mat3(
        normalize(normal_matrix * a_tangent),
        normalize(normal_matrix * a_bitangent),
        normalize(normal_matrix * a_normal)
    );
    world_position = world.xyz;
    world_normal = normalize(normal_matrix * a_normal);
    effect_phase = wave;
}
