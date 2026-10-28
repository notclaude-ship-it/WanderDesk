mod catalog;
mod settings;

use catalog::{discover_apps, launch_app, AppEntry};
use eframe::egui::{self, Color32, FontId, Key, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use settings::Settings;
use std::time::{Duration, Instant};

struct WanderDesk {
    apps: Vec<AppEntry>,
    settings: Settings,
    saved_settings: Settings,
    menu: bool,
    search: String,
    position: [f32; 2],
    angle: f32,
    arm: Vec<usize>,
    selected: Option<usize>,
    notice: String,
    last_save: Instant,
}

impl WanderDesk {
    fn new(cc: &eframe::CreationContext<'_>, settings: Settings) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Self {
            apps: discover_apps(),
            saved_settings: settings.clone(),
            settings,
            menu: true,
            search: String::new(),
            position: [0.0, 0.0],
            angle: 0.0,
            arm: Vec::new(),
            selected: None,
            notice: "ZQSD/WASD : bouger · ← → : tourner · E : attacher · Entrée/F : ouvrir".into(),
            last_save: Instant::now(),
        }
    }

    fn world_position(i: usize) -> [f32; 2] {
        [
            ((i * 37 + 13) % 79) as f32 - 39.0,
            ((i * 53 + 29) % 83) as f32 - 41.0,
        ]
    }

    fn project(&self, world: [f32; 2], rect: Rect) -> Option<(Pos2, f32, f32)> {
        let dx = world[0] - self.position[0];
        let dy = world[1] - self.position[1];
        let forward = dx * self.angle.sin() + dy * self.angle.cos();
        if forward <= 0.5 {
            return None;
        }
        let side = dx * self.angle.cos() - dy * self.angle.sin();
        let scale = (rect.height() * 0.8 / forward).min(24.0);
        Some((
            Pos2::new(
                rect.center().x + side * scale,
                rect.center().y + scale * 0.30,
            ),
            scale,
            forward,
        ))
    }

    fn toggle_arm(&mut self) {
        if let Some(i) = self.selected {
            if let Some(slot) = self.arm.iter().position(|n| *n == i) {
                self.arm.remove(slot);
                self.notice = format!("{} détachée", self.apps[i].name);
            } else {
                self.arm.push(i);
                self.notice = format!("{} attachée au bras", self.apps[i].name);
            }
        }
    }

    fn launch_index(&mut self, index: usize) {
        if let Some(app) = self.apps.get(index) {
            self.notice = match launch_app(app) {
                Ok(()) => format!("Lancement : {}", app.name),
                Err(e) => e,
            };
        }
    }

    fn movement(&mut self, ctx: &egui::Context) {
        if self.menu {
            return;
        }
        let (forward, back, left, right, tl, tr, attach, launch, escape) = ctx.input(|i| {
            (
                i.key_down(Key::W) || i.key_down(Key::Z) || i.key_down(Key::ArrowUp),
                i.key_down(Key::S) || i.key_down(Key::ArrowDown),
                i.key_down(Key::A) || i.key_down(Key::Q),
                i.key_down(Key::D),
                i.key_down(Key::ArrowLeft),
                i.key_down(Key::ArrowRight),
                i.key_pressed(Key::E),
                i.key_pressed(Key::Enter) || i.key_pressed(Key::F),
                i.key_pressed(Key::Escape),
            )
        });
        let speed = self.settings.speed * 0.07;
        if forward {
            self.position[0] += self.angle.sin() * speed;
            self.position[1] += self.angle.cos() * speed;
        }
        if back {
            self.position[0] -= self.angle.sin() * speed;
            self.position[1] -= self.angle.cos() * speed;
        }
        if left {
            self.position[0] -= self.angle.cos() * speed;
            self.position[1] += self.angle.sin() * speed;
        }
        if right {
            self.position[0] += self.angle.cos() * speed;
            self.position[1] -= self.angle.sin() * speed;
        }
        if tl {
            self.angle -= 0.035;
        }
        if tr {
            self.angle += 0.035;
        }
        if escape {
            self.menu = true;
        }
        if attach {
            self.toggle_arm();
        }
        if launch {
            if let Some(i) = self.selected {
                self.launch_index(i);
            }
        }
        for digit in 1..=9 {
            let key = [
                Key::Num1,
                Key::Num2,
                Key::Num3,
                Key::Num4,
                Key::Num5,
                Key::Num6,
                Key::Num7,
                Key::Num8,
                Key::Num9,
            ][digit - 1];
            if ctx.input(|i| i.key_pressed(key)) {
                if let Some(index) = self.arm.get(digit - 1).copied() {
                    self.launch_index(index);
                }
            }
        }
        if let Some((distance, index)) = self
            .apps
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let p = Self::world_position(i);
                (
                    ((p[0] - self.position[0]).powi(2) + (p[1] - self.position[1]).powi(2)).sqrt(),
                    i,
                )
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
        {
            if distance < 4.5 {
                self.selected = Some(index);
            }
        }
    }

    fn draw_world(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, Color32::from_rgb(8, 14, 29));
        let horizon = rect.top() + rect.height() * 0.53;
        painter.rect_filled(
            Rect::from_min_max(rect.min, Pos2::new(rect.right(), horizon)),
            0.0,
            Color32::from_rgb(17, 34, 67),
        );
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(rect.left(), horizon), rect.max),
            0.0,
            Color32::from_rgb(13, 29, 40),
        );
        let grid = Color32::from_rgba_unmultiplied(54, 137, 158, 65);
        for n in (-30..=30).step_by(2) {
            for (a, b) in [
                ([n as f32, -50.0], [n as f32, 50.0]),
                ([-50.0, n as f32], [50.0, n as f32]),
            ] {
                if let (Some((pa, _, _)), Some((pb, _, _))) =
                    (self.project(a, rect), self.project(b, rect))
                {
                    painter.line_segment([pa, pb], Stroke::new(1.0_f32, grid));
                }
            }
        }
        let query = self.search.to_lowercase();
        let mut visible: Vec<_> = self
            .apps
            .iter()
            .enumerate()
            .filter(|(_, app)| query.is_empty() || app.name.to_lowercase().contains(&query))
            .filter_map(|(i, _)| {
                let (p, s, d) = self.project(Self::world_position(i), rect)?;
                (d < 70.0 && rect.expand(120.0).contains(p)).then_some((d, i, p, s))
            })
            .collect();
        visible.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (_, i, center, scale) in visible {
            let size = (scale * 0.92).clamp(30.0, 110.0);
            let card = Rect::from_center_size(
                center + Vec2::new(0.0, -size * 0.25),
                Vec2::new(size * 1.5, size * 0.78),
            );
            let active = self.selected == Some(i);
            painter.rect_filled(
                card,
                8.0,
                if active {
                    Color32::from_rgb(30, 91, 103)
                } else {
                    Color32::from_rgb(22, 48, 70)
                },
            );
            painter.rect_stroke(
                card,
                8.0,
                Stroke::new(
                    if active { 2.5_f32 } else { 1.2_f32 },
                    if active {
                        Color32::from_rgb(112, 247, 240)
                    } else {
                        Color32::from_rgb(66, 126, 177)
                    },
                ),
            );
            painter.text(
                card.center(),
                egui::Align2::CENTER_CENTER,
                &self.apps[i].name,
                FontId::proportional((size * 0.15).clamp(9.0, 15.0)),
                Color32::from_rgb(229, 250, 255),
            );
            if response.clicked()
                && card.contains(response.interact_pointer_pos().unwrap_or(Pos2::ZERO))
            {
                self.selected = Some(i);
            }
        }
        let c = rect.center();
        painter.line_segment(
            [c - Vec2::new(8.0, 0.0), c + Vec2::new(8.0, 0.0)],
            Stroke::new(2.0_f32, Color32::WHITE),
        );
        painter.line_segment(
            [c - Vec2::new(0.0, 8.0), c + Vec2::new(0.0, 8.0)],
            Stroke::new(2.0_f32, Color32::WHITE),
        );
        painter.text(
            rect.left_top() + Vec2::new(16.0, 14.0),
            egui::Align2::LEFT_TOP,
            format!("WANDERDESK // {} applications", self.apps.len()),
            FontId::proportional(14.0),
            Color32::WHITE,
        );
        painter.text(
            rect.left_bottom() + Vec2::new(16.0, -16.0),
            egui::Align2::LEFT_BOTTOM,
            &self.notice,
            FontId::proportional(12.0),
            Color32::from_rgb(196, 233, 242),
        );
    }
}

