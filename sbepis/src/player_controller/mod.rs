use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_marching_cubes::chunk_generator::ChunkLoader;
use bevy_pretty_nice_input::{
    Action, ButtonPress, ComponentBuffer, Cooldown, FilterBuffered, InputBuffer, ResetBuffer,
    binding1d, binding2d, input, input_transition,
};
use bevy_pretty_nice_menus::{Menu, MenuInputOf, MenuStack, MenuWithInput, MenuWithoutMouse};
use bevy_rapier3d::prelude::*;
use movement::MovementControlSystems;
use movement::di::WalkDI;
use movement::stand::Standing;
use stamina::Stamina;

use crate::camera::PlayerCamera;
use crate::gridbox_material;
use crate::inventory::Inventory;
use crate::main_bundles::Mob;
use crate::player_controller::movement::charge::{
    Charge, ChargeCrouch, ChargeCrouching, ChargeDash, ChargeStanding, ChargeWalk, ChargeWalking,
};
use crate::player_controller::movement::crouch::{Crouch, Crouching};
use crate::player_controller::movement::dash::{Dash, Dashing, HasEnoughStaminaToDash};
use crate::player_controller::movement::grounded::Grounded;
use crate::player_controller::movement::jump::{
    ChargeCrouchJump, ChargeJump, CrouchJump, HasEnoughStaminaToChargeCrouchJump,
    HasEnoughStaminaToChargeJump, HasEnoughStaminaToCrouchJump, HasEnoughStaminaToJump, Jump,
};
use crate::player_controller::movement::roll::{
    CrouchRoll, NeutralCrouchRoll, NeutralRolling, RollNeutral, Rolling, SprintRoll,
};
use crate::player_controller::movement::slide::{
    NeutralSliding, Slide, SlideNeutral, SlideStand, Sliding,
};
use crate::player_controller::movement::sneak::{CrouchSneak, Sneaking, WalkSneak};
use crate::player_controller::movement::sprint::{
    Sprint, SprintStanding, SprintWalk, Sprinting, UnSprintWalk,
};
use crate::player_controller::movement::trip::{GroundParry, Trip, TripRecover};
use crate::player_controller::movement::walk::{Walk, Walking};
use crate::prelude::*;
use crate::worldgen::terrain::WorldGen;

use self::camera_controls::*;
use self::weapons::hammer::*;
use self::weapons::rifle::*;
use self::weapons::sword::*;
use self::weapons::*;

pub mod camera_controls;
pub mod movement;
#[cfg(feature = "movement_indicators")]
mod movement_indicators;
pub mod stamina;
pub mod weapons;

#[add_plugin(to_plugin = SbepisPlugin)]
pub struct PlayerControllerPlugin;
#[butler_plugin]
impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                MovementControlSystems::UpdateDi.before(MovementControlSystems::UpdateState),
                MovementControlSystems::UpdateGrounded.before(MovementControlSystems::UpdateState),
                MovementControlSystems::DoHorizontalMovement
                    .after(MovementControlSystems::UpdateState),
                MovementControlSystems::DoVerticalMovement
                    .after(MovementControlSystems::UpdateState),
            ),
        );
    }
}

