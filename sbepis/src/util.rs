use bevy::ecs::query::{QueryData, QueryFilter, ROQueryItem};
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use bevy_rapier3d::math::Real;
use return_ok::ok_or_return;
use std::array::IntoIter;
use std::ops::{Add, Mul, Range, Sub};

use crate::SbepisPlugin;
use crate::camera::PlayerCamera;
use crate::prelude::PlayerBody;

pub trait MapRange<T> {
    fn map_range(self, range_out: Range<T>) -> T;
}
impl<T> MapRange<T> for Real
where
    T: Copy + Sub<Output = T> + Mul<Real, Output = T> + Add<Output = T>,
{
    #[inline]
    fn map_range(self, range_out: Range<T>) -> T {
        (range_out.end - range_out.start) * self + range_out.start
    }
}

pub trait MapRangeBetween<T> {
    fn map_range_between(self, range_in: Range<T>, range_out: Range<T>) -> T;
    fn map_to_01(self, range_in: Range<T>) -> Self;
    fn map_from_01(self, range_out: Range<T>) -> T;
}
impl MapRangeBetween<Real> for Real {
    fn map_range_between(self, range_in: Range<Real>, range_out: Range<Real>) -> Real {
        self.map_to_01(range_in).map_from_01(range_out)
    }

    fn map_to_01(self, range_in: Range<Real>) -> Real {
        (self - range_in.start) / (range_in.end - range_in.start)
    }

    fn map_from_01(self, range_out: Range<Real>) -> Real {
        self * (range_out.end - range_out.start) + range_out.start
    }
}

pub trait TransformEx {
    fn transform_vector3(&self, vector: Vec3) -> Vec3;
    fn inverse_transform_point(&self, point: Vec3) -> Vec3;
    #[allow(dead_code)]
    fn inverse_transform_vector3(&self, vector: Vec3) -> Vec3;
}
impl TransformEx for GlobalTransform {
    fn transform_vector3(&self, vector: Vec3) -> Vec3 {
        self.affine().transform_vector3(vector)
    }

    fn inverse_transform_point(&self, point: Vec3) -> Vec3 {
        self.affine().inverse().transform_point3(point)
    }

    fn inverse_transform_vector3(&self, vector: Vec3) -> Vec3 {
        self.affine().inverse().transform_vector3(vector)
    }
}

pub trait IterElements<T, const N: usize> {
    fn iter_elements(&self) -> IntoIter<T, N>;
}
impl IterElements<f32, 3> for Vec3 {
    fn iter_elements(&self) -> IntoIter<f32, 3> {
        [self.x, self.y, self.z].into_iter()
    }
}

#[auto_component(plugin = SbepisPlugin, derive(Deref, DerefMut), reflect, register)]
pub struct DespawnTimer(Timer);

impl DespawnTimer {
    pub fn new(duration: f32) -> Self {
        Self(Timer::from_seconds(duration, TimerMode::Once))
    }
}

#[auto_system(plugin = SbepisPlugin, schedule = Update)]
fn despawn_after_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DespawnTimer)>,
) {
    for (entity, mut despawn_timer) in query.iter_mut() {
        despawn_timer.tick(time.delta());
        if despawn_timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[auto_component(plugin = SbepisPlugin, derive, reflect, register)]
#[require(Transform, Visibility)]
pub struct Billboard;

#[auto_system(plugin = SbepisPlugin, schedule = Update)]
fn billboard(
    mut transforms: Query<&mut Transform, With<Billboard>>,
    player_camera: Query<&GlobalTransform, With<PlayerCamera>>,
    player_body: Query<&GlobalTransform, With<PlayerBody>>,
) {
    let player_camera_position = ok_or_return!(player_camera.single()).translation();
    let player_body = player_body
        .single()
        .map(GlobalTransform::up)
        .unwrap_or(Dir3::Y);
    for mut transform in transforms.iter_mut() {
        transform.look_at(player_camera_position, player_body);
    }
}

pub trait QuaternionEx {
    fn from_look_at(position: Vec3, target: Vec3, up: impl TryInto<Dir3>) -> Quat;
    fn from_look_to(direction: impl TryInto<Dir3>, up: impl TryInto<Dir3>) -> Quat;
}

impl QuaternionEx for Quat {
    fn from_look_at(position: Vec3, target: Vec3, up: impl TryInto<Dir3>) -> Quat {
        Self::from_look_to(target - position, up)
    }

    fn from_look_to(direction: impl TryInto<Dir3>, up: impl TryInto<Dir3>) -> Quat {
        let back = -direction.try_into().unwrap_or(Dir3::NEG_Z);
        let up = up.try_into().unwrap_or(Dir3::Y);
        let right = up
            .cross(back.into())
            .try_normalize()
            .unwrap_or_else(|| up.any_orthonormal_vector());
        let up = back.cross(right);
        Quat::from_mat3(&Mat3::from_cols(right, up, back.into()))
    }
}

#[derive(Clone)]
pub struct DomainedEasingData<T>
where
    T: Ease + Clone,
{
    pub domain_start: f32,
    pub domain_end: f32,
    pub start: T,
    pub end: T,
    pub easing: EaseFunction,
}

impl<T> DomainedEasingData<T>
where
    T: Ease + Clone,
{
    pub fn new(domain_start: f32, domain_end: f32, start: T, end: T, easing: EaseFunction) -> Self {
        Self {
            domain_start,
            domain_end,
            start,
            end,
            easing,
        }
    }

    pub fn into_curve(self) -> LinearReparamCurve<T, EasingCurve<T>> {
        EasingCurve::new(self.start, self.end, self.easing)
            .reparametrize_linear(Interval::new(self.domain_start, self.domain_end).unwrap())
            .unwrap()
    }
}

pub fn find_in_ancestors<'w, 's: 'w, 'a: 'w, D: QueryData, F: QueryFilter>(
    entity: Entity,
    query: &'a Query<'w, 's, D, F>,
    parents: &Query<'w, 's, &ChildOf>,
) -> Option<ROQueryItem<'w, 's, D>> {
    if let Ok(item) = query.get(entity) {
        return Some(item);
    }

    for e in parents.iter_ancestors(entity) {
        if let Ok(item) = query.get(e) {
            return Some(item);
        }
    }

    None
}

#[auto_component(plugin = SbepisPlugin, derive, reflect, register)]
pub struct AnimationRootReference(pub Entity);
