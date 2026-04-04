/**
 * Example C program demonstrating the SceneGraph C API
 * 
 * Compile with:
 *   clang example.c -L./target/release -lengine -o example
 *   ./example
 */

#include "scenegraph.h"
#include "scenegraph_helpers.h"
#include <stdio.h>

int main() {
    printf("=== SceneGraph C API Example ===\n\n");
    
    // 1. Create a new scene graph
    printf("1. Creating scene graph...\n");
    SceneGraph* graph = create_scene_graph();
    if (!graph) {
        fprintf(stderr, "Failed to create scene graph\n");
        return 1;
    }
    
    // 2. Set up the camera
    printf("2. Setting up camera...\n");
    Camera cam = create_camera(90.0f, 16.0f/9.0f, 0.1f, 100.0f);
    set_scene_camera(graph, &cam);
    
    const Camera* current_cam = get_scene_camera(graph);
    printf("   Camera FOV: %.1f degrees\n", current_cam->fov);
    printf("   Aspect Ratio: %.2f\n", current_cam->aspect_ratio);
    
    // 3. Add lights to the scene
    printf("\n3. Adding lights...\n");
    
    SceneObject sun_light = create_light_object(1.5f, 1.0f, 0.95f, 0.8f);
    add_scene_object_str(graph, "SunLight", &sun_light);
    printf("   Added SunLight\n");
    
    SceneObject fill_light = create_light_object(0.5f, 0.6f, 0.7f, 1.0f);
    add_scene_object_str(graph, "FillLight", &fill_light);
    printf("   Added FillLight\n");
    
    // 4. Find and inspect a light
    printf("\n4. Finding and inspecting SunLight...\n");
    const SceneObject* found = find_scene_object_str(graph, "SunLight");
    if (found && found->tag == SCENE_OBJECT_LIGHT) {
        const Light* light = &found->data.light;
        printf("   Intensity: %.2f\n", light->intensity);
        printf("   Color: RGB(%.2f, %.2f, %.2f)\n", 
               light->color[0], light->color[1], light->color[2]);
    }
    
    // 5. Serialize the scene
    printf("\n5. Serializing scene to YAML...\n");
    const char* yaml = serialize_scene_graph_str(graph);
    if (yaml) {
        printf("%s\n", yaml);
        free_serialized_string((uint8_t*)yaml);
    }
    
    // 6. Remove an object
    printf("6. Removing FillLight...\n");
    bool removed = remove_scene_object_str(graph, "FillLight");
    printf("   Removal %s\n", removed ? "successful" : "failed");
    
    // 7. Verify removal
    const SceneObject* check = find_scene_object_str(graph, "FillLight");
    printf("   FillLight %s in scene\n", check ? "still exists" : "removed");
    
    // 8. Load from YAML
    printf("\n8. Loading scene from YAML...\n");
    const char* yaml_data = 
        "root:\n"
        "  camera:\n"
        "    fov: 75.0\n"
        "    aspect_ratio: 1.5\n"
        "    near: 0.05\n"
        "    far: 200.0\n"
        "  lights:\n"
        "    KeyLight:\n"
        "      intensity: 2.0\n"
        "      color: [1.0, 1.0, 1.0]\n"
        "  meshes: {}\n";
    
    const SceneGraph* loaded_graph = scene_graph_from_yaml(yaml_data);
    if (loaded_graph) {
        printf("   Scene loaded successfully\n");
        
        const Camera* loaded_cam = get_scene_camera(loaded_graph);
        printf("   Loaded camera FOV: %.1f\n", loaded_cam->fov);
        
        const SceneObject* key_light = find_scene_object_str(loaded_graph, "KeyLight");
        if (key_light && key_light->tag == SCENE_OBJECT_LIGHT) {
            printf("   Found KeyLight with intensity: %.1f\n", 
                   key_light->data.light.intensity);
        }
        
        free_scene_graph((SceneGraph*)loaded_graph);
    }
    
    // 9. Cleanup
    printf("\n9. Cleaning up...\n");
    free_scene_graph(graph);
    
    printf("\nDone!\n");
    return 0;
}