// TODO: figure out how to UNIT TEST this stuff
#[add_system(plugin = PlayerControllerPlugin, schedule = OnEnter(GameState::InGame))]
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    mut menu_stack: ResMut<MenuStack>,
) -> Result {
    let input_bundle = (
        input_transition!(Walk: Standing <=> Walking, Axis2D[binding2d::wasd()]),
        input_transition!(Jump: (Standing, Walking) => *, Axis1D[binding1d::space()], [
            ButtonPress::default(),
            InputBuffer::new(0.2),
            FilterBuffered::<Grounded>::default(),
            HasEnoughStaminaToJump,
            Cooldown::new(0.5),
            ResetBuffer,
        ]),
        input!(Look, Axis2D[binding2d::mouse_move()]),
        (
            input_transition!(Dash: Walking => *, Axis1D[binding1d::left_shift()], [
                ButtonPress::default(),
                InputBuffer::new(0.2),
                FilterBuffered::<Grounded>::default(),
                HasEnoughStaminaToDash,
                Cooldown::new(0.5),
                ResetBuffer,
            ]),
            input_transition!(Sprint: Walking <=> Sprinting, Axis1D[binding1d::left_shift()]),
            input_transition!(SprintWalk: SprintStanding <=> Sprinting, Axis2D[binding2d::wasd()]),
            input_transition!(UnSprintWalk: Standing <= SprintStanding, Axis1D[binding1d::left_shift()]),
            input_transition!(Crouch: Standing <=> Crouching, Axis1D[binding1d::left_ctrl()]),
            input_transition!(CrouchJump: (Crouching, Sneaking, Sliding, Rolling) => *, Axis1D[binding1d::space()], [
                ButtonPress::default(),
                InputBuffer::new(0.2),
                FilterBuffered::<Grounded>::default(),
                HasEnoughStaminaToCrouchJump,
                Cooldown::new(0.5),
                ResetBuffer,
            ]),
            input_transition!(CrouchSneak: Crouching <=> Sneaking, Axis2D[binding2d::wasd()]),
            input_transition!(WalkSneak: Walking <= Sneaking, Axis1D[binding1d::left_ctrl()]),
        ),
        (
            input_transition!(Slide: Walking <=> Sliding, Axis1D[binding1d::left_ctrl()]),
            input_transition!(SlideNeutral: NeutralSliding <=> Sliding, Axis2D[binding2d::wasd()]),
            input_transition!(SlideStand: Standing <= NeutralSliding, Axis1D[binding1d::left_ctrl()]),
            input_transition!(CrouchRoll: (Sliding <=, Sneaking) => Rolling, Axis1D[binding1d::left_shift()]),
            input_transition!(RollNeutral: NeutralRolling <=> Rolling, Axis2D[binding2d::wasd()]),
            input_transition!(NeutralCrouchRoll: (NeutralSliding <=, Crouching) => NeutralRolling, Axis1D[binding1d::left_shift()]),
            input_transition!(SprintRoll: (Sprinting <=, Dashing) => Rolling, Axis1D[binding1d::left_ctrl()]),
        ),
        (
            input_transition!(Charge: Standing <=> ChargeStanding, Axis1D[binding1d::left_shift()]),
            input_transition!(ChargeJump: ChargeStanding => Standing, Axis1D[binding1d::space()], [
                ButtonPress::default(),
                InputBuffer::new(0.2),
                FilterBuffered::<Grounded>::default(),
                HasEnoughStaminaToChargeJump,
                Cooldown::new(0.5),
                ResetBuffer,
            ]),
            input_transition!(ChargeCrouch: ChargeStanding <=> ChargeCrouching, Axis1D[binding1d::left_ctrl()]),
            input_transition!(ChargeCrouchJump: ChargeCrouching => Crouching, Axis1D[binding1d::space()], [
                ButtonPress::default(),
                InputBuffer::new(0.2),
                FilterBuffered::<Grounded>::default(),
                HasEnoughStaminaToChargeCrouchJump,
                Cooldown::new(0.5),
                ResetBuffer,
            ]),
            input_transition!(ChargeWalk: ChargeStanding <=> ChargeWalking, Axis2D[binding2d::wasd()]),
            input_transition!(ChargeDash: * <= ChargeWalking, Axis1D[binding1d::left_shift()]),
            input_transition!(Trip: * <= ChargeCrouching, Axis1D[binding1d::left_shift()]),
            input!(
                GroundParry,
                Axis1D[binding1d::left_ctrl()],
                [
                    ButtonPress::default(),
                    InputBuffer::new(0.2),
                    FilterBuffered::<Grounded>::default(),
                    FilterBuffered::<TripRecover>::default(),
                    ResetBuffer,
                ],
            ),
        ),
        input!(Interact, Axis1D[binding1d::key(KeyCode::KeyE)]),
        // TODO: move these
        input!(OpenQuestScreen, Axis1D[binding1d::key(KeyCode::KeyJ)]),
        input!(OpenInventory, Axis1D[binding1d::key(KeyCode::KeyV)]),
        input!(OpenStaff, Axis1D[binding1d::key(KeyCode::Backquote)]),
        (
            ComponentBuffer::<Grounded>::observe(0.2),
            ComponentBuffer::<TripRecover>::observe(0.2),
        ),
    );

    let input = commands
        .spawn((
            Menu,
            MenuWithInput,
            MenuWithoutMouse,
            DespawnOnExit(GameState::InGame),
        ))
        .id();
    menu_stack.push(input);

    let collider = commands
        .spawn((
            Name::new("Player Collider"),
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            DespawnOnExit(GameState::InGame),
        ))
        .id();

    let mesh = commands
        .spawn((
            Name::new("Player Mesh"),
            MeshMaterial3d(gridbox_material("white", &mut materials, &asset_server)),
            DespawnOnExit(GameState::InGame),
        ))
        .id();

    let camera = commands
        .spawn((
            Name::new("Player Camera"),
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: 70.0 / 180. * PI,
                ..default()
            }),
            PlayerCamera,
            Pitch(0.0),
            SpatialListener::new(-0.25),
            DespawnOnExit(GameState::InGame),
            PostProcessSettings {
                intensity: 0.02,
                radius: 4.0,
            },
            Msaa::Off,
        ))
        .id();

    let body = commands
        .spawn((
            Name::new("Player Body"),
            Transform::from_translation(Vec3::new(5.0, 10.0, 0.0)),
            Mob,
            Inventory::default(),
            WalkDI::default(),
            Stamina {
                current: 1.0,
                max: 1.0,
                recovery_rate: 0.1,
            },
            Standing,
            PlayerBody {
                camera,
                collider,
                mesh,
            },
            ChunkLoader::<WorldGen>::new(3),
            Ccd::enabled(),
            DespawnOnExit(GameState::InGame),
            UninitializedWeaponSet,
            input_bundle,
            MenuInputOf(input),
        ))
        .add_children(&[camera, collider, mesh])
        .id();

    spawn_hammer(
        &mut commands,
        &asset_server,
        &mut materials,
        &mut meshes,
        &mut animations,
        &mut graphs,
        body,
    );

    spawn_sword(
        &mut commands,
        &asset_server,
        &mut materials,
        &mut meshes,
        &mut animations,
        &mut graphs,
        body,
    );

    spawn_rifle(
        &mut commands,
        &asset_server,
        &mut materials,
        &mut meshes,
        &mut animations,
        &mut graphs,
        body,
    );

    commands.spawn((
        Name::new("Damage Numbers"),
        Text("Damage".to_owned()),
        TextLayout::new_with_justify(Justify::Right),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        DamageNumbers,
        UiTargetCamera(camera),
    ));

    commands.spawn((
        Name::new("Debug Collider Visualizer"),
        DebugColliderVisualizer,
        CollisionGroups::new(Group::NONE, Group::NONE),
    ));

    Ok(())
}

#[cfg(feature = "debug_movement_graph")]
#[add_system(plugin = PlayerControllerPlugin, schedule = OnEnter(GameState::InGame), after = setup)]
fn debug_graph(graph: Res<bevy_pretty_nice_input::debug_graph::DebugGraph>) {
    use itertools::Itertools;
    let output = format!(
        "{}\n{}",
        graph.nodes.iter().join("\n"),
        graph
            .edges
            .iter()
            .map(|e| format!("{} {} {}", e.0, e.1, e.2))
            .join("\n")
    );
    println!("{output}");
}

#[derive(Component)]
pub struct PlayerBody {
    pub camera: Entity,
    pub mesh: Entity,
    pub collider: Entity,
}

#[derive(Action)]
pub struct Interact;

#[derive(Action)]
pub struct OpenQuestScreen;

#[derive(Action)]
pub struct OpenInventory;

#[derive(Action)]
pub struct OpenStaff;
