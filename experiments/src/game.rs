use engine::{
    bevy::{prelude::*, utils::Duration},
    bevy_egui::{egui, egui::Ui, EguiContext},
};

#[derive(Debug, Clone)]
pub enum Unit {
    Unready,
    Preparing(Timer),
    Ready,
    Patrolling(Timer),
    Returning(Timer),
}

impl Unit {
    fn tick(&mut self, time: &Time) {
        match self {
            Self::Preparing(timer) => {
                timer.tick(time.delta());

                if timer.finished() {
                    *self = Self::Ready;
                }
            }
            Self::Patrolling(timer) => {
                timer.tick(time.delta());

                if timer.finished() {
                    *self = Self::Unready;
                }
            }
            Self::Returning(timer) => {
                timer.tick(time.delta());

                if timer.finished() {
                    *self = Self::Unready;
                }
            }
            _ => {}
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
            Unit::Preparing(timer) => {
                ui.label(format!(
                    "Preparing. {:.0} / {:.1} seconds to go.",
                    timer.percent() * 100.0,
                    (timer.duration() - timer.elapsed()).as_secs_f64()
                ));
            }
            Unit::Ready => {
                ui.horizontal(|ui| {
                    ui.label("Ready");
                    if ui.button("Take off!").clicked() {
                        *self = Self::Patrolling(Timer::from_seconds(30.0, false))
                    }
                });
            }
            Unit::Patrolling(timer) => {
                ui.label(format!(
                    "Patrolling. Time remaining: {:.1}s",
                    (timer.duration() - timer.elapsed()).as_secs_f64()
                ));
            }
            Unit::Returning(timer) => {
                ui.label(format!(
                    "Returning. Time remaining: {:.1}s",
                    (timer.duration() - timer.elapsed()).as_secs_f64()
                ));
            }
        };
    }

    fn progress_percent(&self) -> f32 {
        match self {
            Self::Patrolling(timer) => timer.percent(),
            _ => 0.0,
        }
    }

    fn return_to_base(&mut self) {
        if let Self::Patrolling(timer) = self {
            *self = Self::Returning(timer.clone());
        } else {
            panic!("Invalid state for returning to base.");
        }
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

    fn remaining_percent(&self) -> f32 {
        self.progress.percent_left()
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

pub fn units_meet_enemies(
    mut commands: Commands,
    mut units: Query<&mut Unit>,
    mut enemies: Query<(Entity, &mut Enemy)>,
) {
    let mut units: Vec<_> = units
        .iter_mut()
        .filter(|unit| matches!(**unit, Unit::Patrolling(_)))
        .collect();
    units.sort_by(|a, b| {
        b.progress_percent()
            .partial_cmp(&a.progress_percent())
            .unwrap()
    });

    let mut enemies: Vec<_> = enemies.iter_mut().collect();
    enemies.sort_by(|(_, a), (_, b)| {
        a.remaining_percent()
            .partial_cmp(&b.remaining_percent())
            .unwrap()
    });

    if (units.len() == 0) || (enemies.len() == 0) {
        return;
    }

    let mut first_unit = units.remove(0);
    let (first_enemy_entity, first_enemy) = enemies.remove(0);

    if first_unit.progress_percent() >= first_enemy.remaining_percent() {
        first_unit.return_to_base();
        commands.entity(first_enemy_entity).despawn();
    }
}

pub struct EnemySpawner {
    timer: Timer,
}

impl Default for EnemySpawner {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_secs_f64(15.0), true),
        }
    }
}

impl EnemySpawner {
    fn tick(&mut self, time: &Time, mut commands: Commands) {
        self.timer.tick(time.delta());

        if self.timer.finished() {
            commands
                .spawn()
                .insert(Enemy::new(Duration::from_secs_f64(20.0)));
        }
    }
}

pub fn spawn_enemies(
    mut enemy_spawner: ResMut<EnemySpawner>,
    time: Res<Time>,
    mut commands: Commands,
) {
    enemy_spawner.tick(&time, commands);
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
                egui::Window::new("Visitor!").show(egui_ctx.ctx(), |ui| {
                    ui.heading("Visitor has arrived! You are dead !!!!");
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
