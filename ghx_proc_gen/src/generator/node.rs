use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

#[cfg(feature = "debug-traces")]
use core::fmt;

use crate::grid::direction::{Cartesian2D, Cartesian3D, Direction, DirectionSet};

use super::rules::CARTESIAN_2D_ROTATION_AXIS;

/// Index of a model
pub type ModelIndex = usize;

pub(crate) fn expand_models<T: DirectionSet>(
    models: Vec<NodeModel<T>>,
    rotation_axis: Direction,
) -> Vec<ExpandedNodeModel> {
    let mut expanded_models = Vec::new();
    for (index, model) in models.iter().enumerate() {
        // Iterate on a vec of all possible node rotations and filter with the set to have a deterministic insertion order of expanded nodes.
        for rotation in ALL_NODE_ROTATIONS {
            if model.allowed_rotations.contains(&rotation) {
                let rotated_sockets = model.rotated_sockets(*rotation, rotation_axis);
                expanded_models.push(ExpandedNodeModel {
                    sockets: rotated_sockets
                        .iter()
                        .map(|dir| dir.iter().map(|s| s.id()).collect())
                        .collect(),
                    weight: model.weight,
                    original_index: index,
                    rotation: *rotation,
                    #[cfg(feature = "debug-traces")]
                    name: model.name,
                });
            }
        }
    }
    expanded_models
}

/// Id of a possible connection type
pub(crate) type SocketId = u64;

#[derive(Clone)]
pub struct SocketCollection {
    incremental_socket_index: u32,

    /// For uniqueness
    uniques: HashMap<SocketId, HashSet<SocketId>>,
    /// For determinism and sequential access
    compatibles: HashMap<SocketId, Vec<SocketId>>,
}

impl SocketCollection {
    pub fn new() -> Self {
        Self {
            incremental_socket_index: 0,
            uniques: HashMap::new(),
            compatibles: HashMap::new(),
        }
    }

    pub fn create(&mut self) -> Socket {
        let socket = Socket::new(self.incremental_socket_index);
        self.incremental_socket_index += 1;
        socket
    }

    pub fn add_connection<I>(&mut self, from: Socket, to: I) -> &mut Self
    where
        I: IntoIterator<Item = Socket>,
    {
        for to_socket in to.into_iter() {
            self.register_connection(&from, &to_socket);
        }
        self
    }

    pub fn add_connections<I, J>(&mut self, connections: I) -> &mut Self
    where
        I: IntoIterator<Item = (Socket, J)>,
        J: IntoIterator<Item = Socket>,
    {
        for (from, to_sockets) in connections.into_iter() {
            for to in to_sockets.into_iter() {
                self.register_connection(&from, &to);
            }
        }
        self
    }

    pub fn add_rotated_connection(&mut self, from: Socket, to: Vec<Socket>) -> &mut Self {
        for to_rotation in ALL_NODE_ROTATIONS {
            let to_rotated_sockets: Vec<Socket> =
                to.iter().map(|s| s.rotated(*to_rotation)).collect();
            for from_rot in ALL_NODE_ROTATIONS {
                let rotated_socket = from.rotated(*from_rot);
                for to_socket in to_rotated_sockets.iter() {
                    self.register_connection(&rotated_socket, &to_socket);
                }
            }
        }
        self
    }

    /// Modifies `delta_rotations`
    pub fn add_constrained_rotated_connection(
        &mut self,
        from: Socket,
        mut delta_rotations: Vec<NodeRotation>,
        to: Vec<Socket>,
    ) -> &mut Self {
        for to_rotation in ALL_NODE_ROTATIONS {
            let to_rotated_sockets: Vec<Socket> =
                to.iter().map(|s| s.rotated(*to_rotation)).collect();
            for from_rotation in delta_rotations.iter_mut() {
                let from_rotated_socket = from.rotated(*from_rotation);
                for to_socket in to_rotated_sockets.iter() {
                    self.register_connection(&from_rotated_socket, &to_socket);
                }
                *from_rotation = from_rotation.next();
            }
        }
        self
    }

    fn register_connection_half(&mut self, from: &Socket, to: &Socket) {
        // TODO Decide if we check for existence
        let connectable_sockets = self.uniques.entry(from.id()).or_insert(HashSet::new());

        if connectable_sockets.insert(to.id()) {
            self.compatibles
                .entry(from.id())
                .or_insert(Vec::new())
                .push(to.id());
        }
    }

    fn register_connection(&mut self, from: &Socket, to: &Socket) {
        self.register_connection_half(from, to);
        self.register_connection_half(to, from);
    }

