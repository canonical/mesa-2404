# mesa libraries for core22 snaps

A content snap providing the mesa userspace libraries and drivers for core22

This supplies the graphics-core22 content interface:

Path|Description|Use
--|--|--
usr/lib/${SNAPCRAFT_ARCH_TRIPLET}/|Shared libraries|add to LD_LIBRARY_PATH
usr/lib/${SNAPCRAFT_ARCH_TRIPLET}/dri/|Drivers|Add to LIBGL_DRIVERS_PATH/LIBVA_DRIVERS_PATH
usr/lib/${SNAPCRAFT_ARCH_TRIPLET}/vdpau/|Video Decode and Presentation|???
||
usr/share/drirc.d|app-specific workarounds|layout to /usr/share/drirc.d
usr/share/glvnd/egl_vendor.d|contains the Mesa ICD|Add to __EGL_VENDOR_LIBRARY_DIRS
usr/share/vulkan|Vulkan ICDs etc|??? maybe: layout to /usr/share/vulkan
usr/share/X11|X11 locales etc|layout to /usr/share/X11
||
etc/mir-quirks|Mir configuration for driver support|Mir specific
etc/vdpau_wrapper.cfg|Not sure if this is useful…|…nor what to do with it

----

For details of the graphics-core22 content interface see:

[TBD]