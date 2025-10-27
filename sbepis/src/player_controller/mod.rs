use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_marching_cubes::chunk_generator::ChunkLoader;
use bevy_pretty_nice_input::{
    Action, ButtonPress, ButtonRelease, ComponentBuffer, Cooldown, Filter, InputBuffer,
    ResetBuffer, binding1d, binding2d, input, input_transition,
};
use bevy_pretty_nice_menus::{Menu, MenuInputOf, MenuStack, MenuWithInput, MenuWithoutMouse};
use bevy_rapier3d::prelude::*;
use movement::MovementControlSystems;
use movement::di::DirectionalInput;
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
use crate::player_controller::movement::dash::{Dash, Dashing};
use crate::player_controller::movement::grounded::Grounded;
use crate::player_controller::movement::jump::Jump;
use crate::player_controller::movement::roll::{CrouchRoll, Rolling, SprintRoll};
use crate::player_controller::movement::slide::{Slide, Sliding};
use crate::player_controller::movement::sneak::{CrouchSneak, Sneaking, WalkSneak};
use crate::player_controller::movement::sprint::{Sprint, Sprinting};
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
        input_transition!(Walk: Standing [<->] Walking, [binding2d::wasd()]),
        input!(
            Jump,                 // The Action to trigger
            [binding1d::space()], // The trigger
            [
                ButtonPress::default(), // If the trigger gets this far, if it was just pressed, continue. Otherwise, stop it
                InputBuffer::new(0.2), // If the trigger gets this far, start firing the trigger repeatedly for a certain amount of time
                Filter::<With<ComponentBuffer<Grounded>>>::default(), // If the trigger gets this far, only let it pass if the entity has this component
                Cooldown::new(0.5), // If the trigger gets this far, don't let it pass again for a certain amount of time
            ],
            // Examples:
            // Space not pressed:					blocked by bindings
            // Space just pressed, not grounded:	start in bindings,	pass Press,			start InputBuffer,		blocked by Filter
            // Space held:							start in bindings,	blocked by Press
            // Space just pressed, grounded:		start in bindings,	pass Press,			start InputBuffer,		pass Filter,		pass Cooldown
            // Space just pressed x2, grounded:		start in bindings,	pass Press,			start InputBuffer,		pass Filter,		blocked by Cooldown // don't do the multi jump bug
            // InputBuffer active, not grounded:											start in InputBuffer,	blocked by Filter
            // InputBuffer active, grounded: 												start in InputBuffer,	pass Filter,		pass Cooldown
            // InputBuffer active x2, grounded:												start in InputBuffer,	pass Filter,		blocked by Cooldown

            // Cons:
            // Unless the cooldown continuously resets the input buffer, the player will automatically jump again.
            // This is especially apparent if the cooldown is shorter than the input buffer.
        ),
        input!(Look, [binding2d::mouse_move()]),
        (
            input!(
                Dash,
                [binding1d::left_shift()],
                [
                    ButtonPress::default(),
                    InputBuffer::new(0.2),
                    Filter::<With<ComponentBuffer<Grounded>>>::default(),
                    ResetBuffer,
                ],
            ),
            input_transition!(Sprint: Walking [<->] Sprinting, [binding1d::left_shift()]),
            input_transition!(Crouch: Standing [<->] Crouching, [binding1d::left_ctrl()]),
            input_transition!(CrouchSneak: Crouching [<->] Sneaking, [binding2d::wasd()]),
            input_transition!(WalkSneak: Walking [<-] Sneaking, [binding1d::left_ctrl()]),
            input_transition!(Slide: Walking [<->] Sliding, [binding1d::left_ctrl()]),
            input_transition!(CrouchRoll: (<- Sliding, Sneaking, Crouching) [->] Rolling, [binding1d::left_shift()]),
            input_transition!(SprintRoll: (<- Sprinting, Dashing) [->] Rolling, [binding1d::left_ctrl()]),
        ),
        (
            input_transition!(Charge: Standing [<->] ChargeStanding, [binding1d::left_shift()]),
            input_transition!(ChargeCrouch: ChargeStanding [<->] ChargeCrouching, [binding1d::left_ctrl()]),
            input_transition!(ChargeWalk: ChargeStanding [<->] ChargeWalking, [binding2d::wasd()]),
            input!(
                ChargeDash,
                [binding1d::left_shift()],
                [
                    ButtonRelease::default(),
                    Filter::<With<ChargeWalking>>::default()
                ],
            ),
        ),
        (
            input!(
                Trip,
                [binding1d::left_shift()],
                [
                    ButtonRelease::default(),
                    Filter::<With<ChargeCrouching>>::default(),
                ],
            ),
            input!(
                GroundParry,
                [binding1d::left_ctrl()],
                [
                    ButtonPress::default(),
                    InputBuffer::new(0.2),
                    Filter::<(
                        With<ComponentBuffer<Grounded>>,
                        With<ComponentBuffer<TripRecover>>
                    )>::default(),
                    ResetBuffer,
                ],
            ),
        ),
        input!(Interact, [binding1d::key(KeyCode::KeyE)]),
        // TODO: move these
        input!(OpenQuestScreen, [binding1d::key(KeyCode::KeyJ)]),
        input!(OpenInventory, [binding1d::key(KeyCode::KeyV)]),
        input!(OpenStaff, [binding1d::key(KeyCode::Backquote)]),
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
            DirectionalInput::default(),
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
