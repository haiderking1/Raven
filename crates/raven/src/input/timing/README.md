# Input timing

Set RAVEN_INPUT_TIMING=1 to enable batched compositor-boundary observations. The default path reads no diagnostic clocks. Combine with RAVEN_FRAME_TIMING=1 when measuring a real VT session and redirect stderr to a file rather than rendering the logs in a terminal.

The collector records source timestamp age at dispatch, observation age at explicit flush attempts and accepted frame queues, queue-to-kernel-presentation intervals and conditional GPU sync waits. Coalesced observations retain oldest and latest ages. Reports run every two seconds through the existing deadline source. VT transitions reset pending observations and exclude inactive time.

These are not input-to-photon or causal application-response measurements. A cursor-only frame may contain the latest pointer position while an application still renders. Flush completion does not acknowledge client receipt. Missing, future, regressed or incompatible timestamps are counted separately.
