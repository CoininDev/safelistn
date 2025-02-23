# SafeListn
SafeListn is a simple application for audio compression in real time using Rust and JACK API. 
Have you ever gotten frustrated watching a movie at night—turning the volume up for quiet dialogue, then scrambling to lower it when loud sound effects kick in? SafeListn is designed to automatically adjust the volume, making late-night watching more comfortable.


⚠ This project is still in development and primarily serves as a learning exercise.

## What's Already Done
- [x] Creating a JACK client and redirecting system ports.
- [x] Running a basic audio compression algorithm.


## Planned Features
- [ ] Customizable compression parameters via command-line flags (e.g. `safelistn --threshold=0.3 --attack_ms=5`)
- [ ] Proper handling of interruptions (restoring system audio routing).
- [ ] Improved compression algorithm for better audio control.

