use engine::{
    bevy::prelude::*,
    bevy_egui::{egui, egui::Ui, EguiContext},
};

#[derive(Debug, Clone)]
pub enum Unit {
    Unready,
    Preparing(Timer),
    Ready,
}

impl Unit {
    fn tick(&mut self, time: &Time) {
        if let Self::Preparing(timer) = self {
            timer.tick(time.delta());

            if timer.finished() {
                *self = Self::Ready;
            }
        }
    }

    fn draw_in_unit_list(&mut self, ui: &mut Ui) {
        match self {
            Unit::Unready => {
                ui.horizontal(|ui| {
                    ui.label("Unready");
                    if ui.button("Prepare").clicked() {
                        *self = Self::Preparing(Timer::from_seconds(5.0, false))
                    }
                });
            }
            Unit::Ready => {
                ui.label("Ready");
            }
            Unit::Preparing(timer) => {
                ui.label(format!(
                    "Preparing. {:.0} / {:.1} seconds to go.",
                    timer.percent() * 100.0,
                    (timer.duration() - timer.elapsed()).as_secs_f64()
                ));
            }
        };
    }
}

pub enum Enemy {}

pub fn init_stuff(mut commands: Commands) {
    commands.spawn().insert(Unit::Unready);
    commands.spawn().insert(Unit::Unready);
    commands.spawn().insert(Unit::Unready);
}

pub fn gui(
    mut commands: Commands,
    mut egui_ctx: ResMut<EguiContext>,
    _assets: Res<AssetServer>,
    mut units: Query<(Entity, &mut Unit)>,
    time: Res<Time>,
    enemies: Query<(Entity, &Enemy)>,
) {
    egui::SidePanel::left("side_panel", 200.0).show(egui_ctx.ctx(), |ui| {
        ui.heading("Units");
        for (unit_entity, mut unit) in units.iter_mut() {
            unit.tick(&time);
            unit.draw_in_unit_list(ui);
        }
    });

    egui::TopPanel::top("top_panel").show(egui_ctx.ctx(), |ui| {
        // The top panel is often a good place for a menu bar:
        egui::menu::bar(ui, |ui| {
            egui::menu::menu(ui, "File", |ui| {
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });
        });
    });

    egui::CentralPanel::default().show(egui_ctx.ctx(), |ui| {
        ui.heading("Hier könnte Ihre Werbung stehen!");
        egui::warn_if_debug_build(ui);

        ui.separator();

        ui.heading("Central Panel");
        ui.label("The central panel the region left after adding TopPanel's and SidePanel's");
        ui.label("It is often a great place for big things, like drawings:");
    });
}
