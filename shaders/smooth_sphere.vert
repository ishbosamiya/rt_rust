#version 330 core

uniform mat4 view;
uniform mat4 projection;

in vec3 in_pos;

out mat4 frag_projection;
out mat4 frag_view;
out vec4 frag_near;
out vec4 frag_far;

void main()
{
  mat4 mvp = projection * view;
  mat4 inverse_mvp = inverse(mvp);
  gl_Position = vec4(in_pos, 1.0);

  frag_projection = projection;
  frag_view = view;

  vec2 pos = gl_Position.xy / gl_Position.w;
  frag_near = inverse_mvp * vec4(pos, -1.0, 1.0);
  frag_far = inverse_mvp * vec4(pos, 1.0, 1.0);
}
