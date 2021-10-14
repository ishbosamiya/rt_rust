// Based on https://asliceofrendering.com/scene%20helper/2020/01/05/InfiniteGrid/

#version 330 core

in vec3 from_vert_near_point;
in vec3 from_vert_far_point;
in mat4 frag_projection;
in mat4 frag_view;

out vec4 FragColor;

vec4 grid(vec3 frag_pos_3d, float scale) {
	vec2 coord = frag_pos_3d.xz * scale; // use the scale variable to set the distance between the lines
	vec2 derivative = fwidth(coord);
	vec2 grid = abs(fract(coord - 0.5) - 0.5) / derivative;
	float line = min(grid.x, grid.y);
	float minimumz = min(derivative.y, 1);
	float minimumx = min(derivative.x, 1);
	vec4 color = vec4(0.2, 0.2, 0.2, 1.0 - min(line, 1.0));
	// z axis
	if(frag_pos_3d.x > -0.1 * minimumx && frag_pos_3d.x < 0.1 * minimumx) {
		color.z = 1.0;
	}
	// x axis
	if(frag_pos_3d.z > -0.1 * minimumz && frag_pos_3d.z < 0.1 * minimumz) {
		color.x = 1.0;
	}
	return color;
}

float compute_depth(vec3 pos) {
	vec4 clip_space_pos = frag_projection * frag_view * vec4(pos.xyz, 1.0);
	float clip_space_depth = (clip_space_pos.z / clip_space_pos.w);

	// clip space depth is not the same as the frag depth, so compute it
	// as below see reference
	// https://github.com/martin-pr/possumwood/wiki/Infinite-ground-plane-using-GLSL-shaders
	// and
	// https://stackoverflow.com/questions/10264949/glsl-gl-fragcoord-z-calculation-and-setting-gl-fragdepth
	float far = gl_DepthRange.far;
	float near = gl_DepthRange.near;

	float depth = (((far-near) * clip_space_depth) + near + far) / 2.0;

	return depth;
}

float compute_linear_depth(vec3 pos, float scene_near, float scene_far) {
	vec4 clip_space_pos = frag_projection * frag_view * vec4(pos.xyz, 1.0);
	float clip_space_depth = (clip_space_pos.z / clip_space_pos.w) * 2.0 - 1.0; // put back between -1 and 1
	float linear_depth = (2.0 * scene_near * scene_far) / (scene_far + scene_near - clip_space_depth * (scene_far - scene_near));
	return linear_depth / scene_far; // normalize
}

void main() {
	float t = -from_vert_near_point.y / (from_vert_far_point.y - from_vert_near_point.y);
	vec3 frag_pos_3d = from_vert_near_point + t * (from_vert_far_point - from_vert_near_point);
	gl_FragDepth = compute_depth(frag_pos_3d);

	float linear_depth = compute_linear_depth(frag_pos_3d, 0.1, 100.0);
	float fading = max(0, (0.5 - linear_depth));

	FragColor = (grid(frag_pos_3d, 10) + grid(frag_pos_3d, 1)) * float(t > 0.0);
	FragColor *= fading;
}
