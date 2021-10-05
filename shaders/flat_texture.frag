#version 330 core

uniform sampler2D image;

in vec2 v_uv;

out vec4 FragColor;

void main() {
	FragColor = texture(image, v_uv);
}
