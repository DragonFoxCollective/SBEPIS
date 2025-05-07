use std::f32::consts::PI;

use bevy::gltf::GltfMaterialName;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::scene::SceneInstanceReady;
use bevy_butler::*;
use bevy_hanabi::prelude::*;
use faker_rand::en_us::names::FirstName;
use meshtext::{Face, MeshGenerator, MeshText, TextSection};
use rand::seq::{IteratorRandom, SliceRandom};
use return_ok::some_or_return_ok;
use serde::Deserialize;

use crate::entity::spawner::EntitySpawnedSet;
use crate::entity::{EntityKilled, EntityKilledSet};
use crate::npcs::NpcPlugin;

#[derive(Resource)]
pub struct NameTagAssets {
    pub names: Handle<AvailableNames>,

    pub generated_material: Handle<StandardMaterial>,
    pub past_material: Handle<StandardMaterial>,
    pub pgo_material: Handle<StandardMaterial>,
    pub captcha_material: Handle<StandardMaterial>,
    pub alchemiter_material: Handle<StandardMaterial>,
    pub denizen_materials: [Handle<StandardMaterial>; 4],
    pub master_material: Handle<CandyMaterial>,

    pub denizen_particles: Handle<EffectAsset>,
    pub denizen_particles_trails: Handle<EffectAsset>,
    pub master_particles: [Handle<EffectAsset>; 2],
    pub master_particles_trails: [Handle<EffectAsset>; 2],
}

#[derive(Asset, Deserialize, TypePath)]
pub struct AvailableNames {
    names: Vec<NameTag>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
enum NameTier {
    Past,
    Pgo,
    Captcha,
    Alchemiter,
    Denizen,
    Master,
}

#[derive(Component)]
pub struct SpawnNameTag;

#[derive(Component)]
pub struct NameTagged(pub NameTag);

#[derive(Debug, Clone, Deserialize)]
pub struct NameTag {
    name: String,
    tier: Option<NameTier>,
}

#[derive(Resource)]
#[insert_resource(plugin = NpcPlugin)]
pub struct FontMeshGenerator {
    regular: MeshGenerator<Face<'static>>,
    bold: MeshGenerator<Face<'static>>,
}

impl FontMeshGenerator {
    pub fn new(regular_font_data: &'static [u8], bold_font_data: &'static [u8]) -> Self {
        Self {
            regular: MeshGenerator::new(regular_font_data),
            bold: MeshGenerator::new(bold_font_data),
        }
    }

    pub fn generate_regular(&mut self, text: &str) -> (MeshText, Mesh) {
        Self::generate(text, &mut self.regular)
    }

    pub fn generate_bold(&mut self, text: &str) -> (MeshText, Mesh) {
        Self::generate(text, &mut self.bold)
    }

    fn generate(text: &str, generator: &mut MeshGenerator<Face<'static>>) -> (MeshText, Mesh) {
        let transform = Mat4::from_scale(Vec3::new(1.0, 1.0, 0.2)).to_cols_array();
        let mesh_text: MeshText = generator
            .generate_section(text, false, Some(&transform))
            .unwrap();

        let vertices = mesh_text.vertices.clone();
        let positions: Vec<[f32; 3]> = vertices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
        let uvs_0 = positions
            .iter()
            .map(|&[x, y, _]| [x, y])
            .collect::<Vec<[f32; 2]>>();
        let uvs_1 = positions
            .iter()
            .map(|&[x, y, _]| {
                [
                    x - mesh_text.bbox.size().x * 0.5,
                    y - mesh_text.bbox.size().y * 0.5,
                ]
            })
            .collect::<Vec<[f32; 2]>>();

        let mut mesh = Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs_0);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, uvs_1);
        mesh.compute_flat_normals();

        (mesh_text, mesh)
    }
}
impl Default for FontMeshGenerator {
    fn default() -> Self {
        // Cascadia Code is broken (Err: GlyphTriangulationError(PointOnFixedEdge(1)))
        Self::new(
            include_bytes!("../../assets/FiraSans-Regular.ttf"),
            include_bytes!("../../assets/FiraSans-Bold.ttf"),
        )
    }
}

fn create_particles(color: Color) -> EffectAsset {
    let color: Srgba = color.into();
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(color.red, color.green, color.blue, 1.0));
    color_gradient.add_key(1.0, Vec4::new(color.red, color.green, color.blue, 0.0));

    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0.0, Vec3::splat(0.01));
    size_gradient.add_key(1.0, Vec3::splat(0.0));

    let mut module = Module::default();

    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(0.01),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(2.5),
    };

    let init_age = SetAttributeModifier::new(Attribute::AGE, module.lit(0.0));

    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(0.1));

    let init_ribbon_id = SetAttributeModifier {
        attribute: Attribute::U32_0,
        value: module.attr(Attribute::PARTICLE_COUNTER),
    };

    let update_drag = LinearDragModifier::new(module.lit(0.5));

    let update_spawn_trail = EmitSpawnEventModifier {
        condition: EventEmitCondition::Always,
        count: module.lit(32u32),
        child_index: 0,
    };

    let render_color = ColorOverLifetimeModifier {
        gradient: color_gradient,
        blend: ColorBlendMode::Overwrite,
        mask: ColorBlendMask::RGBA,
    };

    let render_orient = OrientModifier {
        mode: OrientMode::FaceCameraPosition,
        rotation: None,
    };

    let render_size = SizeOverLifetimeModifier {
        gradient: size_gradient,
        screen_space_size: false,
    };

    EffectAsset::new(16, SpawnerSettings::rate(5.0.into()), module)
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .init(init_ribbon_id)
        .update(update_drag)
        .update(update_spawn_trail)
        .render(render_color)
        .render(render_orient)
        .render(render_size)
}

