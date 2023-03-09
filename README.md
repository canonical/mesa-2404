# mesa libraries for core22 snaps

A content snap providing the mesa userspace libraries and drivers for core22

This supplies the graphics-core22 content interface:

Path|Description|Use
--|--|--
usr/bin/graphics-core22-provider-wrapper|Sets up all the environment|Run your application through it
||
usr/share/drirc.d|App-specific workarounds|Layout to /usr/share/drirc.d
usr/share/libdrm|Needed by mesa on AMD GPUs|Layout to /usr/share/libdrm
usr/share/vulkan|Vulkan ICDs etc|??? maybe: layout to /usr/share/vulkan
usr/share/X11|X11 locales etc|Layout to /usr/share/X11
||
etc/mir-quirks|Mir configuration for driver support|Mir specific
etc/vdpau_wrapper.cfg|Not sure if this is useful…|…nor what to do with it

----

For details of the graphics-core22 content interface see:

[TBD]
