use std::time::Duration;

use crate::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::core_pipeline::CorePipelinePlugin;
use bevy::pbr::PbrPlugin;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy_butler::*;
use crossbeam_channel::{SendError, SendTimeoutError, TryRecvError, TrySendError, bounded};

#[add_plugin(to_plugin = SbepisPlugin)]
struct MainMenuPlugin;

#[butler_plugin]
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = bounded(1);

        std::thread::spawn(move || {
            let mut sub_app = App::new();

            // sub_app.add_plugins(
            //     DefaultPlugins
            //         .build()
            //         .disable::<LogPlugin>()
            //         .disable::<TerminalCtrlCHandlerPlugin>()
            //         .disable::<WinitPlugin>(),
            // );
            sub_app.add_plugins((
                MinimalPlugins,
                WindowPlugin::default(),
                AssetPlugin::default(),
                RenderPlugin::default(),
                ImagePlugin::default(),
                CorePipelinePlugin,
                PbrPlugin::default(),
            ));
            sub_app.add_systems(Startup, setup_sub);

            sub_app.add_systems(
                Update,
                move |portal_image: Res<PortalImage>,
                      images: Res<Assets<Image>>,
                      mut app_exit: EventWriter<AppExit>| {
                    if let Some(image) = images.get(&portal_image.0) {
                        // match tx.try_send(image.clone()) {
                        //     Ok(_) => {}
                        //     Err(TrySendError::Full(_)) => {}
                        //     Err(TrySendError::Disconnected(_)) => {
                        //         warn!("Main menu portal image channel disconnected - sub app");
                        //         app_exit.write(AppExit::Success);
                        //     }
                        // }
                        // match tx.send(image.clone()) {
                        //     Ok(_) => {}
                        //     Err(SendError(_)) => {
                        //         warn!("Main menu portal image channel disconnected - sub app");
                        //         app_exit.write(AppExit::Success);
                        //     }
                        // }
                        match tx.send_timeout(image.clone(), Duration::from_secs(1)) {
                            Ok(_) => {}
                            Err(SendTimeoutError::Timeout(_)) => {}
                            Err(SendTimeoutError::Disconnected(_)) => {
                                warn!("Main menu portal image channel disconnected - sub app");
                                app_exit.write(AppExit::Success);
                            }
                        }
                    }
                },
            );

            sub_app.run();
        });

        app.add_systems(
            Update,
            move |portal_image: Option<Res<PortalImage>>,
                  mut images: ResMut<Assets<Image>>,
                  mut commands: Commands| {
                match rx.try_recv() {
                    Ok(image_data) => {
                        match portal_image {
                            Some(portal_image) => images.insert(&portal_image.0, image_data),
                            None => {
                                commands.insert_resource(PortalImage(images.add(image_data)));
                            }
                        };
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => {
                        // panic!("Main menu portal image channel disconnected - main app");
                    }
                }
            },
        );
    }
}

#[derive(Resource, Default, Debug)]
struct PortalImage(Handle<Image>);

fn setup_sub(
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
    commands.insert_resource(PortalImage(image_handle.clone()));

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
