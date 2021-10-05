#version 330 core

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

in vec3 in_pos;
in vec2 in_uv;

out vec2 v_uv;

void main()
{
  gl_Position = projection * view * model * vec4(in_pos, 1.0);
	v_uv = in_uv;
}
