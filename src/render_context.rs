use egui_wgpu::ScreenDescriptor;
use wgpu::util::DeviceExt;
use winit::{
    event::{DeviceEvent, WindowEvent},
    window::Window,
};

use crate::{
    scene::{GpuCamera, GpuMaterial, Scene},
    utils::{EguiRenderer, StorageBuffer, UniformBuffer, Vertex},
};

pub struct RenderContext<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    image_bind_group_layout: wgpu::BindGroupLayout,
    image_bind_group: wgpu::BindGroup,
    image_buffer: StorageBuffer,
    camera_buffer: UniformBuffer,
    render_param_buffer: UniformBuffer,
    frame_data_buffer: UniformBuffer,
    scene_bind_group: wgpu::BindGroup,
    scene: Scene,
    latest_scene: Scene,
    pub egui_renderer: EguiRenderer,
    pub fps: f64,
}

// const RGB_TRIANGLE: &[Vertex] = &[
//     Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
//     Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
//     Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
// ];

// https://webgpufundamentals.org/webgpu/lessons/webgpu-large-triangle-to-cover-clip-space.html
// https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/#the-results
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0], // Bottom-left
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [3.0, -1.0], // Bottom-right (extends beyond clip space)
        tex_coords: [2.0, 0.0],
    },
    Vertex {
        position: [-1.0, 3.0], // Top-left (extends beyond clip space)
        tex_coords: [0.0, 2.0],
    },
];

const VERTICES_LEN: usize = VERTICES.len();

