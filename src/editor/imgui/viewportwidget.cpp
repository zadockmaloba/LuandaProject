#include "viewportwidget.hpp"
#include <Metal/MTLTexture.hpp>
#include <cassert>
#include <cstddef>
#include <renderer_metal.hpp>
#include <scenegraph.h>
#include <imgui.h>

namespace LuandaEditor {

struct ViewPortWidget::ViewPortWidgetPrivate {
    const char *id = nullptr;
    void *renderer = nullptr;
    SceneGraph *scene_graph = nullptr;

    ViewPortWidgetPrivate(const char *id, MTL::Device *device) : id(id) {
        renderer = luanda_renderer_create(device);
        scene_graph = create_scene_graph();
    }

    ~ViewPortWidgetPrivate() {
        if (renderer) luanda_renderer_destroy(renderer);
        if (scene_graph) free_scene_graph(scene_graph);
    }
};

ViewPortWidget::ViewPortWidget(const char *id, MTL::Device *device) : _p(new ViewPortWidgetPrivate(id, device)) {

}

ViewPortWidget::~ViewPortWidget() {
    if (_p) delete _p;
}

void ViewPortWidget::show(unsigned int width, unsigned int height) {
    assert(_p != nullptr);
    assert(_p->id != nullptr);

    luanda_renderer_render(_p->renderer, (size_t)width, (size_t)height);
    MTL::Texture *game_texture = luanda_renderer_get_texture(_p->renderer);

    ImGui::Begin((const char *)_p->id);
    if (game_texture != nil) {
        ImVec2 viewportSize = ImGui::GetContentRegionAvail();
        // Display the texture in the ImGui window
        ImGui::Image((ImTextureID)game_texture, viewportSize);
    }
    ImGui::End();
}

};