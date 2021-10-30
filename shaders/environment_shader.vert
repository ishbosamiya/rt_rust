#version 330 core

in vec3 in_pos;

out vec4 frag_near;
out vec4 frag_far;

uniform mat4 view;
uniform mat4 projection;

void main()
{
	mat4 mvp = projection * view;
  mat4 inverse_mvp = inverse(mvp);
  gl_Position = vec4(in_pos, 1.0);

	vec2 pos = gl_Position.xy / gl_Position.w;
  frag_near = inverse_mvp * vec4(pos, -1.0, 1.0);
  frag_far = inverse_mvp * vec4(pos, 1.0, 1.0);
}
