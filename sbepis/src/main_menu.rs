use crate::prelude::*;
use bevy::app::{AppLabel, MainSchedulePlugin};
use bevy::asset::RenderAssetUsages;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::winit::WinitPlugin;
use bevy_butler::*;

#[add_plugin(to_plugin = SbepisPlugin)]
struct MainMenuPlugin;

#[butler_plugin]
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PortalImage>();

        let mut sub_app = SubApp::new();
        sub_app.update_schedule = Some(Main.intern());

        sub_app.init_resource::<AppTypeRegistry>();
        sub_app.register_type::<Name>();
        sub_app.register_type::<ChildOf>();
        sub_app.register_type::<Children>();

        sub_app.add_plugins(MainSchedulePlugin);
        sub_app.add_systems(
            First,
            bevy::ecs::event::event_update_system
                .in_set(bevy::ecs::event::EventUpdates)
                .run_if(bevy::ecs::event::event_update_condition),
        );
        sub_app.add_event::<AppExit>();

        sub_app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>());
        sub_app.add_systems(Startup, setup);

        sub_app.set_extract(|main_world, sub_world| {
            if let Some(portal_image) = sub_world.get_resource::<PortalImage>() {
                main_world.resource_mut::<PortalImage>().0 = portal_image.0.clone();
            }
        });

        app.insert_sub_app(MainMenuApp, sub_app);
    }
}

#[derive(Resource, Default, Debug)]
struct PortalImage(Option<Handle<Image>>);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, AppLabel)]
struct MainMenuApp;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let image_handle = images.add(image);
    commands.insert_resource(PortalImage(Some(image_handle.clone())));

    let cube_handle = meshes.add(Cuboid::new(4.0, 4.0, 4.0));
    let cube_material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.7, 0.6),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    commands.spawn((
        Mesh3d(cube_handle),
        MeshMaterial3d(cube_material_handle),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    ));

    commands.spawn((
        PointLight::default(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
    ));

    commands.spawn((
        Camera3d::default(),
        Camera {
            target: image_handle.clone().into(),
            clear_color: Color::WHITE.into(),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
