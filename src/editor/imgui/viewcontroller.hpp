#pragma once

#if defined(__APPLE__)
namespace MTL {struct Device;};
typedef MTL::Device GraphicsDevice;
#elif defined(WIN32)
struct ID3D12Device;
typedef ID3D12Device GraphicsDevice;
#elif defined(__linux__) || defined(__unix__)
struct VkInstance_T;
typedef VkInstance_T* GraphicsDevice;
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