impl<'a> RenderContext<'a> {
    pub async fn new(window: &'a Window, scene: &Scene) -> RenderContext<'a> {
        let size;
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                use winit::platform::web::WindowExtWebSys;
                let canvas = window.canvas().unwrap();
                // On the web, `client_width/height` can be 0 during early layout.
                // Fall back to the canvas intrinsic size and clamp to >= 1.
                let mut width = canvas.client_width().max(0) as u32;
                let mut height = canvas.client_height().max(0) as u32;
                if width == 0 || height == 0 {
                    width = canvas.width();
                    height = canvas.height();
                }
                width = width.max(1);
                height = height.max(1);

                size = winit::dpi::PhysicalSize::new(width, height);
            } else {
                size = window.inner_size();
            }
        }

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all),
            ..Default::default()
        });

        log::debug!("Instance: {:?}", instance);
        let surface: wgpu::Surface<'_> = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        log::debug!("Adapter: {:?}", adapter.get_info());
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::default().using_resolution(adapter.limits())
                    } else {
                        wgpu::Limits {
                            max_storage_buffer_binding_size: 512_u32 << 20,
                            ..Default::default()
                        }
                    },
                    label: Some("Device"),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();
        log::debug!("Device: {:?}", device);

        let camera_buffer = {
            let camera = GpuCamera::new(&scene.camera, (size.width, size.height));

            UniformBuffer::new_from_bytes(
                &device,
                bytemuck::bytes_of(&camera),
                0_u32,
                Some("camera buffer"),
            )
        };

        let frame_data_buffer = {
            let frame_data = scene.frame_data;
            UniformBuffer::new_from_bytes(
                &device,
                bytemuck::bytes_of(&frame_data),
                1_u32,
                Some("frame data buffer"),
            )
        };

        let render_param_buffer = {
            UniformBuffer::new_from_bytes(
                &device,
                bytemuck::bytes_of(&scene.render_param),
                2_u32,
                Some("render param buffer"),
            )
        };
        let image_buffer = {
            let buffer = vec![[0_f32; 3]; size.width as usize * size.height as usize];
            StorageBuffer::new_from_bytes(
                &device,
                bytemuck::cast_slice(buffer.as_slice()),
                3_u32,
                Some("image buffer"),
            )
        };

        let image_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    camera_buffer.layout(wgpu::ShaderStages::FRAGMENT),
                    frame_data_buffer.layout(wgpu::ShaderStages::FRAGMENT),
                    render_param_buffer.layout(wgpu::ShaderStages::FRAGMENT),
                    image_buffer.layout(wgpu::ShaderStages::FRAGMENT, false),
                ],
                label: Some("image layout"),
            });

        let image_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &image_bind_group_layout,
            entries: &[
                camera_buffer.binding(),
                frame_data_buffer.binding(),
                render_param_buffer.binding(),
                image_buffer.binding(),
            ],
            label: Some("image bind group"),
        });

        let (scene_bind_group_layout, scene_bind_group) = {
            let objects_buffer = StorageBuffer::new_from_bytes(
                &device,
                bytemuck::cast_slice(scene.objects.as_slice()),
                0_u32,
                Some("objects buffer"),
            );

            let sphere_buffer = StorageBuffer::new_from_bytes(
                &device,
                bytemuck::cast_slice(scene.spheres.as_slice()),
                1_u32,
                Some("sphere buffer"),
            );

            let mut global_texture_data = Vec::new();
            let mut material_data: Vec<GpuMaterial> = Vec::with_capacity(scene.materials.len());
            for material in scene.materials.iter() {
                material_data.push(GpuMaterial::new(material, &mut global_texture_data));
            }

            let material_buffer = StorageBuffer::new_from_bytes(
                &device,
                bytemuck::cast_slice(material_data.as_slice()),
                2_u32,
                Some("material buffer"),
            );

            let texture_buffer = StorageBuffer::new_from_bytes(
                &device,
                bytemuck::cast_slice(global_texture_data.as_slice()),
                3_u32,
                Some("texture buffer"),
            );

            let surfaces_buffer = StorageBuffer::new_from_bytes(
                &device,
                bytemuck::cast_slice(scene.meshes.as_slice()),
                4_u32,
                Some("surfaces buffer"),
            );

            let scene_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        objects_buffer.layout(wgpu::ShaderStages::FRAGMENT, true),
                        sphere_buffer.layout(wgpu::ShaderStages::FRAGMENT, true),
                        material_buffer.layout(wgpu::ShaderStages::FRAGMENT, true),
                        texture_buffer.layout(wgpu::ShaderStages::FRAGMENT, true),
                        surfaces_buffer.layout(wgpu::ShaderStages::FRAGMENT, true),
                    ],
                    label: Some("scene layout"),
                });

            let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &scene_bind_group_layout,
                entries: &[
                    objects_buffer.binding(),
                    sphere_buffer.binding(),
                    material_buffer.binding(),
                    texture_buffer.binding(),
                    surfaces_buffer.binding(),
                ],
                label: Some("scene bind group"),
            });

            (scene_bind_group_layout, scene_bind_group)
        };

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader/raytracing.wgsl"));

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        // if rgb = "fs_main" else "fs_main_srgb"

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        log::debug!("Surface format: {:?}", surface_format);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Configure once at startup. On wasm, you may never get an initial `Resized` event.
        surface.configure(&device, &config);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&image_bind_group_layout, &scene_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: match surface_format.is_srgb() {
                    true => Some("fs_main_srgb"),
                    // for webgpu
                    false => Some("fs_main_rgb"),
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let egui_renderer = EguiRenderer::new(&device, config.format, None, 1, window);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            vertex_buffer,
            image_bind_group_layout,
            image_bind_group,
            image_buffer,
            camera_buffer,
            frame_data_buffer,
            render_param_buffer,
            scene_bind_group,
            scene: scene.clone(),
            latest_scene: scene.clone(),
            egui_renderer,
            fps: 0.0,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // The raytracer stores the image in a storage buffer sized to width*height.
            // When resizing, we must recreate that buffer (otherwise the shader indexes OOB).
            let buffer = vec![[0_f32; 3]; new_size.width as usize * new_size.height as usize];
            self.image_buffer = StorageBuffer::new_from_bytes(
                &self.device,
                bytemuck::cast_slice(buffer.as_slice()),
                3_u32,
                Some("image buffer"),
            );

            self.image_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.image_bind_group_layout,
                entries: &[
                    self.camera_buffer.binding(),
                    self.frame_data_buffer.binding(),
                    self.render_param_buffer.binding(),
                    self.image_buffer.binding(),
                ],
                label: Some("image bind group"),
            });

            // Reset accumulation after resizing.
            self.scene.render_param.total_samples = 0;
            self.scene.frame_data.index = 0;
        }
    }

    pub fn window_event(&mut self, event: &WindowEvent, mouse_pressed: &mut bool) {
        self.egui_renderer.handle_input(self.window, event);
        self.scene
            .camera_controller
            .handle_input(event, mouse_pressed);
    }

    pub fn device_event(&mut self, event: &DeviceEvent, mouse_pressed: bool) {
        self.scene
            .camera_controller
            .handle_mouse(event, mouse_pressed);
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        self.scene
            .camera_controller
            .update_camera(&mut self.scene.camera, dt);

        if self.latest_scene != self.scene {
            let samples_per_pixel = self.latest_scene.render_param.samples_per_pixel;
            self.latest_scene = self.scene.clone();
            self.scene.render_param.total_samples = 0;
            self.scene.render_param.samples_per_pixel = samples_per_pixel;
            self.latest_scene.render_param.samples_per_pixel = samples_per_pixel;
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // On wasm, resizing the browser window typically does NOT emit a reliable `WindowEvent::Resized`.
        // Instead, keep the canvas backing resolution in sync with its CSS size.
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;

            let canvas = self.window.canvas().unwrap();

            let css_width = canvas.client_width().max(1) as f64;
            let css_height = canvas.client_height().max(1) as f64;
            let dpr = web_sys::window()
                .map(|w| w.device_pixel_ratio())
                .unwrap_or(1.0)
                .max(1.0);

            let new_width = (css_width * dpr).round().max(1.0) as u32;
            let new_height = (css_height * dpr).round().max(1.0) as u32;

            if canvas.width() != new_width || canvas.height() != new_height {
                canvas.set_width(new_width);
                canvas.set_height(new_height);
                self.resize(winit::dpi::PhysicalSize::new(new_width, new_height));
            }
        }

        {
            let camera = GpuCamera::new(&self.scene.camera, (self.size.width, self.size.height));

            self.queue
                .write_buffer(&self.camera_buffer.handle(), 0, bytemuck::bytes_of(&camera));

            self.scene.frame_data.width = self.size.width;
            self.scene.frame_data.height = self.size.height;
            self.scene.frame_data.index += 1;

            self.queue.write_buffer(
                &self.frame_data_buffer.handle(),
                0,
                bytemuck::bytes_of(&self.scene.frame_data),
            );

            self.scene.render_param.update();

            self.queue.write_buffer(
                &self.render_param_buffer.handle(),
                0,
                bytemuck::bytes_of(&self.scene.render_param),
            );
        }

        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("View"),
            format: Some(self.config.format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        encoder.insert_debug_marker("Render Pass");

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.012,
                            g: 0.012,
                            b: 0.012,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.image_bind_group, &[]);
            render_pass.set_bind_group(1, &self.scene_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..VERTICES_LEN as u32, 0..1);
        }

        {
            self.egui_renderer.begin_frame(&self.window);

            egui::Window::new("Params")
                // .resizable(true)
                .vscroll(true)
                .default_open(false)
                .collapsible(true)
                .show(self.egui_renderer.context(), |ui| {
                    // ui.label("Label!");

                    // if ui.button("Button!").clicked() {
                    //     println!("boom!")
                    // }

                    // slider for changing the max samples per pixel
                    ui.horizontal(|ui| {
                        ui.label("Max samples per pixel:");
                        ui.add(
                            egui::Slider::new(
                                &mut self.scene.render_param.samples_max_per_pixel,
                                1..=10000,
                            )
                            .text("max samples"),
                        );
                    });

                    // slider for changing the max depth of the ray
                    ui.horizontal(|ui| {
                        ui.label("Max depth:");
                        ui.add(
                            egui::Slider::new(&mut self.scene.render_param.max_depth, 1..=100)
                                .text("depth"),
                        );
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Field of view:");
                        ui.add(
                            egui::Slider::new(&mut self.scene.camera.vfov, 2.0..=179.0).text("fov"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Aperture:");
                        ui.add(
                            egui::Slider::new(&mut self.scene.camera.aperture, 0.0..=1.0)
                                .text("aperture"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Focus distance:");
                        ui.add(
                            egui::Slider::new(&mut self.scene.camera.focus_distance, 0.0..=100.0)
                                .text("focus distance"),
                        );
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "Total samples: {}",
                            self.scene.render_param.total_samples
                        ));
                        ui.label(format!(
                            "Max samples: {}",
                            self.scene.render_param.samples_max_per_pixel
                        ));
                        ui.label(format!("FPS: {:.2}", self.fps));
                    });
                    ui.separator();

                    // camera information
                    ui.label("Camera:");
                    ui.label(format!("Eye direction: {:?}", self.scene.camera.eye_dir));
                    ui.label(format!("Eye position: {:?}", self.scene.camera.eye_pos));
                    ui.label(format!("Up vector: {:?}", self.scene.camera.up));
                });

            self.egui_renderer.end_frame_and_draw(
                &self.device,
                &self.queue,
                &mut encoder,
                &self.window,
                &view,
                ScreenDescriptor {
                    size_in_pixels: self.size.into(),
                    pixels_per_point: self.window.scale_factor() as f32,
                },
            );
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
