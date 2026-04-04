#include "viewportwidget.hpp"
#include <Metal/MTLTexture.hpp>
#include <renderer_metal.hpp>
#include <scenegraph.h>
#include <imgui.h>

namespace LuandaEditor {

struct ViewPortWidget::ViewPortWidgetPrivate {
    void *renderer = nullptr;
    SceneGraph *scene_graph = nullptr;

    ViewPortWidgetPrivate(MTL::Device *device) {
        renderer = luanda_renderer_create(device);
        scene_graph = create_scene_graph();
    }

    ~ViewPortWidgetPrivate() {
        if (renderer) luanda_renderer_destroy(renderer);
        if (scene_graph) free_scene_graph(scene_graph);
    }
};

ViewPortWidget::ViewPortWidget(MTL::Device *device) : _p(new ViewPortWidgetPrivate(device)) {

}

ViewPortWidget::~ViewPortWidget() {
    if (_p) delete _p;
}

void ViewPortWidget::show(unsigned int width, unsigned int height) {
    luanda_renderer_render(_p->renderer, (size_t)width, (size_t)height);
    MTL::Texture *game_texture = luanda_renderer_get_texture(_p->renderer);

    ImGui::Begin("Game Viewport");
    if (game_texture != nil) {
        ImVec2 viewportSize = ImGui::GetContentRegionAvail();
        // Display the texture in the ImGui window
        ImGui::Image((ImTextureID)game_texture, viewportSize);
    }
    ImGui::End();
}

};