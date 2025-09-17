use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_marching_cubes::chunk_generator::ChunkLoader;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;
use movement::MovementControlSet;
use movement::di::DirectionalInput;
use movement::stand::Standing;
use stamina::Stamina;

use crate::camera::PlayerCamera;
use crate::gridbox_material;
use crate::input::*;
use crate::inventory::Inventory;
use crate::main_bundles::Mob;
use crate::menus::{Menu, MenuStack, MenuWithInputManager, MenuWithoutMouse};
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
                MovementControlSet::UpdateDi.before(MovementControlSet::UpdateState),
                MovementControlSet::UpdateGrounded.before(MovementControlSet::UpdateState),
                MovementControlSet::DoHorizontalMovement.after(MovementControlSet::UpdateState),
                MovementControlSet::DoVerticalMovement.after(MovementControlSet::UpdateState),
            ),
        );
    }
}

#[add_plugin(to_plugin = PlayerControllerPlugin, generics = <PlayerAction>)]
use crate::menus::InputManagerMenuPlugin;

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
    let input = commands
        .spawn((
            input_manager_bundle(
                InputMap::default()
                    .with_dual_axis(PlayerAction::Move, VirtualDPad::wasd())
                    .with(PlayerAction::Jump, KeyCode::Space)
                    .with_dual_axis(PlayerAction::Look, MouseMove::default())
                    .with(PlayerAction::Sprint, KeyCode::ShiftLeft)
                    .with(PlayerAction::Crouch, KeyCode::ControlLeft)
                    .with(PlayerAction::Use, MouseButton::Left)
                    .with(PlayerAction::Interact, KeyCode::KeyE)
                    .with(PlayerAction::NextWeapon, MouseScrollDirection::UP)
                    .with(PlayerAction::PrevWeapon, MouseScrollDirection::DOWN)
                    .with(PlayerAction::OpenQuestScreen, KeyCode::KeyJ)
                    .with(PlayerAction::OpenInventory, KeyCode::KeyV)
                    .with(PlayerAction::OpenStaff, KeyCode::Backquote),
                false,
            ),
            Menu,
            MenuWithInputManager,
            MenuWithoutMouse,
            StateScoped(GameState::InGame),
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
            StateScoped(GameState::InGame),
        ))
        .id();

    let mesh = commands
        .spawn((
            Name::new("Player Mesh"),
            MeshMaterial3d(gridbox_material("white", &mut materials, &asset_server)),
            StateScoped(GameState::InGame),
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
            StateScoped(GameState::InGame),
            // PostProcessSettings { intensity: 0.02 },
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
            StateScoped(GameState::InGame),
        ))
        .add_children(&[camera, collider, mesh])
        .id();

    let (hammer_pivot, _hammer_head) = spawn_hammer(
        &mut commands,
        &asset_server,
        &mut materials,
        &mut meshes,
        &mut animations,
        &mut graphs,
        body,
    );

    let (sword_pivot, _sword_blade) = spawn_sword(
        &mut commands,
        &asset_server,
        &mut materials,
        &mut meshes,
        &mut animations,
        &mut graphs,
        body,
    );

    let (rifle_pivot, _rifle_barrel) = spawn_rifle(
        &mut commands,
        &asset_server,
        &mut materials,
        &mut meshes,
        &mut animations,
        &mut graphs,
        body,
    );

    commands.entity(body).insert((
        WeaponSet {
            weapons: vec![hammer_pivot, sword_pivot, rifle_pivot],
            active_weapon: 0,
        },
        UninitializedWeaponSet,
    ));

    commands.spawn((
        Name::new("Damage Numbers"),
        Text("Damage".to_owned()),
        TextLayout::new_with_justify(JustifyText::Right),
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

#[derive(Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub enum PlayerAction {
    Move,
    Jump,
    Look,
    Sprint,
    Crouch,
    Use,
    Interact,
    NextWeapon,
    PrevWeapon,
    OpenQuestScreen,
    OpenInventory,
    OpenStaff,
}
impl Actionlike for PlayerAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            PlayerAction::Move => InputControlKind::DualAxis,
            PlayerAction::Jump => InputControlKind::Button,
            PlayerAction::Look => InputControlKind::DualAxis,
            PlayerAction::Sprint => InputControlKind::Button,
            PlayerAction::Crouch => InputControlKind::Button,
            PlayerAction::Use => InputControlKind::Button,
            PlayerAction::Interact => InputControlKind::Button,
            PlayerAction::NextWeapon => InputControlKind::Button,
            PlayerAction::PrevWeapon => InputControlKind::Button,
            PlayerAction::OpenQuestScreen => InputControlKind::Button,
            PlayerAction::OpenInventory => InputControlKind::Button,
            PlayerAction::OpenStaff => InputControlKind::Button,
        }
    }
}

#[derive(Component)]
pub struct PlayerBody {
    pub camera: Entity,
    pub mesh: Entity,
    pub collider: Entity,
}
