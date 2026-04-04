#pragma once

#if defined(__APPLE__)
namespace MTL {struct Device;};
typedef MTL::Device GraphicsDevice;
#else
#error "Unsupported platform"
#endif

namespace LuandaEditor {

class ViewController {
    struct ViewControllerPrivate;
    ViewControllerPrivate *_p = nullptr;

public:
    ViewController(GraphicsDevice *);
    ViewController(ViewController &&) = delete;
    ViewController(const ViewController &) = delete;
    ~ViewController();

    void show();
};

}