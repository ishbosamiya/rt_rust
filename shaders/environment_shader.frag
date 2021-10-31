#version 330 core

#define PI 3.14159265359

uniform sampler2D environment_map;
uniform mat4 model;
uniform float strength;

in vec4 frag_near;
in vec4 frag_far;

out vec4 fragColor;

// reference:
// https://encreative.blogspot.com/2019/05/computing-ray-origin-and-direction-from.html
vec3 get_ray_origin()
{
  return frag_near.xyz / frag_near.w;
}

// reference:
// https://encreative.blogspot.com/2019/05/computing-ray-origin-and-direction-from.html
vec3 get_ray_direction(in vec3 ray_origin)
{
  return normalize((frag_far.xyz / frag_far.w) - ray_origin);
}

vec2 direction_to_equirectangular_range(in vec3 dir, in vec4 range) {
    float u = (atan(-dir[2], dir[0]) - range[1]) / range[0];
    float v = (acos(-dir[1] / length(dir)) - range[3]) / range[2];

    return vec2(u, v);
}

vec2 direction_to_equirectangular(in vec3 dir) {
    return direction_to_equirectangular_range(dir, vec4(-2.0 * PI, PI, -PI, PI));
}

void main()
{
	vec3 ray_origin = get_ray_origin();
  vec3 ray_direction = (model * vec4(get_ray_direction(ray_origin), 1.0)).xyz;

	vec2 uv = direction_to_equirectangular(ray_direction);

	fragColor = vec4(strength * texture(environment_map, uv).xyz, 1.0);
}
