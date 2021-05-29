use engine::{
    bevy::{prelude::*, utils::Duration},
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

pub struct Enemy {
    progress: Timer,
}

impl Enemy {
    fn new(run_time: Duration) -> Self {
        Self {
            progress: Timer::new(run_time, false),
        }
    }

    fn tick(&mut self, time: &Time) {
        self.progress.tick(time.delta());
    }

    fn reached_destination(&self) -> bool {
        self.progress.finished()
    }

    fn draw_in_enemy_list(&self, ui: &mut Ui) {
        ui.label(format!(
            "Enemy! Time left: {:.1}s",
            (self.progress.duration() - self.progress.elapsed()).as_secs_f64()
        ));
    }
}

pub fn init_stuff(mut commands: Commands) {
    commands.spawn().insert(Unit::Unready);
    commands.spawn().insert(Unit::Unready);
    commands.spawn().insert(Unit::Unready);
    commands
        .spawn()
        .insert(Enemy::new(Duration::from_secs_f64(20.0)));
}

pub fn gui(
    mut commands: Commands,
    mut egui_ctx: ResMut<EguiContext>,
    _assets: Res<AssetServer>,
    mut units: Query<&mut Unit>,
    time: Res<Time>,
    mut enemies: Query<&mut Enemy>,
) {
    egui::SidePanel::left("side_panel", 200.0).show(egui_ctx.ctx(), |ui| {
        ui.heading("Units");
        for mut unit in units.iter_mut() {
            unit.tick(&time);
            unit.draw_in_unit_list(ui);
        }

        ui.separator();

        ui.heading("Enemies");
        for mut enemy in enemies.iter_mut() {
            enemy.tick(&time);
            if enemy.reached_destination() {
                egui::Window::new("Hit!").show(egui_ctx.ctx(), |ui| {
                    ui.heading("You got hit! You are dead !!!!");
                    if ui.button("Oh dear. :-(").clicked() {
                        std::process::exit(0);
                    };
                });
            }
            enemy.draw_in_enemy_list(ui);
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