    pub(crate) fn get_compatibles(&self, socket: SocketId) -> Option<&Vec<SocketId>> {
        self.compatibles.get(&socket)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.incremental_socket_index == 0
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Socket {
    socket_index: u32,
    rot: NodeRotation,
}

impl Socket {
    pub(crate) fn new(socket_index: u32) -> Self {
        Self {
            socket_index,
            rot: NodeRotation::Rot0,
        }
    }

    pub(crate) fn id(&self) -> SocketId {
        self.socket_index as u64 + ((self.rot.index() as u64) << 32)
    }

    pub(crate) fn rotated(&self, rotation: NodeRotation) -> Socket {
        let mut rotated_socket = self.clone();
        rotated_socket.rot = rotated_socket.rot.rotated(rotation);
        rotated_socket
    }

    pub(crate) fn rotate(&mut self, rotation: NodeRotation) {
        self.rot.rotate(rotation);
    }
}

/// Represents a model to be used by a [`crate::generator::Generator`] as a "building-block" to fill out the generated area.
#[derive(Clone)]
pub struct NodeModel<T: DirectionSet> {
    /// Allowed connections for this [`NodeModel`] in the output.
    sockets: Vec<Vec<Socket>>,
    /// Weight factor influencing the density of this [`NodeModel`] in the generated output.
    ///
    ///  Defaults to 1.0
    weight: f32,
    /// Allowed rotations of this [`NodeModel`] in the output, around the rotation axis specified in the rules.
    ///
    /// Defaults to only [`NodeRotation::Rot0`].
    ///
    /// Notes:
    /// - In 3d, top and bottom sockets of a model are rotated into new sockets when the model itself is rotated.
    /// - In 2d, the rotation axis cannot be modified and is set to [`Direction::ZForward`].
    allowed_rotations: HashSet<NodeRotation>,

    /// Name given to this model for debug purposes.
    name: Option<&'static str>,

    typestate: PhantomData<T>,
}

/// Sockets for a model to be used in a 2d cartesian grid.
pub enum SocketsCartesian2D {
    /// The model has only 1 socket, and its is the same in all directions.
    Mono(Socket),
    /// The model has 1 socket per side.
    Simple {
        x_pos: Socket,
        x_neg: Socket,
        y_pos: Socket,
        y_neg: Socket,
    },
    /// The model has multiple sockets per side.
    Multiple {
        x_pos: Vec<Socket>,
        x_neg: Vec<Socket>,
        y_pos: Vec<Socket>,
        y_neg: Vec<Socket>,
    },
}

impl Into<Vec<Vec<Socket>>> for SocketsCartesian2D {
    fn into(self) -> Vec<Vec<Socket>> {
        match self {
            SocketsCartesian2D::Mono(socket) => vec![vec![socket]; 4],
            SocketsCartesian2D::Simple {
                x_pos,
                y_pos,
                x_neg,
                y_neg,
            } => {
                vec![vec![x_pos], vec![y_pos], vec![x_neg], vec![y_neg]]
            }
            SocketsCartesian2D::Multiple {
                x_pos,
                y_pos,
                x_neg,
                y_neg,
            } => {
                vec![x_pos, y_pos, x_neg, y_neg]
            }
        }
    }
}

impl SocketsCartesian2D {
    /// Create a [`NodeModel`] from its sockets definition, with default values for the other members.
    pub fn new_model(self) -> NodeModel<Cartesian2D> {
        NodeModel::new_cartesian_2d(self)
    }
}

impl NodeModel<Cartesian2D> {
    /// Create a [`NodeModel`] from a [`SocketsCartesian2D`] definition, with default values for the other members.
    pub fn new_cartesian_2d(sockets: SocketsCartesian2D) -> NodeModel<Cartesian2D> {
        Self {
            sockets: sockets.into(),
            allowed_rotations: HashSet::from([NodeRotation::Rot0]),
            weight: 1.0,
            name: None,
            typestate: PhantomData,
        }
    }

    /// Returns a clone of the [`NodeModel`] with its sockets rotated by `rotation` around [`CARTESIAN_2D_ROTATION_AXIS`].
    pub fn rotated(&self, rotation: NodeRotation) -> Self {
        Self {
            sockets: self.rotated_sockets(rotation, CARTESIAN_2D_ROTATION_AXIS),
            weight: self.weight,
            allowed_rotations: self.allowed_rotations.clone(),
            name: self.name.clone(),
            typestate: PhantomData,
        }
    }
}

/// Sockets for a model to be used in a 3d cartesian grid.
pub enum SocketsCartesian3D {
    /// The model has only 1 socket, and its is the same in all directions.
    Mono(Socket),
    /// The model has 1 socket per side.
    Simple {
        x_pos: Socket,
        x_neg: Socket,
        z_pos: Socket,
        z_neg: Socket,
        y_pos: Socket,
        y_neg: Socket,
    },
    /// The model has multiple sockets per side.
    Multiple {
        x_pos: Vec<Socket>,
        x_neg: Vec<Socket>,
        z_pos: Vec<Socket>,
        z_neg: Vec<Socket>,
        y_pos: Vec<Socket>,
        y_neg: Vec<Socket>,
    },
}

impl Into<Vec<Vec<Socket>>> for SocketsCartesian3D {
    fn into(self) -> Vec<Vec<Socket>> {
        match self {
            SocketsCartesian3D::Mono(socket) => vec![vec![socket]; 6],
            SocketsCartesian3D::Simple {
                x_pos,
                y_pos,
                x_neg,
                y_neg,
                z_pos,
                z_neg,
            } => {
                vec![
                    vec![x_pos],
                    vec![y_pos],
                    vec![x_neg],
                    vec![y_neg],
                    vec![z_pos],
                    vec![z_neg],
                ]
            }
            SocketsCartesian3D::Multiple {
                x_pos,
                y_pos,
                x_neg,
                y_neg,
                z_pos,
                z_neg,
            } => {
                vec![x_pos, y_pos, x_neg, y_neg, z_pos, z_neg]
            }
        }
    }
}

impl SocketsCartesian3D {
    /// Create a [`NodeModel`] from its sockets definition, with default values for the other members: weight is 1.0 and the model will not be rotated.
    pub fn new_model(self) -> NodeModel<Cartesian3D> {
        NodeModel::new_cartesian_3d(self)
    }
}

impl NodeModel<Cartesian3D> {
    /// Create a [`NodeModel`] from a [`SocketsCartesian3D`] definition, with default values for the other members: weight is 1.0 and the model will not be rotated.
    pub fn new_cartesian_3d(sockets: SocketsCartesian3D) -> NodeModel<Cartesian3D> {
        Self {
            sockets: sockets.into(),
            allowed_rotations: HashSet::from([NodeRotation::Rot0]),
            weight: 1.0,
            name: None,
            typestate: PhantomData,
        }
    }

    /// Returns a clone of the [`NodeModel`] with its sockets rotated by `rotation` around `axis`.
    pub fn rotated(&self, rotation: NodeRotation, axis: Direction) -> Self {
        Self {
            sockets: self.rotated_sockets(rotation, axis),
            weight: self.weight,
            allowed_rotations: self.allowed_rotations.clone(),
            name: self.name.clone(),
            typestate: PhantomData,
        }
    }
}

impl<T: DirectionSet> NodeModel<T> {
    /// Specify that this [`NodeModel`] can be rotated in exactly one way: `rotation` (in addition to the default [`NodeRotation::Rot0`] rotation)
    ///
    /// Rotations are specified as counter-clockwise
    pub fn with_rotation(mut self, rotation: NodeRotation) -> Self {
        self.allowed_rotations = HashSet::from([NodeRotation::Rot0, rotation]);
        self
    }
    /// Specify that this [`NodeModel`] can be rotated in every way specified in `rotations`, (in addition to the default [`NodeRotation::Rot0`] rotation)
    ///
    /// Rotations are specified as counter-clockwise
    pub fn with_rotations<R: Into<HashSet<NodeRotation>>>(mut self, rotations: R) -> Self {
        self.allowed_rotations = rotations.into();
        self.allowed_rotations.insert(NodeRotation::Rot0);
        self
    }
    /// Specify that this [`NodeModel`] can be rotated in every way (in addition to the default [`NodeRotation::Rot0`] rotation)
    ///
    /// Rotations are specified as counter-clockwise
    pub fn with_all_rotations(mut self) -> Self {
        self.allowed_rotations = ALL_NODE_ROTATIONS.iter().cloned().collect();
        self
    }
    /// Specify that this [`NodeModel`] can not be rotated in any way except the default [`NodeRotation::Rot0`] rotation
    pub fn with_no_rotations(mut self) -> Self {
        self.allowed_rotations = HashSet::from([NodeRotation::Rot0]);
        self
    }
    /// Specify this [`NodeModel`] weight. Used by a [`Generator`] when using [`ModelSelectionHeuristic::WeightedProbability`]. All the variations(rotations) of this [`NodeModel`] will use the same weight.
    pub fn with_weight<W: Into<f32>>(mut self, weight: W) -> Self {
        self.weight = weight.into();
        self
    }

    #[allow(unused_mut)]
    /// Register the given name for this model.
    ///
    /// Does nothing if the `debug-traces` feature is not enabled.
    pub fn with_name(mut self, _name: &'static str) -> Self {
        #[cfg(feature = "debug-traces")]
        {
            self.name = Some(_name);
        }

        self
    }

    fn rotated_sockets(&self, rotation: NodeRotation, rot_axis: Direction) -> Vec<Vec<Socket>> {
        let mut rotated_sockets = vec![Vec::new(); self.sockets.len()];

        // Not pretty: if the node sockets contain the rotation axis
        if self.sockets.len() > rot_axis as usize {
            // Sockets on the rotation axis are marked as rotated
            for fixed_axis in [rot_axis, rot_axis.opposite()] {
                rotated_sockets[fixed_axis as usize]
                    .extend(self.sockets[fixed_axis as usize].clone());
                for socket in &mut rotated_sockets[fixed_axis as usize] {
                    socket.rotate(rotation);
                }
            }
        }

        let basis = rot_axis.rotation_basis();
        let mut rotated_basis = basis.to_vec();
        rotated_basis.rotate_right(rotation.index() as usize);

        for i in 0..basis.len() {
            rotated_sockets[basis[i] as usize]
                .extend(self.sockets[rotated_basis[i] as usize].clone());
        }
        rotated_sockets
    }
}

#[derive(Debug)]
pub struct ExpandedNodeModel {
    /// Allowed connections for this [`NodeModel`] in the output
    sockets: Vec<Vec<SocketId>>,
    /// Weight factor influencing the density of this [`NodeModel`] in the generated output. Defaults to 1
    weight: f32,
    /// Index of the [`NodeModel`] this was expanded from
    original_index: ModelIndex,
    /// Rotation of the [`NodeModel`]
    rotation: NodeRotation,

    #[cfg(feature = "debug-traces")]
    pub name: Option<&'static str>,
}

impl ExpandedNodeModel {
    pub fn sockets(&self) -> &Vec<Vec<SocketId>> {
        &self.sockets
    }
    pub fn weight(&self) -> f32 {
        self.weight
    }
    pub fn original_index(&self) -> ModelIndex {
        self.original_index
    }
    pub fn rotation(&self) -> NodeRotation {
        self.rotation
    }

    pub(crate) fn to_instance(&self) -> ModelInstance {
        ModelInstance {
            model_index: self.original_index,
            rotation: self.rotation,
        }
    }
}

#[cfg(feature = "debug-traces")]
impl fmt::Display for ExpandedNodeModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{} ({:?}) rotation {}]",
            self.original_index,
            self.name,
            self.rotation.value()
        )
    }
}

