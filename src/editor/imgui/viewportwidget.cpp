#include "viewportwidget.hpp"
#include <cassert>
#include <cstddef>
#include <scenegraph.h>
#include <imgui.h>

#if defined(__APPLE__)
#include <Metal/Metal.hpp>
#include <Metal/MTLTexture.hpp>
#include <renderer_metal.hpp>
#elif defined(WIN32)
#include <d3d12.h>
#include "imgui_dx12_texture_bridge.hpp"
#include <renderer.h>
#else
#error "Unsupported platform"
#endif

namespace LuandaEditor {

struct ViewPortWidget::ViewPortWidgetPrivate {
    const char *id = nullptr;
    void *renderer = nullptr;
    SceneGraph *scene_graph = nullptr;
    LuandaTextureHandle texture_handle = {
#if defined(WIN32)
        LUANDA_BACKEND_D3D12,
#elif defined(__APPLE__)
        LUANDA_BACKEND_METAL,
#else
#error "Unsupported platform"
#endif
        nullptr
    };
    LuandaExternalDevice external_device = {
#if defined(WIN32)
        LUANDA_BACKEND_D3D12,
#elif defined(__APPLE__)
        LUANDA_BACKEND_METAL,
#else
#error "Unsupported platform"
#endif
        nullptr
    };
#if defined(WIN32)
    ID3D12Resource *last_texture_resource = nullptr;
    D3D12_CPU_DESCRIPTOR_HANDLE texture_srv_cpu = {};
    D3D12_GPU_DESCRIPTOR_HANDLE texture_srv_gpu = {};
    bool has_texture_srv = false;
#endif

    ViewPortWidgetPrivate(const char *id, GraphicsDevice *device) : id(id) {
        external_device.device = device;
        renderer = luanda_renderer_create((int)external_device.backend, &external_device);
        scene_graph = create_scene_graph();
    }

    ~ViewPortWidgetPrivate() {
#if defined(WIN32)
        if (has_texture_srv) {
            luanda_imgui_dx12_free_srv(texture_srv_cpu, texture_srv_gpu);
            has_texture_srv = false;
        }
#endif
        if (renderer) luanda_renderer_destroy(renderer);
        if (scene_graph) free_scene_graph(scene_graph);
    }
};

ViewPortWidget::ViewPortWidget(const char *id, GraphicsDevice *device) : _p(new ViewPortWidgetPrivate(id, device)) {

}

ViewPortWidget::~ViewPortWidget() {
    if (_p) delete _p;
}

void ViewPortWidget::show(unsigned int width, unsigned int height) {
    assert(_p != nullptr);
    assert(_p->id != nullptr);

    ImGui::Begin((const char *)_p->id);

    ImVec2 viewportSize = ImGui::GetContentRegionAvail();
    const unsigned int targetWidth =
        (viewportSize.x > 1.0f) ? static_cast<unsigned int>(viewportSize.x) : (width > 0 ? width : 1u);
    const unsigned int targetHeight =
        (viewportSize.y > 1.0f) ? static_cast<unsigned int>(viewportSize.y) : (height > 0 ? height : 1u);

    luanda_renderer_draw(_p->renderer, (size_t)targetWidth, (size_t)targetHeight);
#if defined(__APPLE__)
    MTL::Texture *game_texture = luanda_renderer_get_texture(_p->renderer);
#elif defined(WIN32)
    luanda_renderer_get_texture(_p->renderer, &_p->texture_handle);
    ID3D12Resource *game_texture = (ID3D12Resource *)_p->texture_handle.handle;
    ImTextureID game_texture_id = (ImTextureID)0;
    if (game_texture != nullptr) {
        if (!_p->has_texture_srv || _p->last_texture_resource != game_texture) {
            if (_p->has_texture_srv) {
                luanda_imgui_dx12_free_srv(_p->texture_srv_cpu, _p->texture_srv_gpu);
                _p->has_texture_srv = false;
            }

            if (luanda_imgui_dx12_alloc_srv(game_texture, &_p->texture_srv_cpu, &_p->texture_srv_gpu)) {
                _p->has_texture_srv = true;
                _p->last_texture_resource = game_texture;
            }
        }

        if (_p->has_texture_srv) {
            game_texture_id = (ImTextureID)_p->texture_srv_gpu.ptr;
        }
    }
#endif

#if defined(__APPLE__)
    if (game_texture != nullptr) {
        // Display the texture in the ImGui window
        ImGui::Image((ImTextureID)game_texture, viewportSize);
    }
#elif defined(WIN32)
    if (game_texture_id != (ImTextureID)0) {
        ImGui::Image(game_texture_id, viewportSize);
    }
#endif
    ImGui::End();
}

};