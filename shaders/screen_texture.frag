#version 330 core

uniform sampler2D image;
uniform float alpha;

in vec2 v_uv;

out vec4 FragColor;

void main() {
	FragColor = vec4(texture(image, v_uv).xyz, alpha);
}