/// Used to identify a specific variation of an input model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelInstance {
    /// Index of the [`NodeModel`] this was expanded from
    pub model_index: ModelIndex,
    /// Rotation of the [`NodeModel`]
    pub rotation: NodeRotation,
}

/// Output of a [`Generator`] in the context of its [`GridDefinition`].
#[derive(Clone, Copy, Debug)]
pub struct GridNode {
    /// Index of the node in the [`crate::grid::GridDefinition`]
    pub node_index: usize,
    /// Generated node data
    pub model_instance: ModelInstance,
}

/// Represents a rotation around an Axis, in the trigonometric(counterclockwise) direction
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum NodeRotation {
    Rot0,
    Rot90,
    Rot180,
    Rot270,
}

impl NodeRotation {
    /// Returns the value of the rotation in °(degrees).
    pub fn value(&self) -> u32 {
        match *self {
            NodeRotation::Rot0 => 0,
            NodeRotation::Rot90 => 90,
            NodeRotation::Rot180 => 180,
            NodeRotation::Rot270 => 270,
        }
    }
    /// Returns the index of the enum member in the enumeration.
    pub fn index(&self) -> u8 {
        match *self {
            NodeRotation::Rot0 => 0,
            NodeRotation::Rot90 => 1,
            NodeRotation::Rot180 => 2,
            NodeRotation::Rot270 => 3,
        }
    }

    #[inline]
    pub fn rotated(&self, rotation: NodeRotation) -> NodeRotation {
        ALL_NODE_ROTATIONS
            [(self.index() as usize + rotation.index() as usize) % ALL_NODE_ROTATIONS.len()]
    }

    #[inline]
    pub fn rotate(&mut self, rotation: NodeRotation) {
        *self = self.rotated(rotation);
    }

    #[inline]
    pub fn next(&self) -> NodeRotation {
        self.rotated(NodeRotation::Rot90)
    }
}

/// All the possible rotations for a [`NodeModel`]
pub const ALL_NODE_ROTATIONS: &'static [NodeRotation] = &[
    NodeRotation::Rot0,
    NodeRotation::Rot90,
    NodeRotation::Rot180,
    NodeRotation::Rot270,
];
