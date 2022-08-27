# bevy-aabb-instancing

Render millions of AABBs every frame with an instancing renderer.

![Example](https://raw.githubusercontent.com/ForesightMiningSoftwareCorporation/bevy-aabb-instancing/main/examples/wave.png)

## Plugins

The [`VertexPullingRenderPlugin`] uses the "vertex pulling" technique to render all entities with a [`Cuboids`] component.
In vertex pulling, rather than pushing vertex attributes through the shader pipeline, you only push an index buffer, and the
shader "pulls" your instance data from a storage buffer by decoding the `vertex_index` input.
