//_DEFINES
#ifdef EXTERNAL
#extension GL_OES_EGL_image_external : require
#endif
precision highp float;
varying vec2 v_coords;
#ifdef EXTERNAL
uniform samplerExternalOES tex;
#else
uniform sampler2D tex;
#endif
uniform float alpha;
#ifdef DEBUG_FLAGS
uniform float tint;
#endif
//_CORNERS
void main() {
    vec4 color = texture2D(tex, v_coords);
#ifdef NO_ALPHA
    color.a = 1.0;
#endif
    gl_FragColor = color * alpha * corner_coverage();
#ifdef DEBUG_FLAGS
    gl_FragColor.rgb = mix(gl_FragColor.rgb, vec3(0.0, 0.2, 0.0), tint);
#endif
}
