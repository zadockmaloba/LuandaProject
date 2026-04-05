#include "viewcontroller.hpp"
#include "viewportwidget.hpp"
#include <imgui.h>
#include <memory>
#include <vector>

#if defined(__APPLE__)
#include <Metal/Metal.hpp>
#elif defined(WIN32)
#include <d3d12.h>
#else
#error "Unsupported platform"
#endif

namespace LuandaEditor {

struct ViewController::ViewControllerPrivate {
    typedef std::vector<std::unique_ptr<ViewPortWidget>> list_of_viewports_t;

    GraphicsDevice *device = nullptr;
    list_of_viewports_t viewport_widgets = {};

    ViewControllerPrivate(GraphicsDevice *device) : device(device) {
        viewport_widgets.emplace_back(std::make_unique<ViewPortWidget>("ViewPort 1", device));
        viewport_widgets.emplace_back(std::make_unique<ViewPortWidget>("ViewPort 2", device));
    }

    ~ViewControllerPrivate() {
    }

};

ViewController::ViewController(GraphicsDevice *device) : _p(new ViewControllerPrivate(device)) {}

ViewController::~ViewController() {
    if (_p) delete _p;
}

void ViewController::show() {
    assert(_p != nullptr);

    ImGui::DockSpaceOverViewport(0, ImGui::GetMainViewport());

    ImGui::SetNextWindowSize({400,400});

    //FIXME: Use dynamic size instead of hardcoded values
    for (auto &v : _p->viewport_widgets) 
        if (v.get()) v->show(800, 600);

    ImGui::Begin("TextEditor");
    //self.editor.Render("TextEditor");
    ImGui::End();

    ImGui::SetNextWindowSize({400, ImGui::GetWindowHeight()});
    ImGui::Begin("Scene");
    ImGui::End();
}

} // namespace LuandaEditor