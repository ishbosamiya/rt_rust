use std::{
    cell::RefCell,
    rc::Rc,
    sync::{mpsc, Arc, RwLock},
    thread::JoinHandle,
};

use crate::{
    camera::{self, Camera},
    image::Image,
    progress::Progress,
    rasterize::{
        drawable::Drawable, gpu_immediate::GPUImmediate, gpu_utils, shader,
        texture::TextureRGBAFloat,
    },
    viewport::Viewport,
};

use super::{RayTraceMessage, RayTraceParams};

#[derive(Debug)]
struct RenderData {
    target_viewport: Viewport,
    trace_max_depth: usize,
    samples_per_pixel: usize,
    camera: Camera,
    frame_count: usize,
}

impl RenderData {
    fn new(
        target_viewport: Viewport,
        trace_max_depth: usize,
        samples_per_pixel: usize,
        camera: Camera,
    ) -> Self {
        Self {
            target_viewport,
            trace_max_depth,
            samples_per_pixel,
            camera,
            frame_count: 0,
        }
    }
}

#[derive(Debug)]
enum ViewportRenderMessage {
    Restart(RenderData),
    Stop,
    KillThread,
}

pub struct ViewportRenderer {
    message_sender: mpsc::Sender<ViewportRenderMessage>,
    thread_handle: Option<JoinHandle<()>>,
    rendered_image: Arc<RwLock<Image>>,
    rendered_texture: Rc<RefCell<TextureRGBAFloat>>,
}

impl ViewportRenderer {
    pub fn new(
        target_viewport: Viewport,
        trace_max_depth: usize,
        samples_per_pixel: usize,
        camera: Camera,
        path_trace_progress: Arc<RwLock<Progress>>,
        ray_trace_thread_sender: mpsc::Sender<RayTraceMessage>,
    ) -> Self {
        let rendered_image = Image::new(1, 1);
        let rendered_texture = Rc::new(RefCell::new(TextureRGBAFloat::from_image(&rendered_image)));
        let rendered_image = Arc::new(RwLock::new(rendered_image));

        let (message_sender, message_receiver) = mpsc::channel();

        let thread_handle = Some(Self::spawn_thread(
            message_receiver,
            path_trace_progress,
            ray_trace_thread_sender,
            rendered_image.clone(),
        ));

        let res = Self {
            message_sender,
            thread_handle,
            rendered_image,
            rendered_texture,
        };

        res.restart_render(target_viewport, trace_max_depth, samples_per_pixel, camera);

        res
    }

    fn stop_job(
        stop_render: Arc<RwLock<bool>>,
        thread_handle: Option<JoinHandle<()>>,
    ) -> Option<JoinHandle<()>> {
        *stop_render.write().unwrap() = true;
        let thread_handle = thread_handle.and_then(|join_handle| {
            join_handle.join().unwrap();
            None
        });
        *stop_render.write().unwrap() = false;
        thread_handle
    }

    /// spawn a job that progressively increases the size of the image
    /// that is rendered until completion
    fn spawn_job(
        mut render_data: RenderData,
        stop_render: Arc<RwLock<bool>>,
        path_trace_progress: Arc<RwLock<Progress>>,
        ray_trace_thread_sender: mpsc::Sender<RayTraceMessage>,
        rendered_image: Arc<RwLock<Image>>,
    ) -> JoinHandle<()> {
        assert_eq!(render_data.frame_count, 0);

        std::thread::spawn(move || {
            let mut do_next_render = true;
            let starting_dimension = 32.0;
            let size_multipler: f64 = 2.0;
            let viewport_width = render_data.target_viewport.get_width() as f64;
            let viewport_height = render_data.target_viewport.get_height() as f64;
            let get_dimensions = |frame_count: usize| {
                let dimension = starting_dimension * size_multipler.powf(frame_count as f64);

                if viewport_width > viewport_height {
                    (dimension, (dimension * viewport_height / viewport_width))
                } else {
                    ((dimension * viewport_width / viewport_height), dimension)
                }
            };

            let samples_per_pixel = render_data.samples_per_pixel.min(1);

            let starting_dimensions = get_dimensions(render_data.frame_count);

            ray_trace_thread_sender
                .send(RayTraceMessage::StartRender(RayTraceParams::new(
                    starting_dimensions.0.floor() as usize,
                    starting_dimensions.1.floor() as usize,
                    render_data.trace_max_depth,
                    samples_per_pixel,
                    render_data.camera.clone(),
                    rendered_image.clone(),
                )))
                .unwrap();

            render_data.frame_count += 1;

            loop {
                if *stop_render.read().unwrap() {
                    ray_trace_thread_sender
                        .send(RayTraceMessage::StopRenderImmediately)
                        .unwrap();
                    break;
                }

                let progress = path_trace_progress.read().unwrap().get_progress();
                if (progress - 1.0).abs() < f64::EPSILON {
                    if do_next_render {
                        let dimensions = get_dimensions(render_data.frame_count);
                        let dimensions =
                            if dimensions.0 >= viewport_width || dimensions.1 >= viewport_height {
                                do_next_render = false;
                                (viewport_width, viewport_height)
                            } else {
                                dimensions
                            };

                        // in the last run, do required number of samples per pixel
                        let samples_per_pixel = if do_next_render {
                            samples_per_pixel
                        } else {
                            render_data.samples_per_pixel
                        };

                        ray_trace_thread_sender
                            .send(RayTraceMessage::StartRender(RayTraceParams::new(
                                dimensions.0.floor() as usize,
                                dimensions.1.floor() as usize,
                                render_data.trace_max_depth,
                                samples_per_pixel,
                                render_data.camera.clone(),
                                rendered_image.clone(),
                            )))
                            .unwrap();

                        path_trace_progress.write().unwrap().reset();

                        render_data.frame_count += 1;
                    } else {
                        break;
                    }
                }
            }
        })
    }

