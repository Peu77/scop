#version 330 core

layout (location = 0) in vec3 a_position;
layout (location = 1) in vec3 a_color;
layout (location = 2) in vec2 a_uv;
layout (location = 3) in vec3 a_normal;
layout (location = 4) in vec3 a_tangent;
layout (location = 5) in vec3 a_bitangent;

uniform mat4 u_mvp;
uniform mat4 u_model;

out vec3 vertex_color;
out vec2 texture_coordinate;
out mat3 tangent_basis;

void main()
{
    gl_Position = u_mvp * vec4(a_position, 1.0);
    vertex_color = a_color;
    texture_coordinate = a_uv;
    mat3 normal_matrix = mat3(u_model);
    tangent_basis = mat3(
        normalize(normal_matrix * a_tangent),
        normalize(normal_matrix * a_bitangent),
        normalize(normal_matrix * a_normal)
    );
}
