#version 330 core

in vec3 in_pos;
in vec3 in_normal;

out vec3 Normal;
out vec3 FragPos;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main()
{
  FragPos = vec3(model * vec4(in_pos, 1.0));
  Normal = mat3(transpose(inverse(model))) * in_normal;

  gl_Position = projection * view * vec4(FragPos, 1.0);
}
