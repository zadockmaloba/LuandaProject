# SceneGraph C API Usage

## Overview

The SceneGraph C API provides bindings to use the Rust-based scene graph from C/C++ code.

## Header Files

- `scenegraph.h` - Core C bindings
- `scenegraph_helpers.h` - Convenience helper functions

## Basic Usage

### Creating a Scene Graph

```c
#include "scenegraph.h"
#include "scenegraph_helpers.h"

int main() {
    // Create a new scene graph
    SceneGraph* graph = create_scene_graph();
    
    // Add a light to the scene
    SceneObject light = create_light_object(1.0f, 1.0f, 1.0f, 1.0f);
    add_scene_object_str(graph, "MainLight", &light);
    
    // Find the light
    const SceneObject* found = find_scene_object_str(graph, "MainLight");
    if (found && found->tag == SCENE_OBJECT_LIGHT) {
        printf("Light intensity: %f\n", found->data.light.intensity);
    }
    
    // Remove the light
    remove_scene_object_str(graph, "MainLight");
    
    // Clean up
    free_scene_graph(graph);
    
    return 0;
}
```

### Loading from YAML

```c
const char* yaml_data = 
    "root:\n"
    "  camera:\n"
    "    fov: 90.0\n"
    "    aspect_ratio: 1.777777\n"
    "    near: 0.1\n"
    "    far: 100.0\n"
    "  lights:\n"
    "    Light1:\n"
    "      intensity: 1.0\n"
    "      color: [1.0, 1.0, 1.0]\n"
    "  meshes: {}\n";

const SceneGraph* graph = scene_graph_from_yaml(yaml_data);
if (graph) {
    const SceneObject* light = find_scene_object_str(graph, "Light1");
    if (light) {
        printf("Found light!\n");
    }
    free_scene_graph((SceneGraph*)graph);
}
```

### Working with Camera

```c
SceneGraph* graph = create_scene_graph();

// Create and set a custom camera
Camera cam = create_camera(75.0f, 16.0f/9.0f, 0.1f, 1000.0f);
set_scene_camera(graph, &cam);

// Get the camera
const Camera* current_cam = get_scene_camera(graph);
printf("Camera FOV: %f\n", current_cam->fov);

free_scene_graph(graph);
```

### Serialization Example

```c
SceneGraph* graph = create_scene_graph();

// Add some objects
SceneObject light = create_light_object(2.0f, 1.0f, 0.8f, 0.6f);
add_scene_object_str(graph, "SunLight", &light);

// Serialize to YAML
const char* yaml = serialize_scene_graph_str(graph);
printf("Scene YAML:\n%s\n", yaml);

// Remember to free the serialized string
free_serialized_string((uint8_t*)yaml);
free_scene_graph(graph);
```

## API Reference

### Scene Graph Management

- `SceneGraph* create_scene_graph(void)` - Create a new empty scene graph
- `void free_scene_graph(SceneGraph* graph)` - Free a scene graph and all its resources

### Camera Management

- `void set_scene_camera(SceneGraph* graph, const Camera* camera)` - Set the scene's camera
- `const Camera* get_scene_camera(const SceneGraph* graph)` - Get the scene's camera

### Scene Object Management

- `void add_scene_object(SceneGraph* graph, const uint8_t* name, const SceneObject* object)` - Add an object to the scene
- `bool remove_scene_object(SceneGraph* graph, const uint8_t* name)` - Remove an object from the scene
- `const SceneObject* find_scene_object(const SceneGraph* graph, const uint8_t* name)` - Find an object by name

### Serialization

- `const SceneGraph* scene_graph_from_str(const uint8_t* data)` - Deserialize a scene graph from YAML
- `const uint8_t* serialize_scene_graph(const SceneGraph* graph)` - Serialize a scene graph to YAML string
- `void free_serialized_string(uint8_t* s)` - Free a string returned by `serialize_scene_graph`

### Helper Functions (scenegraph_helpers.h)

- `SceneObject create_light_object(float intensity, float r, float g, float b)` - Create a light object
- `Camera create_camera(float fov, float aspect_ratio, float near, float far)` - Create a camera
- `void add_scene_object_str(SceneGraph* graph, const char* name, const SceneObject* object)` - Add object using C string
- `bool remove_scene_object_str(SceneGraph* graph, const char* name)` - Remove object using C string
- `const SceneObject* find_scene_object_str(const SceneGraph* graph, const char* name)` - Find object using C string

### Data Structures

#### Camera
```c
typedef struct Camera {
    float fov;
    float aspect_ratio;
    float near;
    float far;
} Camera;
```

#### Light
```c
typedef struct Light {
    float intensity;
    float color[3];  // RGB values
} Light;
```

#### SceneObject
```c
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
```

## Notes

- All string parameters should be null-terminated UTF-8 strings
- Mesh creation from C is not directly supported - meshes should be created via YAML deserialization or through Rust code
- The API is thread-safe only if you ensure exclusive access to each SceneGraph instance
- Always call `free_scene_graph()` to prevent memory leaks
