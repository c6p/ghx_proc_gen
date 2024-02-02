use std::collections::HashSet;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EventWriter,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    input::{keyboard::KeyCode, Input},
    log::{info, warn},
    render::color::Color,
    time::Time,
};
use ghx_proc_gen::{
    generator::{
        model::ModelIndex,
        observer::{GenerationUpdate, QueuedObserver},
        GenerationStatus, Generator,
    },
    grid::{direction::CoordinateSystem, GridDefinition},
    GeneratorError,
};

use crate::grid::markers::{spawn_marker, MarkerDespawnEvent};

use super::{
    spawn_node, AssetSpawner, AssetsBundleSpawner, ComponentSpawner, GenerationControl,
    GenerationControlStatus, ProcGenKeyBindings, SpawnedNode, StepByStepTimed,
};

/// Component used to store model indexes of models with no assets, just to be able to skip their generation when stepping
#[derive(Component, bevy::prelude::Deref)]
pub struct VoidNodes(pub HashSet<ModelIndex>);

/// Simple system that calculates and add a [`VoidNodes`] component for generator entites which don't have one yet.
pub fn insert_void_nodes_to_new_generations<
    C: CoordinateSystem,
    A: AssetsBundleSpawner,
    T: ComponentSpawner,
>(
    mut commands: Commands,
    mut new_generations: Query<
        (Entity, &mut Generator<C>, &AssetSpawner<A, T>),
        Without<VoidNodes>,
    >,
) {
    for (gen_entity, generation, asset_spawner) in new_generations.iter_mut() {
        let mut void_nodes = HashSet::new();
        for model_index in 0..generation.rules().original_models_count() {
            if !asset_spawner.assets.contains_key(&model_index) {
                void_nodes.insert(model_index);
            }
        }
        commands.entity(gen_entity).insert(VoidNodes(void_nodes));
    }
}

/// This system unpauses the [`GenerationControlStatus`] in the [`GenerationControl`] `Resource` on a keypress.
///
/// The keybind is read from the [`ProcGenKeyBindings`] `Resource`
pub fn update_generation_control(
    keys: Res<Input<KeyCode>>,
    proc_gen_key_bindings: Res<ProcGenKeyBindings>,
    mut generation_control: ResMut<GenerationControl>,
) {
    if keys.just_pressed(proc_gen_key_bindings.unpause) {
        match generation_control.status {
            GenerationControlStatus::Paused => {
                generation_control.status = GenerationControlStatus::Ongoing;
            }
            GenerationControlStatus::Ongoing => (),
        }
    }
}

/// This system request the full generation to all [`Generator`] components, if they already are observed through an [`Observed`] component and if the current control status is [`GenerationControlStatus::Ongoing`]
pub fn generate_all<C: CoordinateSystem>(
    mut generation_control: ResMut<GenerationControl>,
    mut observed_generations: Query<&mut Generator<C>, With<QueuedObserver>>,
) {
    for mut generation in observed_generations.iter_mut() {
        if generation_control.status == GenerationControlStatus::Ongoing {
            match generation.generate() {
                Ok(gen_info) => {
                    info!(
                        "Generation done, try_count: {}, seed: {}; grid: {}",
                        gen_info.try_count,
                        generation.seed(),
                        generation.grid()
                    );
                }
                Err(GeneratorError { node_index }) => {
                    warn!(
                        "Generation Failed at node {}, seed: {}; grid: {}",
                        node_index,
                        generation.seed(),
                        generation.grid()
                    );
                }
            }
            generation_control.status = GenerationControlStatus::Paused;
        }
    }
}

/// This system steps all [`Generator`] components if they already are observed through an [`Observed`] component, if the current control status is [`GenerationControlStatus::Ongoing`] and if the appropriate keys are pressed.
///
/// The keybinds are read from the [`ProcGenKeyBindings`] `Resource`
pub fn step_by_step_input_update<C: CoordinateSystem>(
    keys: Res<Input<KeyCode>>,
    proc_gen_key_bindings: Res<ProcGenKeyBindings>,
    mut generation_control: ResMut<GenerationControl>,
    mut observed_generations: Query<(&mut Generator<C>, &VoidNodes), With<QueuedObserver>>,
) {
    if generation_control.status == GenerationControlStatus::Ongoing
        && (keys.just_pressed(proc_gen_key_bindings.step)
            || keys.pressed(proc_gen_key_bindings.continuous_step))
    {
        for (mut generation, void_nodes) in observed_generations.iter_mut() {
            step_generation(&mut generation, void_nodes, &mut generation_control);
        }
    }
}

