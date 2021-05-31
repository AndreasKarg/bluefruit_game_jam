use std::{fmt::Display, marker::PhantomData, sync::Arc};

use engine::{
    bevy::{
        ecs as bevy_ecs,
        ecs::{bundle::Bundle, prelude::Entity},
        prelude::{
            AssetServer, Commands, EventReader, EventWriter, Query, Res, ResMut, Time, Timer,
        },
        reflect::erased_serde::private::serde::__private::Formatter,
        utils::Duration,
    },
    bevy_egui::{
        egui,
        egui::{Align, Align2, Color32, Pos2, TextStyle, Visuals},
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

pub struct Health(f64);

impl Default for Health {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Health {
    fn repair_tick(&mut self, time: &Time) {
        const SECONDS_TO_FULLY_REPAIR: f64 = 15.0;
        self.0 = (self.0 + time.delta_seconds_f64() / SECONDS_TO_FULLY_REPAIR).min(1.0);
    }

    fn take_hit(&mut self, entity: Entity, commands: &mut Commands) {
        self.0 -= 0.25;

        if self.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

impl Display for Health {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}%", self.0 * 100.0)
    }
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
    Parking(Timer, Token<ParkingSpace>),
}

impl Unit {
    fn tick(&mut self, time: &Time) {
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
                    *self = Self::WaitingToPark;
                }
            }
            Self::Returning(timer, _) => {
                timer.tick(time.delta());

                if timer.finished() {
                    *self = Self::WaitingToPark;
                }
            }
            Self::UnStoring(timer, parking_space) => {
                timer.tick(time.delta());

                if timer.finished() {
                    *self = Self::ParkedUnready(parking_space.clone());
                }
            }
            Unit::Storing(timer) => {
                timer.tick(time.delta());

                if timer.finished() {
                    *self = Self::InStorage;
                }
            }
            Self::Parking(timer, parking_space) => {
                timer.tick(time.delta());

                if timer.finished() {
                    *self = Self::ParkedUnready(parking_space.clone());
                }
            }
            Unit::InStorage => {}
            Unit::ParkedUnready(_) => {}
            Unit::ParkedReady(_, _) => {}
            Unit::WaitingToPark => {}
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
        if let Self::ParkedReady(_, combat_type) = self {
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

    fn park_after_returning(&mut self, parking_spaces: &mut TokenPool<ParkingSpace>) {
        if let Self::WaitingToPark = self {
            let parking_space = parking_spaces.try_take().unwrap();

            *self = Self::Parking(Timer::from_seconds(5.0, false), parking_space);
        } else {
            panic!("Invalid state for parking.")
        }
    }
}

#[derive(Bundle)]
pub struct UnitBundle {
    unit: Unit,
    health: Health,
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

    fn remaining_percent(&self) -> f32 {
        self.progress.percent_left()
    }
}

pub fn repair_tick(time: Res<Time>, mut units: Query<(&Unit, &mut Health)>) {
    for (unit, mut health) in units.iter_mut() {
        if matches!(unit, Unit::InStorage) {
            health.repair_tick(&time);
        }
    }
}

pub fn init_stuff(mut commands: Commands) {
    for _ in 0..8 {
        commands.spawn().insert_bundle(UnitBundle {
            unit: Unit::InStorage,
            health: Health::default(),
        });
    }
}

pub fn units_meet_enemies(
    mut commands: Commands,
    mut units: Query<(Entity, &mut Unit, &mut Health)>,
    mut enemies: Query<(Entity, &mut Enemy)>,
) {
    for (enemy_entity, enemy) in enemies.iter_mut() {
        let suitable_units = units.iter_mut().filter(|(_, unit, _)| {
            matches!(**unit,
                Unit::Patrolling(_, combat_type) if combat_type == enemy.combat_type
            )
        });
        for (unit_entity, mut unit, mut health) in suitable_units {
            if unit.progress_percent() >= enemy.remaining_percent() {
                unit.return_to_base();
                commands.entity(enemy_entity).despawn();
                health.take_hit(unit_entity, &mut commands);
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
        let initial_mean_time_between_enemies = Duration::from_secs_f64(30.0);

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
                Duration::from_secs_f64(30.0),
                CombatType::generate_random(),
            ));

            self.mean_time_between_enemies = self.mean_time_between_enemies.mul_f64(0.97);
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
    mut ev_game_over: EventWriter<GameOver>,
) {
    for mut unit in units.iter_mut() {
        unit.tick(&time);
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
    mut units: Query<(&mut Unit, &Health)>,
    mut enemies: Query<&mut Enemy>,
    mut parking_spaces: ResMut<TokenPool<ParkingSpace>>,
    mut ev_game_over: EventReader<GameOver>,
    time: Res<Time>,
) {
    let dark_purple = Color32::from_rgb(77, 53, 77).linear_multiply(0.25);

    let mut visuals = Visuals::dark();

    visuals.extreme_bg_color = dark_purple;
    visuals.widgets.noninteractive.bg_fill = dark_purple;

    egui_ctx.ctx().set_visuals(visuals);

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
        ui.heading(format!(
            "You have survived for {:.0} seconds so far!",
            time.time_since_startup().as_secs_f64()
        ));
        ui.label("Ze evil people from ze Meatropolis wiz zeir Queen on zat island in ze sea are \
            invading our great country of Fruitopia! \
            
            Zey vant to cut down our precious orchards to make ze trees into zeir wretched sawdust sausages!\n\
            Ze Kaiser has ordered YOU to run ze main defense operation agenst ze Meatropolitans. \
            Zey are stronk and REALLY vant zose orchards. Zey vill come faster and faster.\n\
            
            Hold zem off for as long as you can and ve vill propose you for ze Eiserne Pflaume medal!");
        egui::warn_if_debug_build(ui);

        ui.separator();
        ui.separator();

        ui.heading("Your Base");
        ui.separator();
        ui.heading("Stored Units");
        ui.label("Repair damaged units here.");

        for (mut unit, health) in units.iter_mut() {
            match &*unit {
                Unit::InStorage => {
                    ui.horizontal(|ui| {
                        ui.label(format!("Health: {}. Unit", health));
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
                        "Health: {}. Moving into Storage. {:.0}% / {:.1} seconds to go.",
                        health,
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
        ui.label("Prepare your units for battle in one of the lanes and send them off to fight here!");
        for (mut unit, health) in units.iter_mut() {
            match &*unit {
                Unit::UnStoring(timer, _) => {
                    ui.label(format!(
                        "Health: {}. Coming out of storage. {:.0}% / {:.1} seconds to go.",
                        health,
                        timer.percent() * 100.0,
                        (timer.duration() - timer.elapsed()).as_secs_f64()
                    ));
                }
                Unit::Parking(timer, _) => {
                    ui.label(format!(
                        "Health: {}. Parking. {:.0}% / {:.1} seconds to go.",
                        health,
                        timer.percent() * 100.0,
                        (timer.duration() - timer.elapsed()).as_secs_f64()
                    ));
                }
                Unit::ParkedUnready(_) => {
                    let mut selected_combat_type = None;
                    let mut storage_requested = false;
                    ui.horizontal(|ui| {
                        ui.label(format!("Health: {}. Unit not ready. Prepare for... ", health));
                        for combat_type in CombatType::iter() {
                            if ui.button(format!("... {}", combat_type.to_string())).clicked() {
                                selected_combat_type = Some(combat_type);
                            }
                        }
                        storage_requested = ui.button("Move into storage").clicked();
                    });

                    if let Some(combat_type) = selected_combat_type {
                        unit.prepare(combat_type);
                    } else if storage_requested {
                        unit.move_into_storage();
                    }
                }
                Unit::ParkedPreparing(timer, _, combat_type) => {
                    ui.label(format!(
                        "Health: {}. Preparing combat type {}. {:.0}% / {:.1} seconds to go.",
                        health,
                        combat_type,
                        timer.percent() * 100.0,
                        (timer.duration() - timer.elapsed()).as_secs_f64()
                    ));
                }
                Unit::ParkedReady(_, combat_type) => {
                    let take_off_clicked = ui.horizontal(|ui| {
                        ui.label(format!(
                            "Health: {}. Ready for combat type {}.",
                            health, combat_type
                        ));
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
        ui.label("Units here are just standing around when they could be fighting or getting repaired! Move them on as quickly as you can!");
        for (mut unit, health) in units.iter_mut() {
            match &*unit {
                Unit::WaitingToPark => {
                    ui.horizontal(|ui| {
                        ui.label(format!("Health: {}. Unit", health));

                        if ui.button("Move into storage").clicked() {
                            unit.move_into_storage();
                        }

                        if !parking_spaces.can_take() {
                            ui.set_enabled(false);
                        }

                        if ui.button("Park").clicked() {
                            unit.park_after_returning(&mut parking_spaces);
                        }
                    });
                }
                _ => {}
            }
        }
        ui.separator();
        ui.separator();

        ui.heading("The Battlezone");
        ui.label("Enemies (red) approach from the right on different lanes. Prepare your units for the\
        right type of lane and send them off to fight. Each unit (green) can fend off a single enemy before it returns to base (amber).\
        Your units will wear out with use. Remember to repair them!");
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
                        format!("◀ t-{:.1}s", enemy.progress.remaining_seconds()),
                        TextStyle::Heading,
                        Color32::RED,
                    );
                }

                for (unit, health) in units.iter_mut() {
                    match &*unit {
                        Unit::Patrolling(progress, unit_combat_type)
                            if *unit_combat_type == combat_type =>
                        {
                            let x = rect.left() + rect.width() * progress.percent();
                            painter.text(
                                Pos2 { x, y },
                                Align2([Align::Max, Align::Center]),
                                format!("{} ▶", health),
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
                                format!("{} ▶", health),
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
