use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;

use crate::player::camera::PlayerCameraPlugin;
use crate::prelude::*;

#[auto_component(plugin = PlayerCameraPlugin, derive, reflect, register)]
pub struct PlayerFov(pub f32);

#[derive(Reflect)]
pub struct InterpolateFovCurve {
    pub fov: f32,
    pub duration_secs: f32,
    pub ease: EaseFunction,
}

#[auto_component(plugin = PlayerCameraPlugin, derive, reflect, register)]
pub struct InterpolateFov {
    pub curves: Vec<InterpolateFovCurve>,
}

impl InterpolateFov {
    pub fn new(fov: f32, duration_secs: f32) -> Self {
        Self {
            curves: vec![InterpolateFovCurve {
                fov,
                duration_secs,
                ease: EaseFunction::CircularOut,
            }],
        }
    }
}

type BoxedCurveInner = dyn Curve<f32> + Send + Sync;
type BoxedCurve = Box<BoxedCurveInner>;

#[auto_component(plugin = PlayerCameraPlugin, derive, reflect, register)]
#[reflect(from_reflect = false)]
struct InterpolateFovBuilt {
    #[reflect(ignore)]
    easing: BoxedCurve,
}

#[auto_observer(plugin = PlayerCameraPlugin)]
fn build_interpolate_fov(
    add: On<Add, InterpolateFov>,
    players: Query<(&Player, &InterpolateFov)>,
    cameras: Query<&Projection>,
    time: Res<Time>,
    mut commands: Commands,
) -> Result {
    let (player, fov) = players.get(add.entity)?;
    let projection = cameras.get(player.camera)?;
    let Projection::Perspective(projection) = projection else {
        return Ok(());
    };

    let mut easings = fov
        .curves
        .iter()
        .fold(
            (Vec::new(), projection.fov, time.elapsed_secs()),
            |(mut vec, old_fov, old_time), f| {
                let new_time = old_time + f.duration_secs;
                vec.push(
                    EasingCurve::new(old_fov, f.fov, f.ease)
                        .reparametrize_linear(Interval::new(old_time, new_time).unwrap())
                        .unwrap(),
                );
                (vec, f.fov, new_time)
            },
        )
        .0;
    let first: BoxedCurve = Box::new(easings.remove(0));
    // I am so bad at using boxes
    fn folder(a: BoxedCurve, b: LinearReparamCurve<f32, EasingCurve<f32>>) -> BoxedCurve {
        Box::new(a.chain(b).unwrap())
    }
    let easing: BoxedCurve = easings.into_iter().fold(first, folder);

    commands
        .entity(add.entity)
        .remove::<InterpolateFov>()
        .insert(InterpolateFovBuilt { easing });
    Ok(())
}

#[auto_system(plugin = PlayerCameraPlugin, schedule = Update)]
fn interpolate_fov(
    players: Query<(&Player, &InterpolateFovBuilt)>,
    mut cameras: Query<&mut Projection>,
    time: Res<Time>,
) -> Result {
    for (player, fov) in players.iter() {
        let mut projection = cameras.get_mut(player.camera)?;
        let Projection::Perspective(projection) = projection.as_mut() else {
            continue;
        };
        projection.fov = fov.easing.sample_clamped(time.elapsed_secs());
    }
    Ok(())
}
