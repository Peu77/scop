#version 330 core

in vec3 vertex_color;
in vec2 texture_coordinate;

uniform sampler2D u_texture;
uniform float u_texture_blend;

out vec4 fragment_color;

void main()
{
    vec4 side_color = vec4(vertex_color, 1.0);
    vec4 texture_color = texture(u_texture, texture_coordinate);
    fragment_color = mix(side_color, texture_color, u_texture_blend);
}
