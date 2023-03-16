# mesa libraries for core22 snaps

A content snap providing the mesa userspace libraries and drivers for core22

This supplies the graphics-core22 content interface:

Path|Description|Use
--|--|--
bin/graphics-core22-provider-wrapper|Sets up all the environment|Run your application through it
||
drirc.d|App-specific workarounds|Layout to /usr/share/drirc.d
X11|X11 locales etc|Layout to /usr/share/X11
||
mir-quirks|Mir configuration for driver support|Mir specific

----

For details of the graphics-core22 content interface see:

[TBD]
