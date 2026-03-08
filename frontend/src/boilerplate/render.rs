use crate::boilerplate::ui;
use image::EncodableLayout;
use std::sync::Arc;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    wgt::{DeviceDescriptor, SamplerDescriptor, TextureViewDescriptor},
    *,
};
use winit::{event::MouseButton, keyboard::KeyCode, window::Window};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    tex_index: u32,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Uint32];

    fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct RenderState {
    window: Arc<Window>,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
    scene: Arc<dyn ui::Scene>,
}

impl RenderState {
    pub async fn initialize(window: Arc<Window>, scene: Arc<dyn ui::Scene>) -> RenderState {
        // Obtain the GPU
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        // Obtain the Surface (Screen)
        let surface = instance.create_surface(window.clone()).unwrap();

        // Obtain the Adapter (Translation Layer)
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();

        // Obtain the Device and Queue
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                required_features: Features::IMMEDIATES | Features::BUFFER_BINDING_ARRAY,
                ..Default::default()
            })
            .await
            .unwrap();

        // Obtain the Configuration
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .unwrap_or(&surface_caps.formats[0])
            .clone();
        let size = window.inner_size();
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Begin shader construction
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Main Shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // BindGroupLayouts
        let texture_bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2Array,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bgl],
            immediate_size: 0,
        });

        // Render pipeline
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            vertex: VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout()],
                compilation_options: Default::default(),
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            multisample: MultisampleState {
                count: 1,
                mask: u64::MAX,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            depth_stencil: None,
            cache: None,
        });

        surface.configure(&device, &config);
        window.request_redraw();
        Self {
            window,
            surface,
            device,
            queue,
            config,
            pipeline,
            scene,
        }
    }

    pub fn resize(&mut self) {
        let size = self.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    #[allow(unused)]
    pub fn handle_key(&mut self, key: KeyCode, pressed: bool) {}
    #[allow(unused)]
    pub fn handle_mouse(&mut self, button: MouseButton, pressed: bool) {}

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        self.window.request_redraw();

        let elements = self.scene.elements();
        if elements.is_empty() {
            return Ok(());
        }

        let frame = self.surface.get_current_texture()?;

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        {
            let view = frame.texture.create_view(&TextureViewDescriptor::default());

            let vertex_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                usage: BufferUsages::VERTEX,
                contents: bytemuck::cast_slice(
                    &elements
                        .iter()
                        .enumerate()
                        .map(|(i, el)| {
                            let corners = el.corners();
                            let i = i as u32;
                            [
                                Vertex {
                                    // Bottom Left
                                    position: corners[0],
                                    tex_coords: [0., 1.],
                                    tex_index: i,
                                },
                                Vertex {
                                    // Bottom Right
                                    position: corners[1],
                                    tex_coords: [1., 1.],
                                    tex_index: i,
                                },
                                Vertex {
                                    // Top Left
                                    position: corners[2],
                                    tex_coords: [0., 0.],
                                    tex_index: i,
                                },
                                Vertex {
                                    // Top Right
                                    position: corners[3],
                                    tex_coords: [1., 0.],
                                    tex_index: i,
                                },
                                Vertex {
                                    // Top Left
                                    position: corners[2],
                                    tex_coords: [0., 0.],
                                    tex_index: i,
                                },
                                Vertex {
                                    // Bottom Right
                                    position: corners[1],
                                    tex_coords: [1., 1.],
                                    tex_index: i,
                                },
                            ]
                        })
                        .collect::<Vec<_>>(),
                ),
            });

            let tex_dims = elements[0].image.dimensions();
            let texture = self
                .device
                .create_texture_with_data(
                    &self.queue,
                    &TextureDescriptor {
                        label: Some("Texture"),
                        size: Extent3d {
                            width: tex_dims.0,
                            height: tex_dims.1,
                            depth_or_array_layers: elements.len() as u32,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: TextureDimension::D2,
                        format: TextureFormat::Rgba8UnormSrgb,
                        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                        view_formats: &[],
                    },
                    wgt::TextureDataOrder::LayerMajor,
                    elements
                        .iter()
                        .map(|el| el.image.as_bytes())
                        .collect::<Vec<_>>()
                        .concat()
                        .as_slice(),
                )
                .create_view(&TextureViewDescriptor {
                    dimension: Some(TextureViewDimension::D2Array),
                    array_layer_count: Some(elements.len() as u32),
                    mip_level_count: Some(1),
                    ..Default::default()
                });

            let sampler = self.device.create_sampler(&SamplerDescriptor::default());
            let texture_bg = self.device.create_bind_group(&BindGroupDescriptor {
                label: Some("Texture Bind Group"),
                layout: &self.pipeline.get_bind_group_layout(0),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&texture),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&sampler),
                    },
                ],
            });

            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                    resolve_target: None,
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &texture_bg, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..((6 * elements.len()) as u32), 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }
}
