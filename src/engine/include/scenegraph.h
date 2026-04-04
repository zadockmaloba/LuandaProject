#pragma once

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Forward declarations
typedef struct Mesh Mesh;
typedef struct Light Light;
typedef struct Camera Camera;
typedef struct SceneObject SceneObject;
typedef struct Scene Scene;
typedef struct SceneGraph SceneGraph;

// Camera struct
typedef struct Camera {
    float fov;
    float aspect_ratio;
    float near;
    float far;
} Camera;

// Light struct
typedef struct Light {
    float intensity;
    float color[3];
} Light;

// Mesh struct (opaque - managed by Rust)
// Note: The actual Vec<T> fields are managed internally by Rust
// C code should not directly access these fields
typedef struct Mesh {
    void* vertices_ptr;
    uintptr_t vertices_len;
    uintptr_t vertices_cap;
    void* indices_ptr;
    uintptr_t indices_len;
    uintptr_t indices_cap;
} Mesh;

// SceneObject enum (tagged union)
typedef enum SceneObjectTag {
    SCENE_OBJECT_MESH = 0,
    SCENE_OBJECT_LIGHT = 1,
} SceneObjectTag;

typedef struct SceneObject {
    SceneObjectTag tag;
    union {
        Mesh mesh;
        Light light;
    } data;
} SceneObject;

// Core API functions
SceneGraph* create_scene_graph(void);
void free_scene_graph(SceneGraph* graph);

// Camera management
void set_scene_camera(SceneGraph* graph, const Camera* camera);
const Camera* get_scene_camera(const SceneGraph* graph);

// Scene object management
void add_scene_object(SceneGraph* graph, const uint8_t* name, const SceneObject* object);
bool remove_scene_object(SceneGraph* graph, const uint8_t* name);
const SceneObject* find_scene_object(const SceneGraph* graph, const uint8_t* name);

// Serialization
const SceneGraph* scene_graph_from_str(const uint8_t* data);
const uint8_t* serialize_scene_graph(const SceneGraph* graph);
void free_serialized_string(uint8_t* s);

#ifdef __cplusplus
}
#endif