#include "defines.glsl"
#include "structs.glsl"

void global_0(float time, out Global g) {
  g.transformation.scale = cos(time / 25.0) * 5.0 + 6.0;
  g.transformation.rotation = time / 1024.0;
}
void get_global(float time, out Global g) {
  global_0(time, g);
}

void boxes_pattern_0(float step, out Parameters params) {
  params.transformation.scale = 1.0;
  params.transformation.rotation = 0.0;
  params.color_index = COLOR_INDEX_3;
}
void boxes_pattern_1(float step, out Parameters params) {
  params.transformation.scale = mix(0.0, 1.0, float(bool(int(step) & 4)));
  params.transformation.rotation = step / 8.0;
  params.color_index = COLOR_INDEX_2;
}
void boxes_pattern_2(float step, out Parameters params) {
  params.transformation.scale = mix(1.0, 0.0, float(bool(int(step) & 4)));
  params.transformation.rotation = -step / 8.0;
  params.color_index = COLOR_INDEX_2;
}
void boxes_get_parameters_for_step(float step, ivec2 grid_cell, out Parameters params) {
  bool is_odd = bool((grid_cell.x + grid_cell.y) & 1);
  if (grid_cell == ivec2(0)) {
    boxes_pattern_0(step, params);
  }
  else {
  if (is_odd) {
    boxes_pattern_1(step, params);
  } else {
    boxes_pattern_2(step, params);
  }
  }
}

float boxes_get_grid_time(ivec2 grid_cell, float time) {
  float time_offset = length(vec2(grid_cell))*0.05;
  return time + time_offset;
}

void boxes_is_hit(in Parameters params, vec2 pos, ivec2 grid_cell, out Hit hit) {
  vec2 grid_pos = vec2(grid_cell) + vec2(0.5);
  vec2 delta = grid_pos - pos;
  vec2 rotated = rotate(params.transformation, delta);
  vec2 scaled = rotated;
  float size_h = params.transformation.scale * 0.5;
  vec2 check = abs(rotated);
  hit.opacity_color[params.color_index] = float(
   check.x < size_h && check.y < size_h);
}

void boxes_is_hit_multiple(float time, vec2 pos, out Hit hit) {
  ivec2 center_grid = ivec2(floor(pos + vec2(0.5)));
  for (int deltax = -1 ; deltax <=1 ; deltax++) {
  for (int deltay = -1 ; deltay <=1 ; deltay++) {
    ivec2 grid_cell = center_grid + ivec2(deltax, deltay);
    float grid_time = boxes_get_grid_time(grid_cell, time);
    float step = floor(grid_time);
    float factor = fract(grid_time);
    Parameters p1;
    boxes_get_parameters_for_step(step, grid_cell, p1);
    Parameters p2;
    boxes_get_parameters_for_step(step+1.0, grid_cell, p2);
    Parameters p;
    interpolate_parameters(p1, p2, factor, p);
    Hit h;
    vec2 delta_pos = vec2(float(deltax), float(deltay));
    boxes_is_hit(p, pos, grid_cell, h);
    for (int i = 0 ; i < 4; i ++) {
      hit.opacity_color[i] += h.opacity_color[i];
    }
  }

  }
}

void background_layer(in float time, out vec4 out_color) {
    out_color = vec4(COLOR_1, 1.0);
}
void box_layer(in float time, in vec2 uv, out vec4 out_color) {
  Hit h;
  boxes_is_hit_multiple(time, uv, h);
  
  float opacity = h.opacity_color[0] + h.opacity_color[1] + h.opacity_color[2] + h.opacity_color[3];
  float background_opacity = max(1.0 - opacity, 0.0);
  float full_opacity = opacity + background_opacity;
  vec3 color = (h.opacity_color[0] * COLOR_0 + h.opacity_color[1] * COLOR_1 + h.opacity_color[2] * COLOR_2 + h.opacity_color[3] * COLOR_3) / full_opacity;
  out_color = vec4(color, 1.0 - background_opacity);
}

void merge_layers(inout vec4 out_color, in vec4 layer_color) {
  out_color.rgb = mix(out_color.rgb, layer_color.rgb, layer_color.a);

}

void mainImage(out vec4 fragColor, in vec2 inPosition) {
  float time = mod(iTime * BEATS_PER_SECOND, MAX_BEATS);
  
  Global g;
  get_global(time, g);
  
  vec2 uv = transform(g.transformation, inPosition / iResolution.xx - 0.5);
  vec4 box_color, background_color, final_color;
  
  background_layer(time, background_color);
  box_layer(time, uv, box_color);

  final_color = background_color;
  merge_layers(final_color, box_color);
  fragColor = final_color;
}
