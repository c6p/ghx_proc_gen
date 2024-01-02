use bevy_examples::AssetDef;
use bevy_ghx_proc_gen::proc_gen::{
    generator::node::{NodeModel, NodeRotation, Socket, SocketCollection, SocketsCartesian3D},
    grid::direction::{Cartesian3D, Direction, GridDelta},
};

const UP_AXIS: Direction = Direction::ZForward;

pub(crate) fn rules_and_assets() -> (
    Vec<Vec<AssetDef>>,
    Vec<NodeModel<Cartesian3D>>,
    SocketCollection,
) {
    let mut sockets = SocketCollection::new();

    let mut s = || -> Socket { sockets.create() };
    let (void, dirt) = (s(), s());
    let (layer_0_down, layer_0_up) = (s(), s());

    let (grass, void_and_grass, grass_and_void) = (s(), s(), s());
    let (layer_1_down, layer_1_up, grass_up) = (s(), s(), s());

    let yellow_grass_down = s();
    let (layer_2_down, layer_2_up) = (s(), s());

    let (water, void_and_water, water_and_void) = (s(), s(), s());
    let (layer_3_down, layer_3_up, ground_up) = (s(), s(), s());

    let (layer_4_down, layer_4_up, props_down) = (s(), s(), s());
    let (big_tree_1_base, big_tree_2_base) = (s(), s());

    let asset = |str| -> Vec<AssetDef> { vec![AssetDef::new(str)] };

    // ---------------------------- Layer 0 ----------------------------

    let mut assets_and_models = vec![(
        asset("dirt"),
        SocketsCartesian3D::Simple {
            x_pos: dirt,
            x_neg: dirt,
            z_pos: layer_0_up,
            z_neg: layer_0_down,
            y_pos: dirt,
            y_neg: dirt,
        }
        .new_model()
        .with_weight(20.),
    )];

    // ---------------------------- Layer 1 ----------------------------

    let green_grass_corner_out = SocketsCartesian3D::Simple {
        x_pos: void_and_grass,
        x_neg: void,
        z_pos: layer_1_up,
        z_neg: layer_1_down,
        y_pos: void,
        y_neg: grass_and_void,
    }
    .new_model();
    let green_grass_corner_in = SocketsCartesian3D::Simple {
        x_pos: grass_and_void,
        x_neg: grass,
        z_pos: layer_1_up,
        z_neg: layer_1_down,
        y_pos: grass,
        y_neg: void_and_grass,
    }
    .new_model();
    let green_grass_side = SocketsCartesian3D::Simple {
        x_pos: void_and_grass,
        x_neg: grass_and_void,
        z_pos: layer_1_up,
        z_neg: layer_1_down,
        y_pos: void,
        y_neg: grass,
    }
    .new_model();

    assets_and_models.extend(vec![
        (
            vec![], // Layer 1 Void
            SocketsCartesian3D::Simple {
                x_pos: void,
                x_neg: void,
                z_pos: layer_1_up,
                z_neg: layer_1_down,
                y_pos: void,
                y_neg: void,
            }
            .new_model(),
        ),
        (
            asset("green_grass"),
            SocketsCartesian3D::Multiple {
                x_pos: vec![grass],
                x_neg: vec![grass],
                z_pos: vec![layer_1_up, grass_up],
                z_neg: vec![layer_1_down],
                y_pos: vec![grass],
                y_neg: vec![grass],
            }
            .new_model()
            .with_weight(5.),
        ),
        // Here, we have different tiles asset for each rotation (grass blades are facing up), so we chose not to specify `with_all_rotations` but instead re-use a model definition by manually create rotatint it and creating different models.
        (
            asset("green_grass_corner_out_tl"),
            green_grass_corner_out.clone(),
        ),
        (
            asset("green_grass_corner_out_bl"),
            green_grass_corner_out.rotated(NodeRotation::Rot90, UP_AXIS),
        ),
        (
            asset("green_grass_corner_out_br"),
            green_grass_corner_out.rotated(NodeRotation::Rot180, UP_AXIS),
        ),
        (
            asset("green_grass_corner_out_tr"),
            green_grass_corner_out.rotated(NodeRotation::Rot270, UP_AXIS),
        ),
        (
            asset("green_grass_corner_in_tl"),
            green_grass_corner_in.clone(),
        ),
        (
            asset("green_grass_corner_in_bl"),
            green_grass_corner_in.rotated(NodeRotation::Rot90, UP_AXIS),
        ),
        (
            asset("green_grass_corner_in_br"),
            green_grass_corner_in.rotated(NodeRotation::Rot180, UP_AXIS),
        ),
        (
            asset("green_grass_corner_in_tr"),
            green_grass_corner_in.rotated(NodeRotation::Rot270, UP_AXIS),
        ),
        (asset("green_grass_side_t"), green_grass_side.clone()),
        (
            asset("green_grass_side_l"),
            green_grass_side.rotated(NodeRotation::Rot90, UP_AXIS),
        ),
        (
            asset("green_grass_side_b"),
            green_grass_side.rotated(NodeRotation::Rot180, UP_AXIS),
        ),
        (
            asset("green_grass_side_r"),
            green_grass_side.rotated(NodeRotation::Rot270, UP_AXIS),
        ),
    ]);

    // ---------------------------- Layer 2 ----------------------------

    let yellow_grass_corner_out = SocketsCartesian3D::Simple {
        x_pos: void_and_grass,
        x_neg: void,
        z_pos: layer_2_up,
        z_neg: yellow_grass_down,
        y_pos: void,
        y_neg: grass_and_void,
    }
    .new_model();
    let yellow_grass_corner_in = SocketsCartesian3D::Simple {
        x_pos: grass_and_void,
        x_neg: grass,
        z_pos: layer_2_up,
        z_neg: yellow_grass_down,
        y_pos: grass,
        y_neg: void_and_grass,
    }
    .new_model();
    let yellow_grass_side = SocketsCartesian3D::Simple {
        x_pos: void_and_grass,
        x_neg: grass_and_void,
        z_pos: layer_2_up,
        z_neg: yellow_grass_down,
        y_pos: void,
        y_neg: grass,
    }
    .new_model();

    assets_and_models.extend(vec![
        (
            vec![], // Layer 2 Void
            SocketsCartesian3D::Simple {
                x_pos: void,
                x_neg: void,
                z_pos: layer_2_up,
                z_neg: layer_2_down,
                y_pos: void,
                y_neg: void,
            }
            .new_model(),
        ),
        (
            asset("yellow_grass"),
            SocketsCartesian3D::Simple {
                x_pos: grass,
                x_neg: grass,
                z_pos: layer_2_up,
                z_neg: layer_2_down,
                y_pos: grass,
                y_neg: grass,
            }
            .new_model()
            .with_weight(1.),
        ),
        (
            asset("yellow_grass_corner_out_tl"),
            yellow_grass_corner_out.clone(),
        ),
        (
            asset("yellow_grass_corner_out_bl"),
            yellow_grass_corner_out.rotated(NodeRotation::Rot90, UP_AXIS),
        ),
        (
            asset("yellow_grass_corner_out_br"),
            yellow_grass_corner_out.rotated(NodeRotation::Rot180, UP_AXIS),
        ),
        (
            asset("yellow_grass_corner_out_tr"),
            yellow_grass_corner_out.rotated(NodeRotation::Rot270, UP_AXIS),
        ),
        (
            asset("yellow_grass_corner_in_tl"),
            yellow_grass_corner_in.clone(),
        ),
        (
            asset("yellow_grass_corner_in_bl"),
            yellow_grass_corner_in.rotated(NodeRotation::Rot90, UP_AXIS),
        ),
        (
            asset("yellow_grass_corner_in_br"),
            yellow_grass_corner_in.rotated(NodeRotation::Rot180, UP_AXIS),
        ),
        (
            asset("yellow_grass_corner_in_tr"),
            yellow_grass_corner_in.rotated(NodeRotation::Rot270, UP_AXIS),
        ),
        (asset("yellow_grass_side_t"), yellow_grass_side.clone()),
        (
            asset("yellow_grass_side_l"),
            yellow_grass_side.rotated(NodeRotation::Rot90, UP_AXIS),
        ),
        (
            asset("yellow_grass_side_b"),
            yellow_grass_side.rotated(NodeRotation::Rot180, UP_AXIS),
        ),
        (
            asset("yellow_grass_side_r"),
            yellow_grass_side.rotated(NodeRotation::Rot270, UP_AXIS),
        ),
    ]);

    // ---------------------------- Layer 3 ----------------------------

    const WATER_WEIGHT: f32 = 0.02;
    let water_corner_out = SocketsCartesian3D::Simple {
        x_pos: void_and_water,
        x_neg: void,
        z_pos: layer_3_up,
        z_neg: layer_3_down,
        y_pos: void,
        y_neg: water_and_void,
    }
    .new_model()
    .with_weight(WATER_WEIGHT);
    let water_corner_in = SocketsCartesian3D::Simple {
        x_pos: water_and_void,
        x_neg: water,
        z_pos: layer_3_up,
        z_neg: layer_3_down,
        y_pos: water,
        y_neg: void_and_water,
    }
    .new_model()
    .with_weight(WATER_WEIGHT);
    let water_side = SocketsCartesian3D::Simple {
        x_pos: void_and_water,
        x_neg: water_and_void,
        z_pos: layer_3_up,
        z_neg: layer_3_down,
        y_pos: void,
        y_neg: water,
    }
    .new_model()
    .with_weight(WATER_WEIGHT);

    assets_and_models.extend(vec![
        (
            vec![], // Layer 3 Void
            SocketsCartesian3D::Multiple {
                x_pos: vec![void],
                x_neg: vec![void],
                z_pos: vec![layer_3_up, ground_up],
                z_neg: vec![layer_3_down],
                y_pos: vec![void],
                y_neg: vec![void],
            }
            .new_model(),
        ),
        (
            asset("water"),
            SocketsCartesian3D::Simple {
                x_pos: water,
                x_neg: water,
                z_pos: layer_3_up,
                z_neg: layer_3_down,
                y_pos: water,
                y_neg: water,
            }
            .new_model()
            .with_weight(10. * WATER_WEIGHT),
        ),
        (asset("water_corner_out_tl"), water_corner_out.clone()),
        (
            asset("water_corner_out_bl"),
            water_corner_out.rotated(NodeRotation::Rot90, UP_AXIS),
        ),
        (
            asset("water_corner_out_br"),
            water_corner_out.rotated(NodeRotation::Rot180, UP_AXIS),
        ),
        (
            asset("water_corner_out_tr"),
            water_corner_out.rotated(NodeRotation::Rot270, UP_AXIS),
        ),
        (asset("water_corner_in_tl"), water_corner_in.clone()),
        (
            asset("water_corner_in_bl"),
            water_corner_in.rotated(NodeRotation::Rot90, UP_AXIS),
        ),
        (
            asset("water_corner_in_br"),
            water_corner_in.rotated(NodeRotation::Rot180, UP_AXIS),
        ),
        (
            asset("water_corner_in_tr"),
            water_corner_in.rotated(NodeRotation::Rot270, UP_AXIS),
        ),
        (asset("water_side_t"), water_side.clone()),
        (
            asset("water_side_l"),
            water_side.rotated(NodeRotation::Rot90, UP_AXIS),
        ),
        (
            asset("water_side_b"),
            water_side.rotated(NodeRotation::Rot180, UP_AXIS),
        ),
        (
            asset("water_side_r"),
            water_side.rotated(NodeRotation::Rot270, UP_AXIS),
        ),
    ]);

    // ---------------------------- Layer 4 ----------------------------

    const PROPS_WEIGHT: f32 = 0.025;
    const ROCKS_WEIGHT: f32 = 0.008;
    const PLANTS_WEIGHT: f32 = 0.025;
    const STUMPS_WEIGHT: f32 = 0.012;
    let prop = SocketsCartesian3D::Simple {
        x_pos: void,
        x_neg: void,
        z_pos: layer_4_up,
        z_neg: props_down,
        y_pos: void,
        y_neg: void,
    }
    .new_model()
    .with_weight(PROPS_WEIGHT);
    let plant_prop = prop.clone().with_weight(PLANTS_WEIGHT);
    let stump_prop = prop.clone().with_weight(STUMPS_WEIGHT);
    let rock_prop = prop.clone().with_weight(ROCKS_WEIGHT);

    assets_and_models.extend(vec![
        (
            vec![], // Layer 4 Void
            SocketsCartesian3D::Multiple {
                x_pos: vec![void],
                x_neg: vec![void],
                z_pos: vec![layer_4_up],
                z_neg: vec![layer_4_down],
                y_pos: vec![void],
                y_neg: vec![void],
            }
            .new_model(),
        ),
        (
            vec![
                AssetDef::new("small_tree_bottom"),
                AssetDef::new("small_tree_top").with_offset(GridDelta::new(0, 1, 0)),
            ],
            plant_prop.clone(),
        ),
        (
            vec![
                AssetDef::new("big_tree_1_bl"),
                AssetDef::new("big_tree_1_tl").with_offset(GridDelta::new(0, 1, 0)),
            ],
            SocketsCartesian3D::Simple {
                x_pos: big_tree_1_base,
                x_neg: void,
                z_pos: layer_4_up,
                z_neg: props_down,
                y_pos: void,
                y_neg: void,
            }
            .new_model()
            .with_weight(PROPS_WEIGHT),
        ),
        (
            vec![
                AssetDef::new("big_tree_1_br"),
                AssetDef::new("big_tree_1_tr").with_offset(GridDelta::new(0, 1, 0)),
            ],
            SocketsCartesian3D::Simple {
                x_pos: void,
                x_neg: big_tree_1_base,
                z_pos: layer_4_up,
                z_neg: props_down,
                y_pos: void,
                y_neg: void,
            }
            .new_model()
            .with_weight(PROPS_WEIGHT),
        ),
        (
            vec![
                AssetDef::new("big_tree_2_bl"),
                AssetDef::new("big_tree_2_tl").with_offset(GridDelta::new(0, 1, 0)),
            ],
            SocketsCartesian3D::Simple {
                x_pos: big_tree_2_base,
                x_neg: void,
                z_pos: layer_4_up,
                z_neg: props_down,
                y_pos: void,
                y_neg: void,
            }
            .new_model()
            .with_weight(PROPS_WEIGHT),
        ),
        (
            vec![
                AssetDef::new("big_tree_2_br"),
                AssetDef::new("big_tree_2_tr").with_offset(GridDelta::new(0, 1, 0)),
            ],
            SocketsCartesian3D::Simple {
                x_pos: void,
                x_neg: big_tree_2_base,
                z_pos: layer_4_up,
                z_neg: props_down,
                y_pos: void,
                y_neg: void,
            }
            .new_model()
            .with_weight(PROPS_WEIGHT),
        ),
        (asset("tree_stump_1"), stump_prop.clone()),
        (asset("tree_stump_2"), stump_prop.clone()),
        (asset("tree_stump_3"), stump_prop.clone()),
        (asset("rock_1"), rock_prop.clone()),
        (asset("rock_2"), rock_prop.clone()),
        (asset("rock_3"), rock_prop.clone()),
        (asset("rock_4"), rock_prop.clone()),
        (asset("plant_1"), plant_prop.clone()),
        (asset("plant_2"), plant_prop.clone()),
        (asset("plant_3"), plant_prop.clone()),
        (asset("plant_4"), plant_prop.clone()),
    ]);

    sockets
        .add_connections(vec![
            (dirt, vec![dirt]),
            (void, vec![void]),
            (grass, vec![grass]),
            (void_and_grass, vec![grass_and_void]),
            (water, vec![water]),
            (water_and_void, vec![void_and_water]),
            (big_tree_1_base, vec![big_tree_1_base]),
            (big_tree_2_base, vec![big_tree_2_base]),
        ])
        .add_rotated_connection(layer_0_up, vec![layer_1_down])
        .add_rotated_connection(layer_1_up, vec![layer_2_down])
        .add_rotated_connection(layer_2_up, vec![layer_3_down])
        .add_rotated_connection(layer_3_up, vec![layer_4_down])
        .add_rotated_connection(yellow_grass_down, vec![grass_up])
        .add_rotated_connection(props_down, vec![ground_up]);

    (
        // Assets
        assets_and_models.iter().map(|t| t.0.clone()).collect(),
        // Node models
        assets_and_models
            .iter()
            .map(|t| {
                t.1.clone()
                    .with_name(t.0.first().unwrap_or(&AssetDef::new("void")).path())
            })
            .collect(),
        sockets,
    )
}