    fn spawn_thread(
        message_receiver: mpsc::Receiver<ViewportRenderMessage>,
        path_trace_progress: Arc<RwLock<Progress>>,
        ray_trace_thread_sender: mpsc::Sender<RayTraceMessage>,
        rendered_image: Arc<RwLock<Image>>,
    ) -> JoinHandle<()> {
        let stop_render = Arc::new(RwLock::new(false));
        let mut thread_handle: Option<JoinHandle<()>> = None;

        std::thread::spawn(move || loop {
            let get_latest_message = || {
                let message = message_receiver.recv().unwrap();
                // TODO: it might make sense to go over all the list
                // of messages and stop in case KillThread is
                // found. As of right now KillThread should be last
                // message received but it may not be necessary in the
                // future which can lead to problems.
                message_receiver.try_iter().last().unwrap_or(message)
            };
            let message = get_latest_message();

            match message {
                ViewportRenderMessage::Restart(render_data) => {
                    thread_handle = Self::stop_job(stop_render.clone(), thread_handle);
                    assert!(thread_handle.is_none(), "ensure no job is running");
                    thread_handle = Some(Self::spawn_job(
                        render_data,
                        stop_render.clone(),
                        path_trace_progress.clone(),
                        ray_trace_thread_sender.clone(),
                        rendered_image.clone(),
                    ));
                }
                ViewportRenderMessage::Stop => {
                    thread_handle = Self::stop_job(stop_render.clone(), thread_handle);
                }
                ViewportRenderMessage::KillThread => {
                    Self::stop_job(stop_render.clone(), thread_handle);
                    break;
                }
            }

            // Sleep for some time before continuing the loop to try
            // to receive messages. This is to ensure that the
            // messages received are bundled which allows for only the
            // latest message to be processed. A restart message is
            // expensive compared to the rest, since it needs some
            // setup to be done and cannot be stopped during its
            // setup. Without waiting for messages to be bundled,
            // there can be multiple restart messages and each will be
            // processed even though it is not required. Better to
            // wait for some time before continuing the loop.
            std::thread::sleep(std::time::Duration::from_millis(150));
        })
    }

    pub fn stop_render(&self) {
        self.message_sender
            .send(ViewportRenderMessage::Stop)
            .unwrap();
    }

    pub fn restart_render(
        &self,
        target_viewport: Viewport,
        trace_max_depth: usize,
        samples_per_pixel: usize,
        mut camera: Camera,
    ) {
        *camera.get_sensor_mut() = Some(camera::Sensor::new(
            target_viewport.get_width() as _,
            target_viewport.get_height() as _,
        ));
        self.message_sender
            .send(ViewportRenderMessage::Restart(RenderData::new(
                target_viewport,
                trace_max_depth,
                samples_per_pixel,
                camera,
            )))
            .unwrap();
    }

    fn kill_thread(&self) {
        self.stop_render();
        self.message_sender
            .send(ViewportRenderMessage::KillThread)
            .unwrap();
    }
}

impl Drop for ViewportRenderer {
    fn drop(&mut self) {
        self.kill_thread();
        self.thread_handle.take().unwrap().join().unwrap();
    }
}

pub struct ViewportRendererDrawData {
    imm: Rc<RefCell<GPUImmediate>>,
}

impl ViewportRendererDrawData {
    pub fn new(imm: Rc<RefCell<GPUImmediate>>) -> Self {
        Self { imm }
    }
}

impl Drawable for ViewportRenderer {
    type ExtraData = ViewportRendererDrawData;

    type Error = ();

    fn draw(&self, extra_data: &mut Self::ExtraData) -> Result<(), Self::Error> {
        let screen_texture_shader = shader::builtins::get_screen_texture_shader()
            .as_ref()
            .unwrap();
        screen_texture_shader.use_shader();
        screen_texture_shader.set_int("image\0", 25);
        screen_texture_shader.set_float("alpha\0", 1.0);

        self.rendered_texture
            .borrow_mut()
            .update_from_image(&self.rendered_image.read().unwrap());

        self.rendered_texture.borrow_mut().activate(25);

        unsafe {
            gl::Disable(gl::DEPTH_TEST);
        }

        gpu_utils::draw_screen_quad_with_uvs(
            &mut extra_data.imm.borrow_mut(),
            screen_texture_shader,
        );

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
        }

        Ok(())
    }

    fn draw_wireframe(&self, _extra_data: &mut Self::ExtraData) -> Result<(), Self::Error> {
        unreachable!("wireframe drawing is not supported")
    }
}
