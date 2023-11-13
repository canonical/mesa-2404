# mesa libraries for core24 snaps

A content snap providing the mesa userspace libraries and drivers for core24

This supplies the graphics-core24 content interface:

Path|Description|Use
--|--|--
bin/graphics-core24-provider-wrapper|Sets up all the environment|Run your application through it
||
drirc.d|App-specific workarounds|Layout to /usr/share/drirc.d
libdrm|Needed by mesa on AMD GPUs|Layout to /usr/share/libdrm
X11|X11 locales etc|Layout to /usr/share/X11
||
mir-quirks|Mir configuration for driver support|Mir specific

----

For more information about the `graphics-core24` interface, see: [The graphics-core24 snap interface](https://mir-server.io/docs/the-graphics-core24-snap-interface) documentation.
