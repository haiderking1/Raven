uniform vec4 corner_bounds;
uniform float corner_radius;
float rounded_coverage(vec4 bounds, float radius) {
    vec2 half_size = bounds.zw * 0.5;
    vec2 q = abs(gl_FragCoord.xy - bounds.xy - half_size) - half_size + radius;
    float distance = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
    return clamp(0.5 - distance, 0.0, 1.0);
}

float corner_coverage() { return rounded_coverage(corner_bounds, corner_radius); }
