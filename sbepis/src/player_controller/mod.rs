use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_marching_cubes::chunk_generator::ChunkLoader;
use bevy_pretty_nice_input::{
    Action, ButtonPress, ComponentBuffer, Cooldown, Filter, FilterBuffered, InputBuffer,
    ResetBuffer, binding1d, binding2d, input, input_transition,
};
use bevy_pretty_nice_menus::{MenuInputOf, MenuStack, MenuWithInput, MenuWithoutMouse};
use bevy_rapier3d::prelude::*;
use movement::MovementControlSystems;
use movement::di::WalkDI;
use movement::stand::Standing;
use stamina::Stamina;

use crate::camera::PlayerCamera;
use crate::gridbox_material;
use crate::inventory::Inventory;
use crate::main_bundles::Mob;
use crate::player_controller::movement::charge::{ChargeDash, Charging};
use crate::player_controller::movement::crouch::Crouching;
use crate::player_controller::movement::dash::{Dash, HasEnoughStaminaToDash};
use crate::player_controller::movement::di::Moving;
use crate::player_controller::movement::grounded::Grounded;
use crate::player_controller::movement::jump::{
    ChargeCrouchJump, ChargeJump, CrouchJump, HasEnoughStaminaToChargeCrouchJump,
    HasEnoughStaminaToChargeJump, HasEnoughStaminaToCrouchJump, HasEnoughStaminaToJump,
    HasEnoughStaminaToSlideJump, Jump, SlideJump,
};
use crate::player_controller::movement::roll::{Rolling, RollingDI};
use crate::player_controller::movement::slide::{Sliding, SlidingDI};
use crate::player_controller::movement::trip::{GroundParry, Trip, TripRecover, Tripping};
use crate::player_controller::movement::walk::{Sprinting, StandingDI};
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

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = SbepisPlugin)]
pub struct PlayerControllerPlugin;

#[auto_plugin(plugin = PlayerControllerPlugin)]
fn build(app: &mut App) {
    app.configure_sets(
        Update,
        (
            MovementControlSystems::UpdateDi.before(MovementControlSystems::UpdateState),
            MovementControlSystems::UpdateGrounded.before(MovementControlSystems::UpdateState),
            MovementControlSystems::UpdateState
                .before(MovementControlSystems::DoHorizontalMovement),
            MovementControlSystems::UpdateState.before(MovementControlSystems::DoVerticalMovement),
            MovementControlSystems::DoHorizontalMovement
                .before(MovementControlSystems::ExecuteMovement),
            MovementControlSystems::DoVerticalMovement
                .before(MovementControlSystems::ExecuteMovement),
        ),
    );
}