fn create_particles_trails(color: Color) -> EffectAsset {
    let color: Srgba = color.into();
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(color.red, color.green, color.blue, 1.0));
    color_gradient.add_key(1.0, Vec4::new(color.red, color.green, color.blue, 0.0));

    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0.0, Vec3::splat(0.01));
    size_gradient.add_key(1.0, Vec3::splat(0.0));

    let mut module = Module::default();

    let inherit_position = InheritAttributeModifier::new(Attribute::POSITION);

    let init_age = SetAttributeModifier::new(Attribute::AGE, module.lit(0.0));

    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(0.1));

    let init_ribbon_id = SetAttributeModifier {
        attribute: Attribute::RIBBON_ID,
        value: module.parent_attr(Attribute::U32_0),
    };

    let render_color = ColorOverLifetimeModifier {
        gradient: color_gradient,
        blend: ColorBlendMode::Overwrite,
        mask: ColorBlendMask::RGBA,
    };

    let render_orient = OrientModifier {
        mode: OrientMode::FaceCameraPosition,
        rotation: None,
    };

    let render_size = SizeOverLifetimeModifier {
        gradient: size_gradient,
        screen_space_size: false,
    };

    EffectAsset::new(16 * 32, SpawnerSettings::rate(0.1.into()), module)
        .with_motion_integration(MotionIntegration::None)
        .init(inherit_position)
        .init(init_age)
        .init(init_lifetime)
        .init(init_ribbon_id)
        .render(render_color)
        .render(render_orient)
        .render(render_size)
}

#[add_system(
	plugin = NpcPlugin, schedule = Startup,
)]
fn load_names(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut particles: ResMut<Assets<EffectAsset>>,
    mut candy_materials: ResMut<Assets<CandyMaterial>>,
) -> Result {
    let names: Handle<AvailableNames> = asset_server.load("supporters.names.ron");

    commands.insert_resource(NameTagAssets {
        names,
        generated_material: materials.add(Color::srgb(0.4, 0.4, 0.4)),

        past_material: materials.add(Color::WHITE),
        pgo_material: materials.add(Color::from(Srgba::hex("4bec13")?)),
        captcha_material: materials.add(Color::from(Srgba::hex("ff067c")?)),
        alchemiter_material: materials.add(Color::from(Srgba::hex("03a9f4")?)),
        denizen_materials: [
            Color::from(Srgba::hex("0715cd")?),
            Color::from(Srgba::hex("b536da")?),
            Color::from(Srgba::hex("e00707")?),
            Color::from(Srgba::hex("4ac925")?),
        ]
        .map(|color| {
            materials.add(StandardMaterial {
                base_color: color,
                unlit: true,
                ..default()
            })
        }),
        master_material: candy_materials.add(CandyMaterial::default()),

        denizen_particles: particles.add(create_particles(Color::from(Srgba::hex("efbf04")?))),
        denizen_particles_trails: particles
            .add(create_particles_trails(Color::from(Srgba::hex("efbf04")?))),
        master_particles: [
            particles.add(create_particles(Color::from(Srgba::hex("ff0000")?))),
            particles.add(create_particles(Color::from(Srgba::hex("00ff00")?))),
        ],
        master_particles_trails: [
            particles.add(create_particles_trails(Color::from(Srgba::hex("ff0000")?))),
            particles.add(create_particles_trails(Color::from(Srgba::hex("00ff00")?))),
        ],
    });

    Ok(())
}

