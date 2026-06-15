#version 330 core

layout (location = 0) in vec3 a_position;
layout (location = 1) in vec3 a_color;
layout (location = 2) in vec2 a_uv;

uniform mat4 u_mvp;

out vec3 vertex_color;
out vec2 texture_coordinate;

void main()
{
    gl_Position = u_mvp * vec4(a_position, 1.0);
    vertex_color = a_color;
    texture_coordinate = a_uv;
}
