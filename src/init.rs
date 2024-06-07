use std::sync::{Arc, Mutex};

use crate::{buffers::UiBuffers, vertex::Vertex2d};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{
    draw::{DrawState, FillPipeline},
    App, State,
};

impl App<'_> {
    pub fn init() -> Self {
        Self {
            state: State::init(),
            window: None,
            draw_state: None,
        }
    }
}

impl State {
    pub fn init() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self { value: 10. }))
    }
}

impl<'window> DrawState<'window> {
    pub fn init(window: Arc<Window>) -> DrawState<'window> {
        let mut size = Arc::clone(&window).inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = pollster::block_on(async {
            instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    force_fallback_adapter: false,
                    compatible_surface: Some(&surface),
                })
                .await
                .expect("Failed to find an appropriate adapter")
        });

        let (device, queue) = pollster::block_on(async {
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                    },
                    None,
                )
                .await
                .expect("Failed to create device")
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(swapchain_capabilities.formats[0]);

        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();

        config.format = swapchain_format;

        let fill_pipeline = get_fill_pipeline(&device, &config);

        surface.configure(&device, &config);

        let ui_buffers = get_ui_buffers(&device);

        Self {
            window,
            surface,
            config,
            device,
            queue,
            fill_pipeline,
            ui_buffers,
        }
    }
}

fn get_fill_pipeline(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> FillPipeline {
    let fill_shader =
        device.create_shader_module(wgpu::include_wgsl!("../shaders/fill_shader.wgsl"));

    let fill_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Fill pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Fill pipeline"),
        layout: Some(&fill_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &fill_shader,
            entry_point: "vs_main",
            buffers: &[Vertex2d::layout()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &fill_shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(Vertex2d::RECT),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    FillPipeline {
        pipeline,
        vertex_buffer,
    }
}


pub fn get_ui_buffers(device: &wgpu::Device) -> UiBuffers {
    UiBuffers {
        console_bg: get_rect_buffer(device, "Vertex Buffer"),
    }
}

fn get_rect_buffer(device: &wgpu::Device, buffer_label: &str) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(buffer_label),
        contents: bytemuck::cast_slice(Vertex2d::RECT),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    })
}
