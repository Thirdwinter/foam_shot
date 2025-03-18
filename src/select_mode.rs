use cairo::{Context, Format, ImageSurface};
use smithay_client_toolkit::shm::slot::{self, Buffer};
use wayland_client::protocol::{wl_output, wl_shm, wl_surface};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, Layer},
    zwlr_layer_surface_v1::{self, Anchor, KeyboardInteractivity},
};

use crate::{check_options, shot_fome::ShotFome};

#[derive(Default)]
pub struct SelectMode {
    pub surface: Option<wl_surface::WlSurface>,
    pub layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    pub buffer: Option<Buffer>,
}

impl SelectMode {
    pub fn prev_select(
        &mut self,
        phys_width: Option<i32>,
        phys_height: Option<i32>,
        layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
        output: Option<wl_output::WlOutput>,
        qh: Option<wayland_client::QueueHandle<ShotFome>>,
    ) {
        let (phys_width, phys_height, surface, layer_shell, output, qh) = check_options!(
            phys_width,
            phys_height,
            self.surface.as_ref(),
            layer_shell,
            output,
            qh
        );
        // NOTE: 创建layer
        let layer = zwlr_layer_shell_v1::ZwlrLayerShellV1::get_layer_surface(
            &layer_shell,
            &surface,
            Some(&output),
            Layer::Overlay,
            "foam_select".to_string(),
            &qh,
            2,
        );

        layer.set_anchor(Anchor::all());

        layer.set_exclusive_zone(-1); // 将表面扩展到锚定边缘

        layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
        self.layer_surface = Some(layer);

        surface.damage(0, 0, phys_width, phys_height);
        surface.commit();
    }
    pub fn before_select_handle(
        &mut self,
        phys_width: Option<i32>,
        phys_height: Option<i32>,
        pool: &mut slot::SlotPool,
    ) {
        let (phys_width, phys_height, surface) =
            check_options!(phys_width, phys_height, self.surface.as_ref());
        let (buffer, canvas) = pool
            .create_buffer(
                phys_width as i32,
                phys_height as i32,
                phys_width as i32 * 4,
                wl_shm::Format::Argb8888,
            )
            .unwrap();
        canvas.fill(100);

        buffer.attach_to(surface).unwrap();
        self.buffer = Some(buffer);
        // 请求重绘
        self.surface
            .as_ref()
            .unwrap()
            .damage_buffer(0, 0, phys_width, phys_height);
        surface.commit();
        // self.select_surface.as_ref().unwrap().commit();
    }

    pub fn update_select(
        &mut self,
        phys_width: Option<i32>,
        phys_height: Option<i32>,
        pool: &mut slot::SlotPool,
        pointer_start: Option<(f64, f64)>,
        current_pos: Option<(f64, f64)>,
    ) {
        let (phys_width, phys_height, (start_x, start_y), (end_x, end_y)) =
            check_options!(phys_width, phys_height, pointer_start, current_pos);

        let (buffer, canvas) = pool
            .create_buffer(
                phys_width as i32,
                phys_height as i32,
                phys_width as i32 * 4,
                wl_shm::Format::Argb8888,
            )
            .unwrap();
        canvas.fill(0);
        let cairo_surface = unsafe {
            ImageSurface::create_for_data(
                std::slice::from_raw_parts_mut(canvas.as_mut_ptr(), canvas.len()),
                Format::ARgb32,
                phys_width as i32,
                phys_height as i32,
                phys_width as i32 * 4,
            )
            .map_err(|e| format!("Failed to create Cairo surface: {}", e))
            .unwrap()
        };

        // 创建 Cairo 上下文
        let ctx = Context::new(&cairo_surface)
            .map_err(|e| format!("Failed to create Cairo context: {}", e))
            .unwrap();

        // 填充整个表面为白色半透明
        ctx.set_source_rgba(1.0, 1.0, 1.0, 0.3);
        ctx.paint().unwrap();

        let width = end_x - start_x;
        let height = end_y - start_y;
        // 清除矩形区域以显示透明
        ctx.set_operator(cairo::Operator::Clear);
        ctx.rectangle(start_x, start_y, width, height);
        ctx.fill().unwrap();
        cairo_surface.flush();

        buffer.attach_to(self.surface.as_ref().unwrap()).unwrap();
        self.buffer = Some(buffer);
        // 请求重绘
        self.surface
            .as_ref()
            .unwrap()
            .damage_buffer(0, 0, phys_width, phys_height);
        self.surface.as_ref().unwrap().commit();
    }
}
