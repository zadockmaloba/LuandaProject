#pragma once

namespace MTL {struct Device;};

namespace LuandaEditor {

class ViewPortWidget {
    struct ViewPortWidgetPrivate;
    ViewPortWidgetPrivate *_p = nullptr;

public:
    ViewPortWidget(const char *, MTL::Device *);
    ViewPortWidget(ViewPortWidget &&) = delete;
    ViewPortWidget(const ViewPortWidget &) = delete;
    ~ViewPortWidget();

    void show(unsigned int width, unsigned int height);
};

}