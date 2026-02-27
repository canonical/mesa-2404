# Mesa libraries for `base: core24` snaps

A content snap providing the Mesa userspace libraries and drivers for core24-based snaps.

This supplies the gpu-2404 content interface:

Path|Description|Use
--|--|--
bin/gpu-2404-provider-wrapper|Sets up all the environment|Run your application through it
||
X11/XErrorDB|X11 locales etc|Layout to /usr/share/X11/XErrorDB
||
mir-quirks|Mir configuration for driver support|Mir specific

----

For more information about the `gpu-2404` interface, see: [The gpu-2404 snap interface](https://mir-server.io/docs/the-gpu-2404-snap-interface) documentation.
