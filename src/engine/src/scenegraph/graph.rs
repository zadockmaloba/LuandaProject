use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ptr::null};

#[repr(C)]
#[derive(Serialize, Deserialize, Clone)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Clone)]
pub struct Light {
    pub intensity: f32,
    pub color: [f32; 3],
}

#[repr(C)]
#[derive(Serialize, Deserialize, Clone)]
pub struct Camera {
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Clone)]
pub enum SceneObject {
    Mesh(Mesh),
    Light(Light),
}

#[repr(C)]
#[derive(Serialize, Deserialize, Clone)]
pub struct Scene {
    pub camera: Camera,
    pub lights: HashMap<String, Light>,
    pub meshes: HashMap<String, Mesh>,
}

#[repr(C)]
#[derive(Serialize, Deserialize)]
pub struct SceneGraph {
    pub root: Scene,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            root: Scene {
                camera: Camera {
                    fov: 90.0,
                    aspect_ratio: 16.0 / 9.0,
                    near: 0.1,
                    far: 100.0,
                },
                lights: HashMap::new(),
                meshes: HashMap::new(),
            },
        }
    }

    pub fn from_string(data: &str) -> Self {
        serde_yml::from_str(data).unwrap()
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.root.camera = camera;
    }

    pub fn get_camera(&self) -> &Camera {
        &self.root.camera
    }

    pub fn add_scene_object(&mut self, name: &str, object: SceneObject) {
        match object {
            SceneObject::Mesh(mesh) => {
                self.root.meshes.insert(name.to_string(), mesh);
            }
            SceneObject::Light(light) => {
                self.root.lights.insert(name.to_string(), light);
            }
        }
    }

    pub fn remove_object(&mut self, name: &str) -> bool {
        self.root.meshes.remove(name).is_some() || self.root.lights.remove(name).is_some()
    }

    pub fn find_object<'a>(&'a self, name: &str) -> Option<SceneObject> {
        if let Some(mesh) = self.root.meshes.get(name) {
            return Some(SceneObject::Mesh(mesh.clone()));
        }
        if let Some(light) = self.root.lights.get(name) {
            return Some(SceneObject::Light(light.clone()));
        }
        None
    }

    pub fn serialize(&self) -> String {
        serde_yml::to_string(self).unwrap()
    }
}

impl std::fmt::Display for SceneGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.serialize())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn create_scene_graph() -> *mut SceneGraph {
    Box::into_raw(Box::new(SceneGraph::new()))
}

#[unsafe(no_mangle)]
pub extern "C" fn free_scene_graph(graph: *mut SceneGraph) {
    if !graph.is_null() {
        unsafe {
            let _b = Box::from_raw(graph);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn add_scene_object(graph: *mut SceneGraph, name: *const u8, object: *const SceneObject) {
    if graph.is_null() || name.is_null() || object.is_null() {
        return;
    }
    let graph = unsafe { &mut *graph };
    let name = unsafe { std::ffi::CStr::from_ptr(name as *const i8) }.to_str().unwrap();
    let object = unsafe { &*object };
    graph.add_scene_object(name, object.clone());
}

#[unsafe(no_mangle)]
pub extern "C" fn remove_scene_object(graph: *mut SceneGraph, name: *   const u8) -> bool {
    if graph.is_null() || name.is_null() {
        return false;
    }
    let graph = unsafe { &mut *graph };
    let name = unsafe { std::ffi::CStr::from_ptr(name as *const i8) }.to_str().unwrap();
    graph.remove_object(name)
}

#[unsafe(no_mangle)]
pub extern "C" fn find_scene_object(graph: *const SceneGraph, name: *const u8) -> *const SceneObject {
    if graph.is_null() || name.is_null() {
        return std::ptr::null();
    }
    let graph = unsafe { &*graph };
    let name = unsafe { std::ffi::CStr::from_ptr(name as *const i8) }.to_str().unwrap();
    if let Some(object) = graph.find_object(name) {
        Box::into_raw(Box::new(object))
    } else {
        std::ptr::null()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn scene_graph_from_str(data: *const u8) -> *const SceneGraph {
    if data.is_null() { return std::ptr::null(); }

    let data_str = unsafe{ std::ffi::CStr::from_ptr(data as *const i8) }.to_str().unwrap();

    let graph = SceneGraph::from_string(data_str);

    Box::into_raw(Box::new(graph))
}

#[unsafe(no_mangle)]
pub extern "C" fn set_scene_camera(graph: *mut SceneGraph, camera: *const Camera) {
    if graph.is_null() || camera.is_null() {
        return;
    }
    let graph = unsafe { &mut *graph };
    let camera = unsafe { &*camera };
    graph.set_camera(camera.clone());
}

#[unsafe(no_mangle)]
pub extern "C" fn get_scene_camera(graph: *const SceneGraph) -> *const Camera {
    if graph.is_null() {
        return std::ptr::null();
    }
    let graph = unsafe { &*graph };
    graph.get_camera() as *const Camera
}

#[unsafe(no_mangle)]
pub extern "C" fn serialize_scene_graph(graph: *const SceneGraph) -> *const u8 {
    if graph.is_null() {
        return std::ptr::null();
    }
    let graph = unsafe { &*graph };
    let serialized = graph.serialize();
    let c_string = std::ffi::CString::new(serialized).unwrap();
    c_string.into_raw() as *const u8
}

#[unsafe(no_mangle)]
pub extern "C" fn free_serialized_string(s: *mut u8) {
    if !s.is_null() {
        unsafe {
            let _c_string = std::ffi::CString::from_raw(s as *mut i8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_graph() {
        let mut graph = SceneGraph::new();
        graph.add_scene_object("Mesh1", SceneObject::Mesh(Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            indices: vec![0, 1, 2],
        }));
        assert!(graph.find_object("Mesh1").is_some());
        println!("{}", graph);
        assert!(graph.remove_object("Mesh1"));
        assert!(graph.find_object("Mesh1").is_none());
    }

    #[test]
    fn test_serialization() {
        let mut graph = SceneGraph::new();
        graph.add_scene_object("Mesh1", SceneObject::Mesh(Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            indices: vec![0, 1, 2],
        }));
        let serialized = graph.serialize();
        let deserialized = SceneGraph::from_string(&serialized);
        assert!(deserialized.find_object("Mesh1").is_some());
    }

    #[test]
    fn test_deserialization() {
        let data = r#"
        root:
            camera:
                fov: 90.0
                aspect_ratio: 1.7777777777777777
                near: 0.1
                far: 100.0
            lights:
                Light1:
                    intensity: 1.0
                    color: [1.0, 1.0, 1.0]
            meshes:
                Mesh1:
                    vertices:
                    - [0.0, 0.0, 0.0]
                    - [1.0, 0.0, 0.0]
                    - [0.0, 1.0, 0.0]
                    indices: [0, 1, 2]
            "#;
        let graph = SceneGraph::from_string(data);
        assert!(graph.find_object("Mesh1").is_some());
        assert!(graph.find_object("Light1").is_some());
    }
}
