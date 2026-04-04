#pragma once

#include "scenegraph.h"
#include <string.h>

#ifdef __cplusplus
extern "C" {
#endif

// Helper functions for creating scene objects from C

static inline SceneObject create_light_object(float intensity, float r, float g, float b) {
    SceneObject obj;
    obj.tag = SCENE_OBJECT_LIGHT;
    obj.data.light.intensity = intensity;
    obj.data.light.color[0] = r;
    obj.data.light.color[1] = g;
    obj.data.light.color[2] = b;
    return obj;
}

static inline Camera create_camera(float fov, float aspect_ratio, float near, float far) {
    Camera cam;
    cam.fov = fov;
    cam.aspect_ratio = aspect_ratio;
    cam.near = near;
    cam.far = far;
    return cam;
}

// Convenience wrapper for string conversion
static inline void add_scene_object_str(SceneGraph* graph, const char* name, const SceneObject* object) {
    add_scene_object(graph, (const uint8_t*)name, object);
}

static inline bool remove_scene_object_str(SceneGraph* graph, const char* name) {
    return remove_scene_object(graph, (const uint8_t*)name);
}

static inline const SceneObject* find_scene_object_str(const SceneGraph* graph, const char* name) {
    return find_scene_object(graph, (const uint8_t*)name);
}

static inline const SceneGraph* scene_graph_from_yaml(const char* yaml_data) {
    return scene_graph_from_str((const uint8_t*)yaml_data);
}

static inline const char* serialize_scene_graph_str(const SceneGraph* graph) {
    return (const char*)serialize_scene_graph(graph);
}

#ifdef __cplusplus
}
#endif