/// This system steps all [`Generator`] components if they already are observed through an [`Observed`] component, if the current control status is [`GenerationControlStatus::Ongoing`] and if the timer in the [`StepByStepTimed`] `Resource` has finished.
pub fn step_by_step_timed_update<C: CoordinateSystem>(
    mut generation_control: ResMut<GenerationControl>,
    mut steps_and_timer: ResMut<StepByStepTimed>,
    time: Res<Time>,
    mut observed_generations: Query<(&mut Generator<C>, &VoidNodes), With<QueuedObserver>>,
) {
    steps_and_timer.timer.tick(time.delta());
    if steps_and_timer.timer.finished()
        && generation_control.status == GenerationControlStatus::Ongoing
    {
        for (mut generation, void_nodes) in observed_generations.iter_mut() {
            for _ in 0..steps_and_timer.steps_count {
                step_generation(&mut generation, void_nodes, &mut generation_control);
                if generation_control.status != GenerationControlStatus::Ongoing {
                    break;
                }
            }
        }
    }
}

pub fn update_generation_view<C: CoordinateSystem, A: AssetsBundleSpawner, T: ComponentSpawner>(
    mut commands: Commands,
    mut marker_events: EventWriter<MarkerDespawnEvent>,
    mut generators: Query<(
        Entity,
        &GridDefinition<C>,
        &AssetSpawner<A, T>,
        &mut QueuedObserver,
    )>,
    existing_nodes: Query<Entity, With<SpawnedNode>>,
) {
    for (gen_entity, grid, asset_spawner, mut observer) in generators.iter_mut() {
        let mut reinitialized = false;
        let mut nodes_to_spawn = Vec::new();
        for update in observer.dequeue_all() {
            match update {
                GenerationUpdate::Generated(grid_node) => {
                    nodes_to_spawn.push(grid_node);
                }
                GenerationUpdate::Reinitializing(_) => {
                    reinitialized = true;
                    nodes_to_spawn.clear();
                }
                GenerationUpdate::Failed(node_index) => {
                    spawn_marker(
                        &mut commands,
                        gen_entity,
                        Color::RED,
                        grid.pos_from_index(node_index),
                    );
                }
            }
        }

        if reinitialized {
            for existing_node in existing_nodes.iter() {
                commands.entity(existing_node).despawn_recursive();
            }
            marker_events.send(MarkerDespawnEvent::ClearAll);
        }

        for grid_node in nodes_to_spawn {
            spawn_node(
                &mut commands,
                gen_entity,
                &grid,
                asset_spawner,
                &grid_node.model_instance,
                grid_node.node_index,
            );
        }
    }
}

fn step_generation<C: CoordinateSystem>(
    generation: &mut Generator<C>,
    void_nodes: &VoidNodes,
    generation_control: &mut ResMut<GenerationControl>,
) {
    loop {
        let mut non_void_spawned = false;
        match generation.select_and_propagate_collected() {
            Ok((status, nodes_to_spawn)) => {
                for grid_node in nodes_to_spawn {
                    // We still collect the generated nodes here even though we don't really use them to spawn entities. We just check them for void nodes (for visualization purposes)
                    if !void_nodes.contains(&grid_node.model_instance.model_index) {
                        non_void_spawned = true;
                    }
                }
                match status {
                    GenerationStatus::Ongoing => {}
                    GenerationStatus::Done => {
                        info!(
                            "Generation done, seed: {}; grid: {}",
                            generation.seed(),
                            generation.grid()
                        );
                        if generation_control.pause_when_done {
                            generation_control.status = GenerationControlStatus::Paused;
                        }
                        break;
                    }
                }
            }
            Err(GeneratorError { node_index }) => {
                warn!(
                    "Generation Failed at node {}, seed: {}; grid: {}",
                    node_index,
                    generation.seed(),
                    generation.grid()
                );
                if generation_control.pause_on_error {
                    generation_control.status = GenerationControlStatus::Paused;
                }
                break;
            }
        }

        // If we want to skip over void nodes, we eep looping until we spawn a non-void
        if non_void_spawned | !generation_control.skip_void_nodes {
            break;
        }
    }
}
