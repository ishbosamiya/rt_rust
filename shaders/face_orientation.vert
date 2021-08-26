#version 330 core

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform vec4 color_face_front;
uniform vec4 color_face_back;

in vec3 in_pos;

out vec4 from_vert_color_face_front;
out vec4 from_vert_color_face_back;

void main()
{
	from_vert_color_face_front = color_face_front;
	from_vert_color_face_back = color_face_back;

  gl_Position = projection * view * model * vec4(in_pos, 1.0);
}