impl eframe::App for WanderDesk {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.movement(ctx);
        if self.settings != self.saved_settings
            && self.last_save.elapsed() >= Duration::from_millis(500)
        {
            match self.settings.save() {
                Ok(()) => self.saved_settings = self.settings.clone(),
                Err(e) => self.notice = format!("Paramètres non sauvegardés : {e}"),
            }
            self.last_save = Instant::now();
        }
        if self.menu {
            egui::CentralPanel::default().show(ctx,|ui| {
                ui.add_space(42.0);
                ui.heading(RichText::new("WANDERDESK").size(42.0).strong().color(Color32::from_rgb(112,247,240)));
                ui.label("Ton bureau est un monde. Explore et lance tes applications Linux.");
                ui.add_space(24.0); ui.heading("Paramètres de session");
                ui.checkbox(&mut self.settings.stereo,"Vue stéréoscopique expérimentale (pas encore OpenXR)");
                ui.add(egui::Slider::new(&mut self.settings.speed,1.0..=15.0).text("Vitesse de déplacement"));
                if ui.checkbox(&mut self.settings.fullscreen,"Plein écran").changed() { ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.settings.fullscreen)); }
                ui.label(format!("{} applications détectées",self.apps.len()));
                if ui.button(RichText::new("▶  JOUER").size(24.0)).clicked() { self.menu=false; }
                if ui.button("Actualiser le catalogue").clicked() { self.apps=discover_apps(); }
                ui.add_space(10.0);
                ui.label("ZQSD/WASD : bouger · ← → : tourner · E : attacher · Entrée/F : lancer · Échap : menu");
                ui.label(RichText::new("Binaire natif Linux — Rust + egui — x86-64 et ARM64").weak());
            });
        } else {
            egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("WANDERDESK")
                            .strong()
                            .color(Color32::from_rgb(112, 247, 240)),
                    );
                    ui.separator();
                    ui.label(format!("{} apps", self.apps.len()));
                    ui.add_sized(
                        [180.0, 24.0],
                        egui::TextEdit::singleline(&mut self.search).hint_text("Rechercher…"),
                    );
                    if ui.button("Menu").clicked() {
                        self.menu = true;
                    }
                    if ui.button("Actualiser").clicked() {
                        self.apps = discover_apps();
                    }
                });
            });
            egui::SidePanel::right("arm")
                .default_width(205.0)
                .show(ctx, |ui| {
                    ui.heading("◈ Bras / raccourcis");
                    ui.label("Touches 1–9 pour lancer");
                    ui.separator();
                    let mut remove = None;
                    let mut launch = None;
                    let arm_snapshot = self.arm.clone();
                    for (slot, index) in arm_snapshot.iter().enumerate() {
                        if let Some(app) = self.apps.get(*index) {
                            ui.horizontal(|ui| {
                                if ui.small_button(format!("{} ×", slot + 1)).clicked() {
                                    remove = Some(slot);
                                }
                                if ui.button(&app.name).clicked() {
                                    launch = Some(*index);
                                }
                            });
                        }
                    }
                    if let Some(slot) = remove {
                        self.arm.remove(slot);
                    }
                    if let Some(index) = launch {
                        self.launch_index(index);
                    }
                    ui.separator();
                    if let Some(i) = self.selected {
                        if let Some(app) = self.apps.get(i) {
                            ui.label(RichText::new(&app.name).strong());
                            if let Some(icon) = &app.icon {
                                ui.label(format!("Icône : {icon}"));
                            }
                            if ui.button("Lancer l’application").clicked() {
                                self.launch_index(i);
                            }
                            if ui
                                .button(if self.arm.contains(&i) {
                                    "Détacher du bras"
                                } else {
                                    "Attacher au bras"
                                })
                                .clicked()
                            {
                                self.toggle_arm();
                            }
                        }
                    }
                });
            egui::TopBottomPanel::bottom("touch_controls").show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let step = self.settings.speed * 0.8;
                    if ui.add_sized([48.0, 38.0], egui::Button::new("◀")).clicked() {
                        self.position[0] -= self.angle.cos() * step;
                        self.position[1] += self.angle.sin() * step;
                    }
                    if ui.add_sized([48.0, 38.0], egui::Button::new("▲")).clicked() {
                        self.position[0] += self.angle.sin() * step;
                        self.position[1] += self.angle.cos() * step;
                    }
                    if ui.add_sized([48.0, 38.0], egui::Button::new("▼")).clicked() {
                        self.position[0] -= self.angle.sin() * step;
                        self.position[1] -= self.angle.cos() * step;
                    }
                    if ui.add_sized([48.0, 38.0], egui::Button::new("▶")).clicked() {
                        self.position[0] += self.angle.cos() * step;
                        self.position[1] -= self.angle.sin() * step;
                    }
                    ui.separator();
                    if ui
                        .add_sized([54.0, 38.0], egui::Button::new("↶ Tourner"))
                        .clicked()
                    {
                        self.angle -= 0.25;
                    }
                    if ui
                        .add_sized([54.0, 38.0], egui::Button::new("Tourner ↷"))
                        .clicked()
                    {
                        self.angle += 0.25;
                    }
                    if ui
                        .add_sized([70.0, 38.0], egui::Button::new("E · Attacher"))
                        .clicked()
                    {
                        self.toggle_arm();
                    }
                    if ui
                        .add_sized([70.0, 38.0], egui::Button::new("F · Ouvrir"))
                        .clicked()
                    {
                        if let Some(i) = self.selected {
                            self.launch_index(i);
                        }
                    }
                });
            });
            egui::CentralPanel::default().show(ctx, |ui| self.draw_world(ui));
        }
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

fn main() -> eframe::Result<()> {
    let settings = Settings::load();
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([800.0, 520.0])
        .with_fullscreen(settings.fullscreen);
    eframe::run_native(
        "WanderDesk",
        eframe::NativeOptions {
            viewport,
            ..Default::default()
        },
        Box::new(move |cc| Box::new(WanderDesk::new(cc, settings))),
    )
}
