# Shaders

| File | Stage | Notes |
|------|-------|-------|
| `sprite.vert` | Vertex | Textured colored quad; `mat4` view-proj push constant |
| `sprite.frag` | Fragment | Combined image sampler × vertex color |

CMake compiles these with `glslc` into `${BUILD}/shaders/*.spv`.