// TODO: figure out how to UNIT TEST this stuff
#[auto_system(plugin = PlayerControllerPlugin, schedule = OnEnter(GameState::InGame))]
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
        (
            // Misc
            input!(Look, Axis2D[binding2d::mouse_move()]),
            input!(Interact, Axis1D[binding1d::key(KeyCode::KeyE)]),
            input!(OpenQuestScreen, Axis1D[binding1d::key(KeyCode::KeyJ)]),
            input!(OpenInventory, Axis1D[binding1d::key(KeyCode::KeyV)]),
            input!(OpenStaff, Axis1D[binding1d::key(KeyCode::Backquote)]),
            ComponentBuffer::<Grounded>::observe(0.2),
        ),
        (
            // Standing
            input_transition!((Standing) <=> StandingDI (Standing, Moving), Axis2D[binding2d::wasd()]),
            input_transition!((Standing, !Crouching) => Jump (Standing), Axis1D[binding1d::space()], [
                ButtonPress::default(),
                InputBuffer::new(0.2),
                FilterBuffered::<Grounded>::default(),
                HasEnoughStaminaToJump,
                Cooldown::new(0.5),
                ResetBuffer,
            ]),
            input_transition!((Standing, Moving) => Dash (Standing, Moving), Axis1D[binding1d::left_shift()], [
                ButtonPress::default(),
                InputBuffer::new(0.2),
                FilterBuffered::<Grounded>::default(),
                HasEnoughStaminaToDash,
                Cooldown::new(0.5),
                ResetBuffer,
            ]),
            input_transition!((Standing, Moving) => (Standing, Moving, Sprinting), Axis1D[binding1d::left_shift()]),
            input_transition!(() <= (Sprinting), Axis1D[binding1d::left_shift()]),
            input_transition!((Standing, !Moving, !Sprinting) <=> (Standing, Crouching), Axis1D[binding1d::left_ctrl()]),
            input_transition!((Standing, Crouching) => CrouchJump (Standing, Crouching), Axis1D[binding1d::space()], [
                ButtonPress::default(),
                InputBuffer::new(0.2),
                FilterBuffered::<Grounded>::default(),
                HasEnoughStaminaToCrouchJump,
                Cooldown::new(0.5),
                ResetBuffer,
            ]),
        ),
        (
            // Sliding
            input_transition!((Standing, Moving) <=> (Sliding, Moving), Axis1D[binding1d::left_ctrl()]),
            input_transition!(
                (Standing) <= (Sliding, !Moving),
                Axis1D[binding1d::left_ctrl()]
            ),
            input_transition!((Sliding) <=> SlidingDI (Sliding, Moving), Axis2D[binding2d::wasd()]),
            input_transition!((Sliding) => SlideJump (Sliding), Axis1D[binding1d::space()], [
                ButtonPress::default(),
                InputBuffer::new(0.2),
                FilterBuffered::<Grounded>::default(),
                HasEnoughStaminaToSlideJump,
                Cooldown::new(0.5),
                ResetBuffer,
            ]),
        ),
        (
            // Rolling
            input_transition!((Sliding) <=> (Rolling), Axis1D[binding1d::left_shift()]),
            input_transition!((Standing, Crouching) => (Rolling), Axis1D[binding1d::left_shift()]),
            input_transition!((Rolling) <=> RollingDI (Rolling, Moving), Axis2D[binding2d::wasd()]),
            input_transition!((Standing, Sprinting) <=> (Rolling), Axis1D[binding1d::left_ctrl()]),
        ),
        (
            // Charging
            input_transition!((Standing, !Moving, !Crouching, !Rolling) <=> (Charging, !Crouching), Axis1D[binding1d::left_shift()]),
            input_transition!((Charging, !Crouching) => ChargeJump (Standing), Axis1D[binding1d::space()], [
                ButtonPress::default(),
                InputBuffer::new(0.2),
                FilterBuffered::<Grounded>::default(),
                HasEnoughStaminaToChargeJump,
                Cooldown::new(0.5),
                ResetBuffer,
            ]),
            input_transition!((Charging) <=> (Charging, Crouching), Axis1D[binding1d::left_ctrl()]),
            input_transition!((Charging, Crouching) => ChargeCrouchJump (Charging, Crouching), Axis1D[binding1d::space()], [
                ButtonPress::default(),
                InputBuffer::new(0.2),
                FilterBuffered::<Grounded>::default(),
                HasEnoughStaminaToChargeCrouchJump,
                Cooldown::new(0.5),
                ResetBuffer,
            ]),
            input_transition!((Charging) <=> (Charging, Moving), Axis2D[binding2d::wasd()]),
            input_transition!(
                ChargeDash(Charging, Moving) <= (Charging, Moving),
                Axis1D[binding1d::left_shift()]
            ),
        ),
        (
            // Tripping
            input_transition!(
                Trip() <= (Charging, Crouching),
                Axis1D[binding1d::left_shift()]
            ),
            input_transition!((Tripping) <=> (Tripping, Moving), Axis2D[binding2d::wasd()]),
            input_transition!((TripRecover) <=> (TripRecover, Moving), Axis2D[binding2d::wasd()]),
            input!(
                GroundParry,
                Axis1D[binding1d::left_ctrl()],
                [
                    ButtonPress::default(),
                    InputBuffer::new(0.2),
                    Filter::<(With<TripRecover>, With<ComponentBuffer<Grounded>>)>::default(),
                    ResetBuffer,
                ]
            ),
        ),
    );

    let input = commands
        .spawn((
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
        ))
        .id();

    let mesh = commands
        .spawn((
            Name::new("Player Mesh"),
            MeshMaterial3d(gridbox_material("white", &mut materials, &asset_server)),
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
            PostProcessOutlinesSettings { radius: 4.0 },
            PostProcessQuantizeSettings { fixed_k: 16 },
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

    debug!("Character up!");

    Ok(())
}

#[cfg(feature = "debug_movement_graph")]
#[auto_system(plugin = PlayerControllerPlugin, schedule = OnEnter(GameState::InGame), config(
	after = setup,
))]
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

#[auto_component(plugin = PlayerControllerPlugin, derive, reflect, register)]
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
