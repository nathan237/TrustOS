import usb.core, usb.backend.libusb1, libusb_package
be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
d = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
if d:
    speed_names = {0:'Unknown',1:'Low(1.5M)',2:'Full(12M)',3:'High/USB2(480M)',4:'Super/USB3(5G)',5:'SuperPlus(10G)'}
    print("speed:", d.speed, "=", speed_names.get(d.speed, "?"))
    print("bcdUSB: 0x%04x" % d.bcdUSB)
    print("bMaxPacketSize0:", d.bMaxPacketSize0)
    try: d.set_configuration()
    except: pass
    dh = d._ctx.handle
    print("handle:", dh.handle)
    lib = be.lib
    if hasattr(lib, 'libusb_get_device'):
        lib.libusb_get_device.restype = __import__('ctypes').c_void_p
        lib.libusb_get_device.argtypes = [__import__('ctypes').c_void_p]
        dev_ptr = lib.libusb_get_device(dh.handle.value)
        print("libusb dev_ptr:", hex(dev_ptr) if dev_ptr else None)
        if hasattr(lib, 'libusb_get_device_speed'):
            lib.libusb_get_device_speed.restype = __import__('ctypes').c_int
            lib.libusb_get_device_speed.argtypes = [__import__('ctypes').c_void_p]
            spd = lib.libusb_get_device_speed(dev_ptr)
            print("libusb speed:", spd, "=", speed_names.get(spd, "?"))
else:
    print("no device")
