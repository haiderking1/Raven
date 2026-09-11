//_DEFINES
#ifdef EXTERNAL
#extension GL_OES_EGL_image_external : require
#endif
precision mediump float;
varying vec2 v_coords;
#ifdef EXTERNAL
uniform samplerExternalOES tex;
#else
uniform sampler2D tex;
#endif
uniform sampler2D previous_image;
uniform float progress;
uniform float alpha;
#ifdef DEBUG_FLAGS
uniform float tint;
#endif
void main() {
    vec4 current = texture2D(tex, v_coords);
#ifdef NO_ALPHA
    current.a = 1.0;
#endif
    vec4 previous = texture2D(previous_image, v_coords);
    gl_FragColor = ((1.0 - progress) * previous + progress * current) * alpha;
#ifdef DEBUG_FLAGS
    gl_FragColor.rgb = mix(gl_FragColor.rgb, vec3(0.0, 0.2, 0.0), tint);
#endif
}
