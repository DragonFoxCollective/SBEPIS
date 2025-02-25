use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;
use movement::DirectionalInput;

use crate::camera::PlayerCamera;
use crate::gridbox_material;
use crate::input::*;
use crate::inventory::Inventory;
use crate::main_bundles::Mob;
use crate::menus::{
	InputManagerMenuPlugin, Menu, MenuStack, MenuWithInputManager, MenuWithoutMouse,
};

use self::camera_controls::*;
use self::weapons::hammer::*;
use self::weapons::rifle::*;
use self::weapons::sword::*;
use self::weapons::*;

pub mod camera_controls;
pub mod movement;
pub mod speed_indicator;
pub mod weapons;

#[butler_plugin(build(
	add_plugins(InputManagerMenuPlugin::<PlayerAction>::default()),
))]
pub struct PlayerControllerPlugin;

#[system(
	plugin = PlayerControllerPlugin, schedule = Startup,
)]
fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut animations: ResMut<Assets<AnimationClip>>,
	mut graphs: ResMut<Assets<AnimationGraph>>,
	asset_server: Res<AssetServer>,
	mut menu_stack: ResMut<MenuStack>,
) {
	let input = commands
		.spawn((
			input_manager_bundle(
				InputMap::default()
					.with_dual_axis(PlayerAction::Move, VirtualDPad::wasd())
					.with(PlayerAction::Jump, KeyCode::Space)
					.with_dual_axis(PlayerAction::Look, MouseMove::default())
					.with(PlayerAction::Sprint, KeyCode::ShiftLeft)
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
		))
		.id();
	menu_stack.push(input);

	let body = commands
		.spawn((
			Name::new("Player Body"),
			Transform::from_translation(Vec3::new(5.0, 10.0, 0.0)),
			Mesh3d(
				meshes.add(
					Capsule3d::new(0.25, 1.0)
						.mesh()
						.rings(1)
						.latitudes(8)
						.longitudes(16)
						.uv_profile(CapsuleUvProfile::Fixed),
				),
			),
			MeshMaterial3d(gridbox_material("white", &mut materials, &asset_server)),
			Collider::capsule_y(0.5, 0.25),
			Mob,
			PlayerBody { is_grounded: false },
			Inventory::default(),
			DirectionalInput::default(),
		))
		.id();

	let camera = commands
		.spawn((
			Name::new("Player Camera"),
			Camera3d::default(),
			Transform::from_translation(Vec3::Y * 0.5),
			Projection::Perspective(PerspectiveProjection {
				fov: 70.0 / 180. * PI,
				..default()
			}),
			PlayerCamera,
			Pitch(0.0),
			SpatialListener::new(-0.25),
		))
		.set_parent(body)
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
		Node {
			position_type: PositionType::Absolute,
			bottom: Val::Px(5.0),
			right: Val::Px(5.0),
			..default()
		},
		DamageNumbers,
		TargetCamera(camera),
	));

	commands.spawn((
		Name::new("Debug Collider Visualizer"),
		DebugColliderVisualizer,
		CollisionGroups::new(Group::NONE, Group::NONE),
	));
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
)]
fn update_is_grounded(
	mut bodies: Query<(Entity, &mut PlayerBody, &GlobalTransform)>,
	rapier_context: Query<&RapierContext>,
) {
	let rapier_context = rapier_context.single();
	for (entity, mut body, transform) in bodies.iter_mut() {
		body.is_grounded = false;
		rapier_context.intersections_with_shape(
			transform.translation() - transform.rotation() * Vec3::Y * 0.5,
			Quat::IDENTITY,
			&Collider::ball(0.25),
			QueryFilter::default(),
			|collided_entity| {
				if collided_entity == entity {
					true
				} else {
					body.is_grounded = true;
					false
				}
			},
		);
	}
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub enum PlayerAction {
	Move,
	Jump,
	Look,
	Sprint,
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
