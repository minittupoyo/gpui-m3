//! Loading indicator（M3 Expressive）: 図形がモーフィングしながら回転する

use std::{f32::consts::TAU, time::Duration};

use gpui::{
    Animation, AnimationExt as _, App, ElementId, IntoElement, PathBuilder, RenderOnce,
    SharedString, Window, canvas, div, point, prelude::*, px,
};

use super::{ActiveTheme, f};

/// (山の数, 凹みの深さ) で表した「クッキー型」の図形列
const SHAPES: [(f32, f32); 7] = [
    (10., 0.12), // soft burst
    (9., 0.16),  // cookie 9
    (5., 0.22),  // pentagon
    (2., 0.32),  // pill
    (8., 0.10),  // sunny
    (4., 0.22),  // cookie 4
    (6., 0.14),  // flower
];

fn radius(shape: (f32, f32), a: f32) -> f32 {
    let (n, k) = shape;
    1. - k * (1. - (n * a).cos()) / 2.
}

/// オーバーシュート付きの ease-out（Expressive のバネっぽさ）
fn ease_out_back(x: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.;
    1. + c3 * (x - 1.).powi(3) + c1 * (x - 1.).powi(2)
}

#[derive(IntoElement)]
pub struct LoadingIndicator {
    id: SharedString,
    contained: bool,
    size: f32,
}

impl LoadingIndicator {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            contained: false,
            size: 48.,
        }
    }
    /// 円形コンテナ付き
    pub fn contained(mut self, v: bool) -> Self {
        self.contained = v;
        self
    }
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for LoadingIndicator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let c = cx.m3().colors.clone();
        let (container, color) = if self.contained {
            (Some(c.primary_container), c.on_primary_container)
        } else {
            (None, c.primary)
        };
        let n = SHAPES.len() as f32;
        let per_shape = 650u64;
        let scale = self.size * 38. / 48. / 2. * if self.contained { 0.8 } else { 1. };

        let mut root = div()
            .size(px(self.size))
            .flex_none()
            .rounded_full();
        if let Some(bg) = container {
            root = root.bg(bg);
        }
        root.with_animation(
            ElementId::Name(format!("{}::morph", self.id).into()),
            Animation::new(Duration::from_millis(per_shape * SHAPES.len() as u64)).repeat(),
            move |el, t| {
                let pos = t * n;
                let i = pos.floor() as usize % SHAPES.len();
                let j = (i + 1) % SHAPES.len();
                let local = pos.fract();
                // 前半でモーフし、後半は保持
                let p = ease_out_back((local / 0.6).min(1.));
                let rotation = t * TAU * 2. + (i as f32 + p) * TAU / 7.;
                el.child(
                    canvas(
                        |_, _, _| {},
                        move |b, _, window, _| {
                            let cc = b.center();
                            let (x0, y0) = (f(cc.x), f(cc.y));
                            let mut path = PathBuilder::fill();
                            let steps = 144;
                            for s in 0..=steps {
                                let a = s as f32 / steps as f32 * TAU;
                                let r1 = radius(SHAPES[i], a);
                                let r2 = radius(SHAPES[j], a);
                                let r = (r1 + (r2 - r1) * p) * scale;
                                let pt = point(
                                    px(x0 + r * (a + rotation).cos()),
                                    px(y0 + r * (a + rotation).sin()),
                                );
                                if s == 0 {
                                    path.move_to(pt);
                                } else {
                                    path.line_to(pt);
                                }
                            }
                            path.close();
                            if let Ok(path) = path.build() {
                                window.paint_path(path, color);
                            }
                        },
                    )
                    .size_full(),
                )
            },
        )
    }
}
