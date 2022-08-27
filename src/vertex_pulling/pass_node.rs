use bevy::{
    core_pipeline::core_3d::Opaque3d,
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_graph::{self, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
        render_phase::{DrawFunctions, RenderPhase, TrackedRenderPass},
        render_resource::{
            LoadOp, Operations, RenderPassDepthStencilAttachment, RenderPassDescriptor,
        },
        renderer::RenderContext,
        view::{ExtractedView, ViewDepthTexture, ViewTarget},
    },
};

pub(crate) const CUBOIDS_PASS: &str = "cuboids_pass";

pub(crate) struct CuboidsPassNode {
    query: QueryState<
        (
            &'static ExtractedCamera,
            &'static RenderPhase<Opaque3d>,
            &'static ViewTarget,
            &'static ViewDepthTexture,
        ),
        With<ExtractedView>,
    >,
}

impl CuboidsPassNode {
    pub(crate) const IN_VIEW: &'static str = "view";

    pub(crate) fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl render_graph::Node for CuboidsPassNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(CuboidsPassNode::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let (camera, cuboids_phase, target, depth) = match self.query.get_manual(world, view_entity)
        {
            Ok(query) => query,
            Err(_) => return Ok(()), // No window
        };

        #[cfg(feature = "trace")]
        let _main_cuboids_pass_span = info_span!("main_cuboids_pass").entered();
        let pass_descriptor = RenderPassDescriptor {
            label: Some("main_cuboids_pass"),
            // NOTE: The cuboids pass loads the color buffer as well as writing to it.
            color_attachments: &[Some(target.get_color_attachment(Operations {
                load: LoadOp::Load,
                store: true,
            }))],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &depth.view,
                // NOTE: The cuboids main pass loads the depth buffer and possibly overwrites it
                depth_ops: Some(Operations {
                    load: LoadOp::Load,
                    store: true,
                }),
                stencil_ops: None,
            }),
        };

        let draw_functions = world.resource::<DrawFunctions<Opaque3d>>();

        let render_pass = render_context
            .command_encoder
            .begin_render_pass(&pass_descriptor);
        let mut draw_functions = draw_functions.write();
        let mut tracked_pass = TrackedRenderPass::new(render_pass);

        if let Some(viewport) = camera.viewport.as_ref() {
            tracked_pass.set_camera_viewport(viewport);
        }

        for item in &cuboids_phase.items {
            let draw_function = draw_functions.get_mut(item.draw_function).unwrap();
            draw_function.draw(world, &mut tracked_pass, view_entity, item);
        }

        Ok(())
    }
}
