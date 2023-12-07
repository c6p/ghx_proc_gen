use std::{f32::consts::PI, time::Duration};

use bevy::{log::LogPlugin, pbr::DirectionalLightShadowMap, prelude::*, utils::HashMap};

use bevy_ghx_proc_gen::{
    grid::{spawn_debug_grids, DebugGridView, DebugGridViewConfig, Grid},
    lines::LineMaterial,
    proc_gen::{
        generator::{
            builder::GeneratorBuilder,
            node::GeneratedNode,
            observer::{GenerationUpdate, QueuedObserver},
            rules::RulesBuilder,
            GenerationStatus, Generator, ModelSelectionHeuristic, NodeSelectionHeuristic, RngMode,
        },
        grid::{direction::Cartesian3D, GridDefinition},
    },
};
use bevy_ghx_utilities::camera::{pan_orbit_camera, PanOrbitCamera};

use crate::rules::rules_and_assets;

mod rules;

#[derive(PartialEq, Eq)]
pub enum GenerationViewMode {
    StepByStep(u64),
    StepByStepPaused,
    Final,
}

/// Change this value to change the way the generation is visualized
const GENERATION_VIEW_MODE: GenerationViewMode = GenerationViewMode::StepByStepPaused;

#[derive(Resource)]
struct Generation {
    models_assets: HashMap<usize, Handle<Scene>>,
    gen: Generator<Cartesian3D>,
    observer: QueuedObserver,
}

#[derive(Resource)]
struct GenerationTimer(Timer);

/// Size of a block in world units
const NODE_SIZE: f32 = 1.;
const HALF_NODE_SIZE: f32 = NODE_SIZE / 2.;
const NODE_SCALE: Vec3 = Vec3::new(NODE_SIZE, NODE_SIZE, NODE_SIZE);

const ASSETS_SCALE_FACTOR: f32 = NODE_SIZE / 40.; // Models are 40 voxels wide
const ASSETS_SCALE: Vec3 = Vec3::new(
    ASSETS_SCALE_FACTOR,
    ASSETS_SCALE_FACTOR,
    ASSETS_SCALE_FACTOR,
);

fn setup_scene(mut commands: Commands) {
    // Camera
    let camera_position = Vec3::new(-2.5, 4.5, 9.0);
    let radius = camera_position.length();
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(camera_position).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera {
            radius,
            ..Default::default()
        },
    ));
    // Scene lights
    commands.insert_resource(AmbientLight {
        color: Color::SEA_GREEN,
        brightness: 0.1,
    });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.,
            // color: Color::SEA_GREEN,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(5.0, 10.0, 2.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });
}

fn setup_generator(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load rules
    let (models_asset_paths, models, sockets_connections) = rules_and_assets();

    // Create generator
    let rules = RulesBuilder::new_cartesian_3d(models, sockets_connections)
        .build()
        .unwrap();
    let grid = GridDefinition::new_cartesian_3d(20, 4, 20, false);
    let mut generator = GeneratorBuilder::new()
        .with_rules(rules)
        .with_grid(grid.clone())
        .with_max_retry_count(250)
        .with_rng(RngMode::RandomSeed)
        .with_node_heuristic(NodeSelectionHeuristic::MinimumRemainingValue)
        .with_model_heuristic(ModelSelectionHeuristic::WeightedProbability)
        .build();
    let observer = QueuedObserver::new(&mut generator);
    info!("Seed: {}", generator.get_seed());

    // Load assets
    let mut models_assets = HashMap::new();
    for (index, path) in models_asset_paths.iter().enumerate() {
        if let Some(path) = path {
            models_assets.insert(
                index,
                asset_server.load(format!("3d_terrain/{path}.glb#Scene0")),
            );
        }
    }

    match GENERATION_VIEW_MODE {
        GenerationViewMode::StepByStepPaused => (),
        GenerationViewMode::StepByStep(interval) => commands.insert_resource(GenerationTimer(
            Timer::new(Duration::from_millis(interval), TimerMode::Repeating),
        )),
        GenerationViewMode::Final => {
            let output = generator.generate().unwrap();
            for (node_index, node) in output.nodes().iter().enumerate() {
                spawn_node(
                    &mut commands,
                    &models_assets,
                    generator.grid(),
                    node,
                    node_index,
                );
            }
        }
    }

    commands.insert_resource(Generation {
        models_assets,
        gen: generator,
        observer,
    });

    commands.spawn((
        SpatialBundle::from_transform(Transform::from_translation(Vec3 {
            x: -(grid.size_x() as f32) / 2.,
            y: 0.,
            z: grid.size_z() as f32 / 2.,
        })),
        Grid { def: grid },
        DebugGridViewConfig {
            node_size: NODE_SCALE,
            color: Color::GRAY.with_a(0.),
        },
    ));
}