#[add_system(
	plugin = NpcPlugin, schedule = Update,
	after = EntitySpawnedSet,
)]
fn spawn_name_tags(
    mut commands: Commands,
    asset: Res<NameTagAssets>,
    mut assets: ResMut<Assets<AvailableNames>>,
    entities: Query<Entity, With<SpawnNameTag>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut font_mesh_generator: ResMut<FontMeshGenerator>,
) -> Result {
    let names = some_or_return_ok!(assets.get_mut(&asset.names));

    for entity in entities.iter() {
        let name_tag = {
            let opt = names
                .names
                .iter()
                .enumerate()
                .choose(&mut rand::thread_rng())
                .map(|(i, name)| (i, name.clone()));
            if let Some((i, name_tag)) = opt {
                names.names.swap_remove(i);
                name_tag
            } else {
                NameTag {
                    name: rand::random::<FirstName>().to_string(),
                    tier: None,
                }
            }
        };

        let (mesh_text, mesh) = match name_tag.tier {
            None => font_mesh_generator.generate_regular(&name_tag.name),
            Some(NameTier::Past) => font_mesh_generator.generate_regular(&name_tag.name),
            Some(NameTier::Pgo) => font_mesh_generator.generate_regular(&name_tag.name),
            Some(NameTier::Captcha) => font_mesh_generator.generate_regular(&name_tag.name),
            Some(NameTier::Alchemiter) => font_mesh_generator.generate_bold(&name_tag.name),
            Some(NameTier::Denizen) => font_mesh_generator.generate_regular(&name_tag.name),
            Some(NameTier::Master) => font_mesh_generator.generate_bold(&name_tag.name),
        };
        let material = match name_tag.tier {
            None => NameTagShader::Standard(asset.generated_material.clone()),
            Some(NameTier::Past) => NameTagShader::Standard(asset.past_material.clone()),
            Some(NameTier::Pgo) => NameTagShader::Standard(asset.pgo_material.clone()),
            Some(NameTier::Captcha) => NameTagShader::Standard(asset.captcha_material.clone()),
            Some(NameTier::Alchemiter) => {
                NameTagShader::Standard(asset.alchemiter_material.clone())
            }
            Some(NameTier::Denizen) => NameTagShader::Standard(
                asset
                    .denizen_materials
                    .choose(&mut rand::thread_rng())
                    .ok_or("No denizen-level materials")?
                    .clone(),
            ),
            Some(NameTier::Master) => NameTagShader::Candy(asset.master_material.clone()),
        };
        let scale = match name_tag.tier {
            None => 0.2,
            Some(NameTier::Past) => 0.2,
            Some(NameTier::Pgo) => 0.2,
            Some(NameTier::Captcha) => 0.2,
            Some(NameTier::Alchemiter) => 0.2,
            Some(NameTier::Denizen) => 0.3,
            Some(NameTier::Master) => 0.3,
        };

        let mut text_entity = commands.spawn((
            Mesh3d(meshes.add(mesh)),
            Transform::from_xyz(mesh_text.bbox.size().x * scale * 0.5, 1.1, 0.0)
                .with_rotation(Quat::from_rotation_y(PI))
                .with_scale(Vec3::splat(scale)),
        ));
        match material {
            NameTagShader::Standard(material) => {
                text_entity.insert(MeshMaterial3d(material));
            }
            NameTagShader::Candy(material) => {
                text_entity.insert(MeshMaterial3d(material));
            }
        }
        let text_entity = text_entity.insert(ChildOf(entity)).id();

        let particles = match name_tag.tier {
            None => vec![],
            Some(NameTier::Past) => vec![],
            Some(NameTier::Pgo) => vec![],
            Some(NameTier::Captcha) => vec![],
            Some(NameTier::Alchemiter) => vec![],
            Some(NameTier::Denizen) => vec![asset.denizen_particles.clone()],
            Some(NameTier::Master) => asset.master_particles.to_vec(),
        };
        if !particles.is_empty() {
            let distance = 0.5;
            let num_instances = (mesh_text.bbox.size().x / distance).floor().max(1.0);
            let start_x = mesh_text.bbox.size().x * 0.5 - (num_instances - 1.0) * distance * 0.5;
            for i in 0..num_instances as usize {
                let particle = particles[i % particles.len()].clone();
                commands.spawn((
                    ParticleEffect::new(particle),
                    Transform::from_xyz(start_x + i as f32 * distance, 0.2, 0.0),
                    ChildOf(text_entity),
                ));
            }
        }

        if !matches!(name_tag.tier, Some(NameTier::Master)) {
            commands.entity(entity).observe(
                |trigger: Trigger<SceneInstanceReady>,
                 material_names: Query<&GltfMaterialName>,
                 mut commands: Commands,
                 children: Query<&Children>| {
                    for child in children.iter_descendants(trigger.target()).filter(|child| {
                        material_names
                            .get(*child)
                            .is_ok_and(|name| name.0 == "Candy")
                    }) {
                        commands.entity(child).insert(Visibility::Hidden);
                    }
                },
            );
        }

        commands
            .entity(entity)
            .remove::<SpawnNameTag>()
            .insert(NameTagged(name_tag));
    }

    Ok(())
}

#[add_system(
	plugin = NpcPlugin, schedule = Update,
	after = EntityKilledSet,
)]
fn add_killed_name_back(
    mut ev_killed: EventReader<EntityKilled>,
    mut names: ResMut<Assets<AvailableNames>>,
    assets: Res<NameTagAssets>,
    name_tagged: Query<&NameTagged>,
) -> Result {
    let names = names.get_mut(&assets.names).ok_or("Names not found")?;
    for ev in ev_killed.read() {
        if let Ok(name_tagged) = name_tagged.get(ev.0) {
            if name_tagged.0.tier.is_some() {
                names.names.push(name_tagged.0.clone());
            }
        }
    }
    Ok(())
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct CandyMaterial {}

impl Material for CandyMaterial {
    fn fragment_shader() -> ShaderRef {
        "candy shader.wgsl".into()
    }
}

enum NameTagShader {
    Standard(Handle<StandardMaterial>),
    Candy(Handle<CandyMaterial>),
}
