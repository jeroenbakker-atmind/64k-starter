#include "defines.glsl"

struct Transformation {
  float rotation;
  float scale;
  vec2 translation;
};
struct Global {
  Transformation transformation;
};
struct Parameters {
  Transformation transformation;
  int color_index;
};
struct Hit {
  float opacity_color[4];
};

float apply_curve(float factor) {
  return clamp(pow(factor*1.5, 1.5), 0.0, 1.0);
}

float interpolate_float(float f1, float f2, float factor) {
  return mix(f1, f2, factor);
}

void interpolate_parameters(in Parameters p1, in Parameters p2, float factor, out Parameters p) {
  float factor2 = apply_curve(factor);
  p.transformation.scale = interpolate_float(p1.transformation.scale, p2.transformation.scale, factor2);
  p.transformation.rotation = interpolate_float(p1.transformation.rotation, p2.transformation.rotation, factor2);
  p.color_index = p1.color_index;
}

vec2 rotate(in Transformation transformation, vec2 pos) {
  float angle = transformation.rotation * TAU;
  return vec2(
    pos.x * cos(angle) + pos.y * sin(angle),
    -pos.x * sin(angle) + pos.y * cos(angle)
  );
}
vec2 scale(in Transformation transformation, vec2 pos) {
  return pos * transformation.scale;
}
vec2 translate(in Transformation transformation, vec2 pos) {
  return pos + transformation.translation;
}
vec2 transform(in Transformation transformation, vec2 pos) {
  return translate(transformation, scale(transformation, rotate(transformation, pos)));
}


