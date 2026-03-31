**Integrating The Scenegraph**

- The scene data lives in graph.rs: `SceneGraph` owns hash maps of named `Mesh`/`Light` objects alongside a single `Camera`. To render it you need a bridge that converts those CPU-side structs into GPU buffers, uniform blocks, and draw calls.
- The Metal backend in metal.rs currently uploads one hard-coded triangle once inside `MetalRenderer::new()` and never looks at the scene graph. The `draw()` method simply binds that cached buffer and issues `drawPrimitives` for three vertices, so step one is to make the renderer accept some scene data (pass `&SceneGraph` into `draw()` or provide a `submit_scene(&SceneGraph)` method).

**Suggested Pipeline**

1. **Decide on a vertex layout**  
   - Right now `Mesh.vertices` is `Vec<[f32; 3]>`. Decide whether that is final (positions only) or whether you need normals/UVs/colors. Whatever layout you choose, create an `MTLVertexDescriptor` and describe the stride/attributes so Metal knows how to read your vertex buffer. Store that descriptor next to the pipeline state so you can reuse it when creating the pipeline.

2. **Convert meshes to GPU buffers**  
   - Add a helper such as `struct MeshBuffers { vertex: Retained<ProtocolObject<dyn MTLBuffer>>, index: Option<Retained<...>>, index_count: usize }`.  
   - Provide `MetalRenderer::upload_mesh(&self, mesh: &Mesh) -> MeshBuffers` that calls `newBufferWithBytes_length_options`, similar to the triangle upload already happening in `MetalRenderer::new()`.  
   - Maintain a map from mesh names (the keys used in the scene graph) to `MeshBuffers`, so you only re-upload when the mesh changes. A simple approach is to call an `ensure_uploaded(name, mesh)` function each frame that compares lengths/hashes to decide whether to recreate.

3. **Camera and per-draw uniforms**  
   - `SceneGraph::root.camera` already stores projection parameters. Create a uniform struct (e.g. `CameraUniform { view_proj: float4x4 }`) and add an `MTLBuffer` updated every frame with the matrix derived from the camera. Bind it via `setVertexBuffer`/`setFragmentBuffer` before drawing meshes. If you need per-object transforms later, extend `SceneObject::Mesh` to include a transform matrix or store it in another map.

4. **Iterate the scene during `draw()`**  
   - Change `MetalRenderer::draw(...)` to accept `scene: &SceneGraph`. Inside, after configuring the encoder and pipeline state, loop through `scene.root.meshes` (or expose an iterator). For each mesh:
     ```rust
     for (name, mesh) in &scene.root.meshes {
         let buffers = self.ensure_mesh_buffers(name, mesh);
         encoder.setVertexBuffer_offset_atIndex(Some(&*buffers.vertex), 0, 0);
         if let Some(index_buffer) = &buffers.index {
             encoder.drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset(
                 MTLPrimitiveType::Triangle,
                 buffers.index_count as u64,
                 MTLIndexType::UInt32,
                 index_buffer,
                 0,
             );
         } else {
             encoder.drawPrimitives_vertexStart_vertexCount(
                 MTLPrimitiveType::Triangle,
                 0,
                 mesh.vertices.len() as u64,
             );
         }
     }
     ```
     This replaces the fixed `drawPrimitives(..., 3)` call currently in metal.rs.

5. **Expose FFI entry points that accept the scene**  
   - The C ABI already exports `create_scene_graph`, `add_scene_object`, etc., so whichever host is driving the engine can hand you a populated `SceneGraph`. Add a new exported function like `luanda_renderer_draw_scene(renderer, descriptor, drawable, scene_graph)` that forwards the pointer to `MetalRenderer::draw_scene`. Internally, you dereference the raw pointer and call the updated `draw()`.

6. **Handle synchronization/lifetimes**  
   - Because Metal expects buffers to outlive command buffer submission, keep `Retained<MTLBuffer>` fields on `MetalRenderer`. When meshes are removed (`SceneGraph::remove_object`), drop their buffers from the cache so GPU memory is reclaimed.
   - If you foresee frequent edits, consider double-buffering or staging buffers to avoid blocking on CPU uploads.

7. **Lighting and materials**  
   - Lights are already represented on the CPU. Once geometry is drawing, extend your shader in metal.rs to accept a uniform buffer of lights and apply them in the fragment shader. You can serialize `Scene.root.lights` into an array of structs, upload it each frame, and bind it to buffer slot 1 while keeping vertices at slot 0.

**Next Steps**

1. Decide on the exact vertex format and update both `scenegraph::Mesh` and the Metal shader/vertex descriptor.
2. Implement a mesh-buffer cache inside `MetalRenderer`, feeding it from the hash maps in `SceneGraph`.
3. Pass the scene (and eventually camera/light uniforms) into `luanda_renderer_draw` so every frame pulls data from the scenegraph instead of the baked triangle.