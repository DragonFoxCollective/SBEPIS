use crate::prelude::*;
use bevy::app::{AppLabel, MainSchedulePlugin};
use bevy::asset::RenderAssetUsages;
use bevy::color::palettes::css;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy_butler::*;

#[add_plugin(to_plugin = SbepisPlugin)]
struct MainMenuPlugin;

#[butler_plugin]
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
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
        image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT;
        let image_handle = app.world_mut().resource_mut::<Assets<Image>>().add(image);
        app.insert_resource::<PortalImage>(PortalImage(image_handle.clone()));

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

        sub_app.insert_resource::<PortalImage>(PortalImage(image_handle.clone()));
        sub_app.insert_resource::<MainMenuAssets>(MainMenuAssets {
            cube_handle: app
                .world_mut()
                .resource_mut::<Assets<Mesh>>()
                .add(Cuboid::new(4.0, 4.0, 4.0)),
            cube_material_handle: app
                .world_mut()
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    base_color: Color::srgb(0.8, 0.7, 0.6),
                    reflectance: 0.02,
                    unlit: false,
                    ..default()
                }),
        });

        app.add_systems(Startup, setup_main);
        sub_app.add_systems(Startup, setup_sub);

        sub_app.set_extract(|main_world, sub_world| {
            // can't get render app :(
        });

        app.insert_sub_app(MainMenuApp, sub_app);
    }
}

#[derive(Resource, Debug)]
struct PortalImage(Handle<Image>);

#[derive(Resource, Debug)]
struct MainMenuAssets {
    cube_handle: Handle<Mesh>,
    cube_material_handle: Handle<StandardMaterial>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, AppLabel)]
struct MainMenuApp;

fn setup_main(mut commands: Commands, portal_image: Res<PortalImage>) {
    commands.spawn((
        Name::new("Main Menu"),
        ImageNode::new(portal_image.0.clone()),
        Node {
            width: Val::Px(100.0),
            height: Val::Px(100.0),
            ..default()
        },
        BackgroundColor(css::GREY.into()),
    ));
}

fn setup_sub(
    mut commands: Commands,
    portal_image: Res<PortalImage>,
    main_menu_assets: Res<MainMenuAssets>,
) {
    commands.spawn((
        Mesh3d(main_menu_assets.cube_handle.clone()),
        MeshMaterial3d(main_menu_assets.cube_material_handle.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    ));

    commands.spawn((
        PointLight::default(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
    ));

    commands.spawn((
        Camera3d::default(),
        Camera {
            target: portal_image.0.clone().into(),
            clear_color: Color::WHITE.into(),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    debug!("Main Menu Setup Complete");
}