#[derive(Component)]
struct SpawnedNode;

fn spawn_node(
    commands: &mut Commands,
    models_assets: &HashMap<usize, Handle<Scene>>,
    grid: &GridDefinition<Cartesian3D>,
    node: &GeneratedNode,
    node_index: usize,
) {
    if let Some(asset) = models_assets.get(&node.model_index) {
        let x_offset = grid.size_x() as f32 / 2.;
        let z_offset = grid.size_z() as f32 / 2.;
        let pos = grid.get_position(node_index);
        commands.spawn((
            SceneBundle {
                scene: asset.clone(),
                transform: Transform::from_xyz(
                    (pos.x as f32) - x_offset + HALF_NODE_SIZE,
                    pos.y as f32,
                    (pos.z as f32) - z_offset + HALF_NODE_SIZE,
                )
                .with_scale(ASSETS_SCALE)
                .with_rotation(Quat::from_rotation_y(f32::to_radians(
                    node.rotation.value() as f32,
                ))),
                ..default()
            },
            SpawnedNode,
        ));
    }
}

fn select_and_propagate(
    commands: &mut Commands,
    generation: &mut ResMut<Generation>,
    nodes: Query<Entity, With<SpawnedNode>>,
) {
    match generation.gen.select_and_propagate() {
        Ok(status) => match status {
            GenerationStatus::Ongoing => (),
            GenerationStatus::Done => info!("Generation done"),
        },
        Err(_) => {
            info!("Generation Failed")
        }
    }
    // Process the observer queue even if generation failed
    let updates = generation.observer.dequeue_all();
    // Buffer the nodes to spawn in case a generation failure invalidate some.
    let mut nodes_to_spawn = vec![];
    let mut despawn_nodes = false;
    for update in updates {
        match update {
            GenerationUpdate::Generated {
                node_index,
                generated_node,
            } => {
                nodes_to_spawn.push((node_index, generated_node));
            }
            GenerationUpdate::Reinitialized | GenerationUpdate::Failed => {
                nodes_to_spawn.clear();
                despawn_nodes = true;
            }
        }
    }
    if despawn_nodes {
        for entity in nodes.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }

    for (node_index, generated_node) in nodes_to_spawn {
        info!("Spawning {:?} at node index {}", generated_node, node_index);
        spawn_node(
            commands,
            &generation.models_assets,
            generation.gen.grid(),
            &generated_node,
            node_index,
        );
    }
}

fn step_by_step_input_update(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    mut generation: ResMut<Generation>,
    nodes: Query<Entity, With<SpawnedNode>>,
) {
    if keys.pressed(KeyCode::Space) || buttons.just_pressed(MouseButton::Left) {
        select_and_propagate(&mut commands, &mut generation, nodes);
    }
}

fn step_by_step_timed_update(
    mut commands: Commands,
    mut generation: ResMut<Generation>,
    mut timer: ResMut<GenerationTimer>,
    time: Res<Time>,
    nodes: Query<Entity, With<SpawnedNode>>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        select_and_propagate(&mut commands, &mut generation, nodes);
    }
}

fn toggle_debug_grid_visibility(
    keys: Res<Input<KeyCode>>,
    mut debug_grids: Query<&mut Visibility, With<DebugGridView>>,
) {
    if keys.just_pressed(KeyCode::F1) {
        for mut view_visibility in debug_grids.iter_mut() {
            *view_visibility = match *view_visibility {
                Visibility::Inherited => Visibility::Hidden,
                Visibility::Hidden => Visibility::Visible,
                Visibility::Visible => Visibility::Hidden,
            }
        }
    }
}

fn main() {
    let mut app = App::new();
    app.insert_resource(DirectionalLightShadowMap { size: 4096 });
    app.add_plugins((
        DefaultPlugins.set(LogPlugin {
            filter: "info,wgpu_core=warn,wgpu_hal=warn,ghx_proc_gen=trace".into(),
            level: bevy::log::Level::DEBUG,
        }),
        MaterialPlugin::<LineMaterial>::default(),
    ));
    app.add_systems(Startup, (setup_generator, setup_scene))
        .add_systems(Update, pan_orbit_camera)
        .add_systems(Update, spawn_debug_grids::<Cartesian3D>)
        .add_systems(Update, toggle_debug_grid_visibility);

    match GENERATION_VIEW_MODE {
        GenerationViewMode::StepByStep(_) => {
            app.add_systems(Update, step_by_step_timed_update);
        }
        GenerationViewMode::StepByStepPaused => {
            app.add_systems(Update, step_by_step_input_update);
        }
        GenerationViewMode::Final => (),
    };

    app.run();
}
