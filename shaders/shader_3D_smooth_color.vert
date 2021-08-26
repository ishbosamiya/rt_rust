#version 330 core

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

in vec3 in_pos;
in vec4 in_color;

out vec4 finalColor;

void main()
{
  gl_Position = projection * view * model * vec4(in_pos, 1.0);
  finalColor = in_color;
}
