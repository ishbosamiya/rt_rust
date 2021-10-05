#version 330 core

uniform vec3 sphere_center;
uniform float sphere_radius;
uniform vec4 outside_color;
uniform vec4 inside_color;

in vec4 frag_near;
in vec4 frag_far;
in mat4 frag_projection;
in mat4 frag_view;

out vec4 fragColor;

// reference:
// https://iquilezles.org/www/articles/spherefunctions/spherefunctions.htm
float intersect_sphere(in vec3 sphere_center,
                       in float sphere_radius,
                       in vec3 ray_origin,
                       in vec3 ray_direction,
                       out bool inside_sphere)
{
  vec3 oc = ray_origin - sphere_center;
  float b = dot(ray_direction, oc);
  float c = dot(oc, oc) - sphere_radius * sphere_radius;
  float t = b * b - c;
  if (t > 0.0) {
    float val = -b - sqrt(t);
    if (val < 0.0) {
      val = -b + sqrt(t);
      inside_sphere = true;
    }
    else {
      inside_sphere = false;
    }
    t = val;
  }
  return t;
}

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

void main()
{
  vec3 ray_origin = get_ray_origin();
  vec3 ray_direction = get_ray_direction(ray_origin);

  bool inside_sphere;
  float t = intersect_sphere(sphere_center, sphere_radius, ray_origin, ray_direction, inside_sphere);

  if (t > 0.0) {
    if (inside_sphere) {
      fragColor = inside_color;
    }
    else {
      fragColor = outside_color;
    }
    gl_FragDepth = compute_depth(ray_origin + t * ray_direction);
  }
  else {
    discard;
  }
}
