use bytes::BytesMut;
use wgpu::util::DeviceExt;

use crate::dsscreen::DSScreen;

const SCENE_WIDTH: u32 = 1270;
const SCENE_HEIGHT: u32 = 720;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    //config: wgpu::SurfaceConfiguration,
    //size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    //window: Window,
    ds_screen_upper: DSScreen,
    ds_screen_lower: DSScreen,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new<
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    >(
        window: &W,
    ) -> Self {
        //  let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: SCENE_WIDTH,
            height: SCENE_HEIGHT,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let diffuse_bytes = include_bytes!("../resources/test/upper_5.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();
        let mut ds_screen_upper = DSScreen::new(
            &device,
            surface_format,
            240,
            400,
            diffuse_rgba.as_raw().as_slice(),
        );
        ds_screen_upper.update_textures(&queue);
        ds_screen_upper.set_position(&queue, 0, 0);

        let diffuse_bytes = include_bytes!("../resources/test/lower_wow.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();
        let mut ds_screen_lower = DSScreen::new(
            &device,
            surface_format,
            240,
            320,
            diffuse_rgba.as_raw().as_slice(),
        );
        ds_screen_lower.update_textures(&queue);
        ds_screen_lower.set_position(&queue, 40, 240);

        Self {
            surface,
            device,
            queue,
            ds_screen_upper,
            ds_screen_lower,
        }
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        self.ds_screen_upper.render(&mut encoder, &view);
        self.ds_screen_lower.render(&mut encoder, &view);

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn write_texture(&mut self, upper_buffer: &BytesMut, lower_buffer: &BytesMut) {
        self.ds_screen_upper.write_texture(upper_buffer);
        self.ds_screen_upper.update_textures(&self.queue);

        self.ds_screen_lower.write_texture(lower_buffer);
        self.ds_screen_lower.update_textures(&self.queue);
    }
}

fn generate_projection_matrix() {
    glam::Mat4::orthographic_rh(0.0, SCENE_WIDTH as f32, SCENE_HEIGHT as f32, 0.0, -2.0, 2.0);
}
