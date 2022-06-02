use archie::wgpu;
use egui_wgpu::renderer::RenderPass;

pub use egui;

pub struct Egui {
    context: egui::Context,
    renderer: RenderPass,
    winit_state: egui_winit::State,
    output: Option<egui::FullOutput>,
}

impl Egui {
    pub fn new(ctx: &archie::Context) -> Self {
        let device = ctx.device();

        let egui_renderer = RenderPass::new(device, ctx.surface_format(), 1);
        let egui_winit_state = egui_winit::State::new(
            device.limits().max_texture_dimension_2d as usize,
            ctx.window(),
        );
        let egui_context = egui::Context::default();

        Egui {
            context: egui_context,
            winit_state: egui_winit_state,
            renderer: egui_renderer,
            output: None,
        }
    }

    pub fn context(&self) -> &egui::Context {
        &self.context
    }

    pub fn update(&mut self, ctx: &archie::Context, mut ui: impl FnMut(&egui::Context)) {
        let mut output = {
            let input = self.winit_state.take_egui_input(ctx.window());
            self.context.begin_frame(input);
            ui(&self.context);
            self.context.end_frame()
        };

        self.winit_state.handle_platform_output(
            ctx.window(),
            &self.context,
            output.platform_output.take(),
        );

        self.output = Some(output);
    }

    pub fn draw(
        &mut self,
        ctx: &mut archie::Context,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        if let Some(output) = self.output.take() {
            let meshes = self.context.tessellate(output.shapes);
            let screen = egui_wgpu::renderer::ScreenDescriptor {
                size_in_pixels: [ctx.width(), ctx.height()],
                pixels_per_point: ctx.window().scale_factor() as f32,
            };
            self.renderer.execute(encoder, view, &meshes, &screen, None);
        }
    }

    pub fn handle_event(&mut self, event: &archie::winit::event::Event<()>) {
        if let archie::winit::event::Event::WindowEvent { event, .. } = event {
            self.winit_state.on_event(&self.context, event);
        }
    }
}
