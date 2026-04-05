#pragma once

#if defined(__APPLE__)
namespace MTL {struct Device;};
typedef MTL::Device GraphicsDevice;
#elif defined(WIN32)
struct ID3D12Device;
typedef ID3D12Device GraphicsDevice;
#else
#error "Unsupported platform"
#endif

namespace LuandaEditor {

class ViewPortWidget {
    struct ViewPortWidgetPrivate;
    ViewPortWidgetPrivate *_p = nullptr;

public:
    ViewPortWidget(const char *, GraphicsDevice *);
    ViewPortWidget(ViewPortWidget &&) = delete;
    ViewPortWidget(const ViewPortWidget &) = delete;
    ~ViewPortWidget();

    void show(unsigned int width, unsigned int height);
};

}