use std::{cmp::Ordering, marker::PhantomData, mem::discriminant, sync::Arc};

use engine::{
    bevy::{
        core::Stopwatch,
        ecs::prelude::{Entity, Mut},
        prelude::{
            AssetServer, Commands, EventReader, EventWriter, Query, Res, ResMut, Time, Timer,
        },
        utils::Duration,
    },
    bevy_egui::{
        egui,
        egui::{Align, Align2, Color32, Grid, Pos2, TextStyle, Ui},
        EguiContext,
    },
};
use rand::prelude::Distribution;
use rand_derive2::RandGen;
use rand_distr::Normal;
use strum::{Display, EnumIter, IntoEnumIterator};

trait TimerRemaining {
    fn remaining_seconds(&self) -> f32;
}

impl TimerRemaining for Timer {
    fn remaining_seconds(&self) -> f32 {
        (self.duration() - self.elapsed()).as_secs_f32()
    }
}

#[derive(RandGen, EnumIter, Display, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatType {
    A,
    B,
    C,
    D,
}

#[derive(Debug, Clone)]
pub enum Unit {
    InStorage,
    UnStoring(Timer, Token<ParkingSpace>),
    ParkedUnready(Token<ParkingSpace>),
    ParkedPreparing(Timer, Token<ParkingSpace>, CombatType),
    ParkedReady(Token<ParkingSpace>, CombatType),
    Patrolling(Timer, CombatType),
    Returning(Timer, CombatType),
    WaitingToPark,
    Storing(Timer),
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
            Self::Returning(timer, _) => {
                timer.tick(time.delta());

                if timer.finished() {
                    self.try_to_park(parking_spaces);
                }
            }
            Self::UnStoring(timer, parking_space) => {
                timer.tick(time.delta());

                if timer.finished() {
                    *self = Self::ParkedUnready(parking_space.clone());
                }
            }
            Self::WaitingToPark => {
                self.try_to_park(parking_spaces);
            }
            Unit::Storing(timer) => {
                timer.tick(time.delta());

                if timer.finished() {
                    *self = Self::InStorage;
                }
            }
            Unit::InStorage => {}
            Unit::ParkedUnready(_) => {}
            Unit::ParkedReady(_, _) => {}
        }
    }

    fn try_to_park(&mut self, parking_spaces: &mut TokenPool<ParkingSpace>) {
        if let Some(parking_space) = parking_spaces.try_take() {
            *self = Self::ParkedUnready(parking_space);
        } else {
            *self = Self::WaitingToPark;
        }
    }

    fn progress_percent(&self) -> f32 {
        match self {
            Self::Patrolling(timer, _) => timer.percent(),
            _ => 0.0,
        }
    }

    fn return_to_base(&mut self) {
        if let Self::Patrolling(timer, combat_type) = self {
            *self = Self::Returning(timer.clone(), *combat_type);
        } else {
            panic!("Invalid state for returning to base.");
        }
    }

    fn un_store(&mut self, parking_spaces: &mut TokenPool<ParkingSpace>) {
        if let Self::InStorage = self {
            let parking_space = parking_spaces.try_take().unwrap();

            *self = Self::UnStoring(Timer::from_seconds(10.0, false), parking_space);
        } else {
            panic!("Invalid state for unmothballing.")
        }
    }

    fn prepare(&mut self, combat_type: CombatType) {
        if let Self::ParkedUnready(parking_space) = self {
            *self = Self::ParkedPreparing(
                Timer::from_seconds(5.0, false),
                parking_space.clone(),
                combat_type,
            )
        } else {
            panic!("Invalid state for preparing")
        }
    }

    fn take_off(&mut self) {
        if let Self::ParkedReady(parking_space, combat_type) = self {
            *self = Self::Patrolling(Timer::from_seconds(30.0, false), *combat_type);
        } else {
            panic!("Invalid state for taking off")
        }
    }

    fn move_into_storage(&mut self) {
        match self {
            Unit::ParkedUnready(_) => {}
            Unit::ParkedPreparing(_, _, _) => {}
            Unit::ParkedReady(_, _) => {}
            Unit::WaitingToPark => {}
            _ => {
                panic!("Invalid state for moving to storage!")
            }
        }

        *self = Self::Storing(Timer::from_seconds(10.0, false));
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

    fn remaining_percent(&self) -> f32 {
        self.progress.percent_left()
    }
}

