# Mesa libraries for `base: core26` snaps

A content snap providing the Mesa userspace libraries and drivers for core26-based snaps.

This supplies the gpu-2604 content interface:

Path|Description|Use
--|--|--
bin/gpu-2604-provider-wrapper|Sets up all the environment|Run your application through it
||
libdrm|Needed by mesa on AMD GPUs|Layout to /usr/share/libdrm
X11/XErrorDB|X11 locales etc|Layout to /usr/share/X11/XErrorDB
||
mir-quirks|Mir configuration for driver support|Mir specific

----

For more information about the `gpu-2604` interface, see: [The gpu-2604 snap interface](https://canonical.com/mir/docs/the-gpu-2604-snap-interface) documentation.
