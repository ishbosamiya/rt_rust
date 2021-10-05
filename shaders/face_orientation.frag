#version 330 core

in vec4 from_vert_color_face_front;
in vec4 from_vert_color_face_back;

out vec4 fragColor;

void main()
{
  fragColor = gl_FrontFacing ? from_vert_color_face_front : from_vert_color_face_back;
}
