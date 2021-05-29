use std::{marker::PhantomData, sync::Arc};

use engine::{
    bevy::{prelude::*, utils::Duration},
    bevy_egui::{egui, egui::Ui, EguiContext},
};

#[derive(Debug, Clone)]
pub enum Unit {
    Mothballed,
    UnMothballing(Timer, Token<ParkingSpace>),
    ParkedUnready(Token<ParkingSpace>),
    ParkedPreparing(Timer, Token<ParkingSpace>),
    ParkedReady(Token<ParkingSpace>),
    Patrolling(Timer),
    Returning(Timer),
    WaitingToPark,
}

impl Unit {
    fn tick(&mut self, time: &Time, parking_spaces: &mut TokenPool<ParkingSpace>) {
        match self {
            Self::ParkedPreparing(timer, parking_space) => {
                timer.tick(time.delta());

                if timer.finished() {
                    *self = Self::ParkedReady(parking_space.clone());
                }
            }
            Self::Patrolling(timer) => {
                timer.tick(time.delta());

                if timer.finished() {
                    self.try_to_park(parking_spaces);
                }
            }
            Self::Returning(timer) => {
                timer.tick(time.delta());

                if timer.finished() {
                    self.try_to_park(parking_spaces);
                }
            }
            Self::UnMothballing(timer, parking_space) => {
                timer.tick(time.delta());

                if timer.finished() {
                    *self = Self::ParkedUnready(parking_space.clone());
                }
            }
            Self::WaitingToPark => {
                self.try_to_park(parking_spaces);
            }
            _ => {}
        }
    }

    fn try_to_park(&mut self, parking_spaces: &mut TokenPool<ParkingSpace>) {
        if let Some(parking_space) = parking_spaces.try_take() {
            *self = Self::ParkedUnready(parking_space);
        } else {
            *self = Self::WaitingToPark;
        }
    }

    fn draw_in_unit_list(&mut self, ui: &mut Ui, parking_spaces: &mut TokenPool<ParkingSpace>) {
        match self {
            Unit::Mothballed => {
                ui.horizontal(|ui| {
                    ui.label("Mothballed");
                    if !parking_spaces.can_take() {
                        ui.set_enabled(false);
                    }

                    if ui.button("UnMothball").clicked() {
                        let parking_space = parking_spaces.try_take().unwrap();

                        *self =
                            Self::UnMothballing(Timer::from_seconds(10.0, false), parking_space);
                    }
                });
            }
            Unit::UnMothballing(timer, _) => {
                ui.label(format!(
                    "UnMothballing. {:.0} / {:.1} seconds to go.",
                    timer.percent() * 100.0,
                    (timer.duration() - timer.elapsed()).as_secs_f64()
                ));
            }
            Unit::ParkedUnready(parking_space) => {
                let prepare_clicked = ui.horizontal(|ui| {
                    ui.label("Unready");
                    ui.button("Prepare").clicked()
                });

                if prepare_clicked.inner {
                    *self = Self::ParkedPreparing(
                        Timer::from_seconds(5.0, false),
                        parking_space.clone(),
                    )
                }
            }
            Unit::ParkedPreparing(timer, _) => {
                ui.label(format!(
                    "Preparing. {:.0} / {:.1} seconds to go.",
                    timer.percent() * 100.0,
                    (timer.duration() - timer.elapsed()).as_secs_f64()
                ));
            }
            Unit::ParkedReady(_) => {
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
            Unit::WaitingToPark => {
                ui.label("Waiting for parking space.");
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
    commands.spawn().insert(Unit::Mothballed);
    commands.spawn().insert(Unit::Mothballed);
    commands.spawn().insert(Unit::Mothballed);
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

    if units.is_empty() || enemies.is_empty() {
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

pub fn spawn_enemies(mut enemy_spawner: ResMut<EnemySpawner>, time: Res<Time>, commands: Commands) {
    enemy_spawner.tick(&time, commands);
}

#[derive(Debug, Clone)]
pub struct ParkingSpace {}

type Token<T> = Arc<PhantomData<T>>;

pub struct TokenPool<T> {
    token_holder: Arc<PhantomData<T>>,
    max_count: usize,
}

impl<T> TokenPool<T> {
    pub fn new(initial_count: usize) -> Self {
        Self {
            token_holder: Arc::new(PhantomData),
            max_count: initial_count,
        }
    }

    pub fn try_take(&mut self) -> Option<Token<T>> {
        if !self.can_take() {
            return None;
        }

        Some(self.token_holder.clone())
    }

    pub fn can_take(&self) -> bool {
        Arc::strong_count(&self.token_holder) < self.max_count + 1
    }
}

pub fn gui(
    egui_ctx: ResMut<EguiContext>,
    _assets: Res<AssetServer>,
    mut units: Query<&mut Unit>,
    time: Res<Time>,
    mut enemies: Query<&mut Enemy>,
    mut parking_spaces: ResMut<TokenPool<ParkingSpace>>,
) {
    egui::SidePanel::left("side_panel", 200.0).show(egui_ctx.ctx(), |ui| {
        ui.heading("Units");
        for mut unit in units.iter_mut() {
            unit.tick(&time, &mut parking_spaces);
            unit.draw_in_unit_list(ui, &mut parking_spaces);
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