pub fn init_stuff(mut commands: Commands) {
    commands.spawn().insert(Unit::InStorage);
    commands.spawn().insert(Unit::InStorage);
    commands.spawn().insert(Unit::InStorage);
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
    time_to_next_spawn: Timer,
    mean_time_between_enemies: Duration,
}

impl Default for EnemySpawner {
    fn default() -> Self {
        let initial_mean_time_between_enemies = Duration::from_secs_f64(20.0);

        let time_to_first_enemy = Self::new_time_to_next_spawn(initial_mean_time_between_enemies);

        Self {
            time_to_next_spawn: Timer::new(time_to_first_enemy, false),
            mean_time_between_enemies: initial_mean_time_between_enemies,
        }
    }
}

impl EnemySpawner {
    fn new_time_to_next_spawn(mean_time_between_enemies: Duration) -> Duration {
        const SPREAD: f64 = 5.0;
        let normal = Normal::new(mean_time_between_enemies.as_secs_f64(), SPREAD).unwrap();

        let seconds_to_next_spawn = normal.sample(&mut rand::thread_rng()).clamp(1.0, 10.0);
        Duration::from_secs_f64(seconds_to_next_spawn)
    }

    fn tick(&mut self, time: &Time, mut commands: Commands) {
        self.time_to_next_spawn.tick(time.delta());

        if self.time_to_next_spawn.finished() {
            commands.spawn().insert(Enemy::new(
                Duration::from_secs_f64(20.0),
                CombatType::generate_random(),
            ));

            self.mean_time_between_enemies = self.mean_time_between_enemies.mul_f64(0.9);
            let time_to_next_spawn = Self::new_time_to_next_spawn(self.mean_time_between_enemies);
            self.time_to_next_spawn.set_duration(time_to_next_spawn);
            self.time_to_next_spawn.reset();
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

    pub fn slots_used(&self) -> usize {
        Arc::strong_count(&self.token_holder) - 1
    }
}

pub struct GameOver {}

pub fn ticker(
    time: Res<Time>,
    mut units: Query<&mut Unit>,
    mut enemies: Query<&mut Enemy>,
    mut parking_spaces: ResMut<TokenPool<ParkingSpace>>,
    mut ev_game_over: EventWriter<GameOver>,
) {
    for mut unit in units.iter_mut() {
        unit.tick(&time, &mut parking_spaces);
    }

    for mut enemy in enemies.iter_mut() {
        enemy.tick(&time);
        if enemy.progress.finished() {
            ev_game_over.send(GameOver {})
        }
    }
}

pub fn gui(
    egui_ctx: ResMut<EguiContext>,
    _assets: Res<AssetServer>,
    mut units: Query<&mut Unit>,
    mut enemies: Query<&mut Enemy>,
    mut parking_spaces: ResMut<TokenPool<ParkingSpace>>,
    mut ev_game_over: EventReader<GameOver>,
    time: Res<Time>,
) {
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
        ui.heading("Stored Units");

        for mut unit in units.iter_mut() {
            match &*unit {
                Unit::InStorage => {
                    ui.horizontal(|ui| {
                        ui.label("Unit");
                        if !parking_spaces.can_take() {
                            ui.set_enabled(false);
                        }

                        if ui.button("Bring out of storage").clicked() {
                            unit.un_store(&mut parking_spaces);
                        }
                    });
                }
                Unit::Storing(timer) => {
                    ui.label(format!(
                        "Moving into Storage. {:.0}% / {:.1} seconds to go.",
                        timer.percent() * 100.0,
                        (timer.duration() - timer.elapsed()).as_secs_f64()
                    ));
                }
                _ => {}
            }
        }
        ui.separator();
        ui.heading(format!(
            "Parking Area ({}/{} spaces used)",
            parking_spaces.slots_used(),
            parking_spaces.max_count
        ));
        for mut unit in units.iter_mut() {
            match &*unit {
                Unit::UnStoring(timer, _) => {
                    ui.label(format!(
                        "Coming out of storage. {:.0}% / {:.1} seconds to go.",
                        timer.percent() * 100.0,
                        (timer.duration() - timer.elapsed()).as_secs_f64()
                    ));
                }
                Unit::ParkedUnready(parking_space) => {
                    let mut selected_combat_type = None;
                    let mut storage_requested = false;
                    ui.horizontal(|ui| {
                        ui.label("Unready");
                        ui.group(|ui| {
                            ui.label("Preparations");
                            for combat_type in CombatType::iter() {
                                if ui.button(combat_type.to_string()).clicked() {
                                    selected_combat_type = Some(combat_type);
                                }
                            }
                            storage_requested = ui.button("Move into storage").clicked();
                        })
                    });

                    if let Some(combat_type) = selected_combat_type {
                        unit.prepare(combat_type);
                    } else if storage_requested {
                        unit.move_into_storage();
                    }
                }
                Unit::ParkedPreparing(timer, _, combat_type) => {
                    ui.label(format!(
                        "Preparing combat type {}. {:.0}% / {:.1} seconds to go.",
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
                        unit.take_off();
                    }
                }
                _ => {}
            }
        }
        ui.separator();
        ui.heading("Queuing for parking");
        for unit in units.iter_mut() {
            match &*unit {
                Unit::WaitingToPark => {
                    ui.label("Unit");
                }
                _ => {}
            }
        }
        ui.separator();
        ui.separator();

        ui.heading("The Battlezone");
        ui.separator();

        for combat_type in CombatType::iter() {
            let enemies = enemies
                .iter_mut()
                .filter(|enemy| enemy.combat_type == combat_type);

            ui.horizontal(|ui| {
                ui.heading(combat_type.to_string());
                ui.separator();
                let (response, painter) = ui
                    .allocate_painter(ui.available_size_before_wrap_finite(), egui::Sense::hover());
                let rect = response.rect;
                let y = 0.5 * rect.height() + rect.top();

                for enemy in enemies {
                    let x = rect.left() + rect.width() * enemy.progress.percent_left();
                    painter.text(
                        Pos2 { x, y },
                        Align2([Align::Min, Align::Center]),
                        format!("◀ {:.1}s", enemy.progress.remaining_seconds()),
                        TextStyle::Heading,
                        Color32::RED,
                    );
                }

                for unit in units.iter_mut() {
                    match &*unit {
                        Unit::Patrolling(progress, unit_combat_type)
                            if *unit_combat_type == combat_type =>
                        {
                            let x = rect.left() + rect.width() * progress.percent();
                            painter.text(
                                Pos2 { x, y },
                                Align2([Align::Max, Align::Center]),
                                "▶",
                                TextStyle::Heading,
                                Color32::GREEN,
                            );
                        }
                        Unit::Returning(progress, unit_combat_type)
                            if *unit_combat_type == combat_type =>
                        {
                            let x = rect.left() + rect.width() * progress.percent();
                            painter.text(
                                Pos2 { x, y },
                                Align2([Align::Max, Align::Center]),
                                "▶",
                                TextStyle::Heading,
                                Color32::GOLD,
                            );
                        }
                        _ => {}
                    }
                }
            });
            ui.separator();
        }
    });

    for _ in ev_game_over.iter() {
        egui::Window::new("Hit!").show(egui_ctx.ctx(), |ui| {
            let survival_duration = time.time_since_startup().as_secs_f64();

            ui.heading("Your base was hit! You are dead !!!!");
            ui.label(format!(
                "You survived for {:.0} seconds though, which is great! Now take a screenshot and brag to your friends about your m4d sk1llz :-D",
                survival_duration
            ));
            if ui.button("Thanks man! This was totally fun!! Let me try this again...").clicked() {
                std::process::exit(0);
            };
        });
    }
}
