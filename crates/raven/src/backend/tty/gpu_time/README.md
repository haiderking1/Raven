# Nonblocking GL elapsed intervals

Adaptive and deadline modes create one `GpuTime` for the device renderer. Immediate mode creates none. This is original integration against pinned Smithay 0.7.0, without dependency or registry changes.

## Integration

`render/submit.rs` brackets each DRM `render_frame` attempt, including composition recovery. It ends the query even if rendering fails, before propagating that error, waiting for a required fence, or queueing a frame. The caller flushes the end marker without waiting for it; Smithay had already flushed its render-completion fence before the marker.

`redraw/frame.rs` collects only available results before another render. The returned duration is the maximum valid interval in that batch, a conservative scheduling input. Results arrive later than the render that produced them; they are not assigned to the newest frame or input. No idle polling source is installed.

Frame admission is recorded before updating the scheduling budget. A timing observation cannot move the admission deadline out from under an already-queued frame.

## Capability and ownership

The collector validates `GL_EXT_disjoint_timer_query`, typed entry points and usable counter width. Missing support yields a disabled collector. Four query slots are reused only after completion. Disjoint events invalidate affected results; reported context resets abandon names to that context instead of reusing them.

The collector belongs to one live EGL context and its thread. A lease prevents duplicate collectors from consuming disjoint status. It never overlaps or ends a foreign query. Smithay 0.7 does not use this query target. The collector does not consume GL error state.

VT return discards pre-pause intervals without recycling unfinished queries. Active shutdown destroys queries before the renderer. Inactive shutdown or device failure leaves cleanup to context teardown without unsafe device work. Drop itself makes no GL calls.

Availability checks avoid explicit query-result waits; they do not guarantee bounded CPU latency for every driver call. Elapsed GL intervals can include dependencies and command-stream gaps. They are not CPU-clock timestamps, pure shader cost, browser frame time or photon measurements.

## Validation

Pool regressions exercise bounded reuse, disjoint handling, counter validity, foreign-query ownership and lifecycle failures. The opt-in hardware regression creates an offscreen EGL renderer, performs real rendering, checks framebuffer/viewport preservation and fence ordering, and verifies cleanup and lease reuse. It does not acquire DRM master or change the display.

```sh
cargo test --workspace --locked --offline real_elapsed_query_preserves_renderer_state_and_releases_its_lease -- --ignored --nocapture
```

That regression passed on the available NVIDIA GeForce RTX 3070. It validates the query integration, not the complete DRM scheduler or browser zoom performance.
