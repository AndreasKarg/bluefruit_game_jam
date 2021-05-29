use std::{cmp::Ordering, marker::PhantomData, mem::discriminant, sync::Arc};

use engine::{
    bevy::{
        ecs::prelude::Entity,
        prelude::{AssetServer, Commands, Query, Res, ResMut, Time, Timer},
        utils::Duration,
    },
    bevy_egui::{
        egui,
        egui::{Align, Align2, Color32, Grid, Pos2, TextStyle, Ui},
        EguiContext,
    },
};
use rand_derive2::RandGen;
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(RandGen, EnumIter, Display, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatType {
    A,
    B,
    C,
    D,
}

#[derive(Debug, Clone)]
pub enum Unit {
    Mothballed,
    UnMothballing(Timer, Token<ParkingSpace>),
    ParkedUnready(Token<ParkingSpace>),
    ParkedPreparing(Timer, Token<ParkingSpace>, CombatType),
    ParkedReady(Token<ParkingSpace>, CombatType),
    Patrolling(Timer, CombatType),
    Returning(Timer),
    WaitingToPark,
}

impl Unit {
    fn tick(&mut self, time: &Time, parking_spaces: &mut TokenPool<ParkingSpace>) {
        match self {
            Self::ParkedPreparing(timer, parking_space, combat_type) => {
                timer.tick(time.delta());

                if timer.finished() {
                    *self = Self::ParkedReady(parking_space.clone(), *combat_type);
                }
            }
            Self::Patrolling(timer, _) => {
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
                let mut selected_combat_type = None;
                ui.horizontal(|ui| {
                    ui.label("Unready");
                    ui.group(|ui| {
                        ui.label("Preparations");
                        for combat_type in CombatType::iter() {
                            if ui.button(combat_type.to_string()).clicked() {
                                selected_combat_type = Some(combat_type);
                            }
                        }
                    })
                });

                if let Some(combat_type) = selected_combat_type {
                    *self = Self::ParkedPreparing(
                        Timer::from_seconds(5.0, false),
                        parking_space.clone(),
                        combat_type,
                    )
                }
            }
            Unit::ParkedPreparing(timer, _, combat_type) => {
                ui.label(format!(
                    "Preparing combat type {}. {:.0} / {:.1} seconds to go.",
                    combat_type,
                    timer.percent() * 100.0,
                    (timer.duration() - timer.elapsed()).as_secs_f64()
                ));
            }
            Unit::ParkedReady(_, combat_type) => {
                let take_off_clicked = ui.horizontal(|ui| {
                    ui.label(format!("Ready for combat type {}.", combat_type));
                    ui.button("Take off!").clicked()
                });

                if take_off_clicked.inner {
                    *self = Self::Patrolling(Timer::from_seconds(30.0, false), *combat_type);
                }
            }
            Unit::Patrolling(timer, combat_type) => {
                ui.label(format!(
                    "Patrolling combat type {}. Time remaining: {:.1}s",
                    combat_type,
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
            Self::Patrolling(timer, _) => timer.percent(),
            _ => 0.0,
        }
    }

    fn return_to_base(&mut self) {
        if let Self::Patrolling(timer, _) = self {
            *self = Self::Returning(timer.clone());
        } else {
            panic!("Invalid state for returning to base.");
        }
    }
}

pub struct Enemy {
    progress: Timer,
    combat_type: CombatType,
}

impl Enemy {
    fn new(run_time: Duration, combat_type: CombatType) -> Self {
        Self {
            progress: Timer::new(run_time, false),
            combat_type,
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
            "Enemy of type {}! Time left: {:.1}s",
            self.combat_type,
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
        .insert(Enemy::new(Duration::from_secs_f64(20.0), CombatType::A));
}

pub fn units_meet_enemies(
    mut commands: Commands,
    mut units: Query<&mut Unit>,
    mut enemies: Query<(Entity, &mut Enemy)>,
) {
    for (enemy_entity, enemy) in enemies.iter_mut() {
        let suitable_units = units.iter_mut().filter(|unit| {
            matches!(**unit,
                Unit::Patrolling(_, combat_type) if combat_type == enemy.combat_type
            )
        });
        for mut unit in suitable_units {
            if unit.progress_percent() >= enemy.remaining_percent() {
                unit.return_to_base();
                commands.entity(enemy_entity).despawn();
            }
        }
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
            commands.spawn().insert(Enemy::new(
                Duration::from_secs_f64(20.0),
                CombatType::generate_random(),
            ));
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
        ui.separator();

        ui.heading("Your Base");
        ui.separator();
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Mothballed units");
                ui.horizontal(|ui| {
                    ui.label("Unit");
                    ui.button("UnMothball")
                });
                ui.horizontal(|ui| {
                    ui.label("Unit");
                    ui.button("UnMothball")
                });
                ui.horizontal(|ui| {
                    ui.label("Unit");
                    ui.button("UnMothball")
                });
            });
            ui.separator();
            ui.vertical(|ui| {
                ui.heading("Parked units");
                ui.horizontal(|ui| {
                    ui.label("Unit");
                    ui.button("A");
                    ui.button("B");
                    ui.button("C");
                    ui.button("D");
                });
                ui.horizontal(|ui| {
                    ui.label("Unit");
                    ui.button("A");
                    ui.button("B");
                    ui.button("C");
                    ui.button("D");
                });
            });
            ui.separator();
            ui.vertical(|ui| {
                ui.heading("Ready");
                ui.horizontal(|ui| {
                    ui.label("Unit, ready for A");
                    ui.button("Launch")
                });
                ui.horizontal(|ui| {
                    ui.label("Unit, ready for B");
                    ui.button("Launch")
                });
                ui.horizontal(|ui| {
                    ui.label("Unit, ready for C");
                    ui.button("Launch")
                });
            });
        });
        ui.separator();
        ui.separator();

        ui.heading("The Battlezone");
        ui.separator();
        ui.horizontal(|ui| {
            ui.heading("A");
            ui.separator();
            let (response, painter) =
                ui.allocate_painter(ui.available_size_before_wrap_finite(), egui::Sense::hover());
            let rect = response.rect;

            let x = 0.2 * rect.width() + rect.left();
            let y = 0.5 * rect.height() + rect.top();

            let pos = Pos2 { x, y };
            painter.text(
                pos,
                Align2([Align::Center, Align::Center]),
                "▶",
                TextStyle::Heading,
                Color32::GREEN,
            );

            let x = 0.3 * rect.width() + rect.left();

            let pos = Pos2 { x, y };
            painter.text(
                pos,
                Align2([Align::Center, Align::Center]),
                "◀",
                TextStyle::Heading,
                Color32::RED,
            );

            let x = 0.85 * rect.width() + rect.left();

            let pos = Pos2 { x, y };
            painter.text(
                pos,
                Align2([Align::Center, Align::Center]),
                "▶",
                TextStyle::Heading,
                Color32::GOLD,
            );
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.heading("B");
            ui.separator();
            let (response, painter) =
                ui.allocate_painter(ui.available_size_before_wrap_finite(), egui::Sense::hover());
            let rect = response.rect;

            let x = 0.2 * rect.width() + rect.left();
            let y = 0.5 * rect.height() + rect.top();

            let pos = Pos2 { x, y };
            painter.text(
                pos,
                Align2([Align::Center, Align::Center]),
                "▶",
                TextStyle::Heading,
                Color32::GREEN,
            );
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.heading("C");
            ui.separator();
            let (response, painter) =
                ui.allocate_painter(ui.available_size_before_wrap_finite(), egui::Sense::hover());
            let rect = response.rect;

            let x = 0.2 * rect.width() + rect.left();
            let y = 0.5 * rect.height() + rect.top();

            let pos = Pos2 { x, y };
            painter.text(
                pos,
                Align2([Align::Center, Align::Center]),
                "▶",
                TextStyle::Heading,
                Color32::GREEN,
            );
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.heading("D");
            ui.separator();
            let (response, painter) =
                ui.allocate_painter(ui.available_size_before_wrap_finite(), egui::Sense::hover());
            let rect = response.rect;

            let x = 0.2 * rect.width() + rect.left();
            let y = 0.5 * rect.height() + rect.top();

            let pos = Pos2 { x, y };
            painter.text(
                pos,
                Align2([Align::Center, Align::Center]),
                "▶",
                TextStyle::Heading,
                Color32::GREEN,
            );
        });
    });
}
