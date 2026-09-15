precision highp float;
uniform vec4 ring_color;
uniform vec4 inner_bounds;
uniform float inner_radius;
uniform float alpha;
//_CORNERS
void main() {
    float coverage = max(0.0, corner_coverage() - rounded_coverage(inner_bounds, inner_radius));
    gl_FragColor = ring_color * alpha * coverage;
}
