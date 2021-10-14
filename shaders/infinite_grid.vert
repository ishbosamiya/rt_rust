// Based on https://asliceofrendering.com/scene%20helper/2020/01/05/InfiniteGrid/

#version 330 core

uniform mat4 view;
uniform mat4 projection;

in vec3 in_pos;

out vec3 from_vert_near_point;
out vec3 from_vert_far_point;
out mat4 frag_projection;
out mat4 frag_view;

mat4 view_inv = inverse(view);
mat4 proj_inv = inverse(projection);

vec3 unproject_point(vec3 pos) {
	vec4 unprojected_point =  view_inv * proj_inv * vec4(pos, 1.0);
	return unprojected_point.xyz / unprojected_point.w;
}

void main() {
	from_vert_near_point = unproject_point(vec3(in_pos.x, in_pos.y, 0.0));
	from_vert_far_point = unproject_point(vec3(in_pos.x, in_pos.y, 1.0));
	frag_projection = projection;
	frag_view = view;

	gl_Position = vec4(in_pos, 1.0);
}
