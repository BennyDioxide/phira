crate::tl_file!("dialog");

use super::{DRectButton, Scroll, Ui};
use crate::{core::BOLD_FONT, ext::RectExt, scene::show_message};
use anyhow::Error;
use macroquad::prelude::*;

const WIDTH_RADIO: f32 = 0.5;
const HEIGHT_RATIO: f32 = 0.7;

#[must_use]
pub struct Dialog {
    title: String,
    message: String,
    buttons: Vec<String>,
    listener: Option<Box<dyn FnMut(i32)>>, // -1 for cancel

    scroll: Scroll,
    window_rect: Option<Rect>,
    rect_buttons: Vec<DRectButton>,
}

impl Default for Dialog {
    fn default() -> Self {
        Self {
            title: tl!("notice").to_string(),
            message: String::new(),
            buttons: vec![tl!("ok").to_string()],
            listener: None,

            scroll: Scroll::new(),
            window_rect: None,
            rect_buttons: vec![DRectButton::new()],
        }
    }
}

impl Dialog {
    pub fn simple(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ..Default::default()
        }
    }

    pub fn plain(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            ..Default::default()
        }
    }

    pub fn error(error: Error) -> Self {
        let error = format!("{error:?}");
        Self {
            title: tl!("error").to_string(),
            message: error.clone(),
            buttons: vec![tl!("error-copy").to_string(), tl!("ok").to_string()],
            listener: Some(Box::new(move |pos| {
                if pos == 0 {
                    unsafe { get_internal_gl() }.quad_context.clipboard_set(&error);
                    show_message(tl!("error-copied")).ok();
                }
            })),

            rect_buttons: vec![DRectButton::new(); 2],
            ..Default::default()
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn buttons(mut self, buttons: Vec<String>) -> Self {
        self.buttons = buttons;
        self.rect_buttons = vec![DRectButton::new(); self.buttons.len()];
        self
    }

    pub fn listener(mut self, f: impl FnMut(i32) + 'static) -> Self {
        self.listener = Some(Box::new(f));
        self
    }

    pub fn show(self) {
        crate::scene::DIALOG.with(|it| *it.borrow_mut() = Some(self));
    }

    pub fn touch(&mut self, touch: &Touch, t: f32) -> bool {
        self.scroll.touch(touch, t);
        let mut exit = false;
        for (index, btn) in self.rect_buttons.iter_mut().enumerate() {
            if btn.touch(touch, t) {
                if let Some(listener) = self.listener.as_mut() {
                    listener(index as i32);
                }
                exit = true;
            }
        }
        if exit {
            return false;
        }
        if self
            .window_rect
            .map_or(true, |rect| rect.contains(touch.position) || touch.phase != TouchPhase::Started)
        {
            true
        } else {
            if let Some(listener) = self.listener.as_mut() {
                listener(-1);
            }
            false
        }
    }

    pub fn update(&mut self, t: f32) {
        self.scroll.update(t);
    }

    pub fn render(&mut self, ui: &mut Ui, t: f32) {
        ui.fill_rect(ui.screen_rect(), Color::new(0., 0., 0., 0.6));
        let mut wr = Rect::new(0., 0., 2. * WIDTH_RADIO, ui.top * 2. * HEIGHT_RATIO);
        wr.x = -wr.w / 2.;
        wr.y = -wr.h / 2.;
        self.window_rect = Some(ui.rect_to_global(wr));
        ui.fill_path(&wr.rounded(0.01), ui.background());

        let s = 0.02;
        let pad = 0.02;
        let bh = 0.09;
        ui.scope(|ui| {
            let s = 0.01;
            let pad = 0.02;
            let mut h = 0.;
            macro_rules! dy {
                ($val:expr) => {{
                    let dy = $val;
                    h += dy;
                    ui.dy(dy);
                }};
            }
            dy!(wr.y + s * 3.);
            let r = BOLD_FONT.with(|it| {
                ui.text(&self.title)
                    .pos(wr.x + pad * 2., 0.)
                    .anchor(0., 0.)
                    .size(0.95)
                    .max_width(wr.w - pad * 2.)
                    .no_baseline()
                    .draw_with_font(it.borrow_mut().as_mut())
            });
            dy!(r.h + s * 2.);
            self.scroll.size((wr.w - pad * 2., wr.bottom() - h - bh - s * 2.));
            ui.dx(wr.x + pad);
            self.scroll.render(ui, |ui| {
                let r = ui
                    .text(&self.message)
                    .pos(pad, 0.)
                    .size(0.43)
                    .max_width(wr.w - pad * 3.)
                    .multiline()
                    .draw();
                (r.w, r.h + 0.04)
            });
        });
        ui.scope(|ui| {
            let bw = (wr.w - pad * (self.buttons.len() + 1) as f32) / self.buttons.len() as f32;
            let mut r = Rect::new(wr.x + pad, wr.bottom() - s - bh, bw, bh);
            for (text, btn) in self.buttons.iter().zip(self.rect_buttons.iter_mut()) {
                btn.render_text(ui, r, t, text, 0.5, true);
                r.x += bw + pad;
            }
        });
    }
}
