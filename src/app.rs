use std::num::NonZeroU32;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

use crate::layout::{Layout, LayoutContext};
use crate::renderer::Renderer;
use crate::styler::StyledTree;

struct App {
    renderer: Renderer,
    styled_tree: StyledTree,
    state: Option<(Arc<Window>, softbuffer::Surface<Arc<Window>, Arc<Window>>)>,
}

impl App {
    fn new(renderer: Renderer, styled_tree: StyledTree) -> Self {
        Self {
            renderer: renderer,
            styled_tree: styled_tree,
            state: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let context = softbuffer::Context::new(Arc::clone(&window)).unwrap();
        let surface = softbuffer::Surface::new(&context, Arc::clone(&window)).unwrap();

        self.state = Some((window, surface));
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(_) | WindowEvent::RedrawRequested => {
                let Some((window, surface)) = self.state.as_mut() else {
                    return;
                };

                let size = window.inner_size();

                let (w, h) = (size.width, size.height);
                if w == 0 || h == 0 {
                    return;
                }

                surface
                    .resize(NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap())
                    .unwrap();

                let mut buf = surface.buffer_mut().unwrap();

                let layout_context = LayoutContext {
                    font: &self.renderer.fonts[0],
                    window_height: h,
                    window_width: w,
                };

                let layout = Layout::build(&self.styled_tree, layout_context);

                self.renderer.draw(&mut buf, layout, w, h);

                buf.present().unwrap();
            }
            _ => (),
        }
    }
}

pub fn init(styled_tree: StyledTree) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let renderer = Renderer::new();
    let mut app = App::new(renderer, styled_tree);

    event_loop.run_app(&mut app).unwrap();
}
