use std::f32::consts::PI;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy_auto_plugin::prelude::*;
use fake::Fake;
use fake::faker::name::en::FirstName;
use meshtext::{Face, MeshGenerator, MeshText, TextSection};
use rand::seq::{IndexedRandom as _, IteratorRandom};
use return_ok::some_or_return;

use crate::entity::Kill;
use crate::main_menu::{Supporter, SupporterTier, Supporters};
use crate::npcs::NpcPlugin;

// #[auto_add_plugin(plugin = NpcPlugin, generics(CandyMaterial>, init = MaterialPlugin::<CandyMaterial)::default())]
// use bevy::pbr::MaterialPlugin;

#[auto_resource(plugin = NpcPlugin, derive, reflect, register)]
pub struct NameTagAssets {
    pub names: Handle<Supporters>,

    pub generated_material: Handle<StandardMaterial>,
    pub past_material: Handle<StandardMaterial>,
    pub pgo_material: Handle<StandardMaterial>,
    pub captcha_material: Handle<StandardMaterial>,
    pub alchemiter_material: Handle<StandardMaterial>,
    pub denizen_materials: [Handle<StandardMaterial>; 4],
    pub master_material: Handle<StandardMaterial>,
    // #[allow(unused)]
    // pub denizen_particles: Handle<EffectAsset>,
    // #[allow(unused)]
    // pub denizen_particles_trails: Handle<EffectAsset>,
    // #[allow(unused)]
    // pub master_particles: [Handle<EffectAsset>; 2],
    // #[allow(unused)]
    // pub master_particles_trails: [Handle<EffectAsset>; 2],
}

#[auto_resource(plugin = NpcPlugin, derive, reflect, register)]
pub struct AvailableNames {
    names: Vec<NameTag>,
}

impl From<Supporters> for AvailableNames {
    fn from(contributors: Supporters) -> Self {
        Self {
            names: contributors.names.into_iter().map(NameTag::from).collect(),
        }
    }
}

/// Marks an entity that should have a name tag spawned for it.
/// This isn't an observer because the names might not be loaded when the entity is spawned.
#[auto_component(plugin = NpcPlugin, derive, reflect, register)]
pub struct SpawnNameTag;

#[auto_component(plugin = NpcPlugin, derive, reflect, register)]
pub struct NameTagged(pub NameTag);

#[derive(Debug, Clone, Reflect)]
pub struct NameTag {
    name: String,
    tier: Option<SupporterTier>,
}

impl Default for NameTag {
    fn default() -> Self {
        Self {
            name: FirstName().fake(),
            tier: None,
        }
    }
}

impl From<Supporter> for NameTag {
    fn from(contributor: Supporter) -> Self {
        Self {
            name: contributor.name,
            tier: Some(contributor.tier),
        }
    }
}

#[auto_resource(plugin = NpcPlugin, derive, init)]
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
            include_bytes!("../../assets/fonts/FiraSans-Regular.ttf"),
            include_bytes!("../../assets/fonts/FiraSans-Bold.ttf"),
        )
    }
}

/*
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
        */

#[auto_system(plugin = NpcPlugin, schedule = Startup)]
fn load_names(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // mut particles: ResMut<Assets<EffectAsset>>,
    // mut candy_materials: ResMut<Assets<CandyMaterial>>,
) -> Result {
    commands.insert_resource(NameTagAssets {
        names: asset_server.load("supporters.supporters.ron"),
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
        // master_material: candy_materials.add(CandyMaterial::default()),
        master_material: materials.add(Color::from(Srgba::hex("03a9f4")?)),
        // denizen_particles: particles.add(create_particles(Color::from(Srgba::hex("efbf04")?))),
        // denizen_particles_trails: particles
        //     .add(create_particles_trails(Color::from(Srgba::hex("efbf04")?))),
        // master_particles: [
        //     particles.add(create_particles(Color::from(Srgba::hex("ff0000")?))),
        //     particles.add(create_particles(Color::from(Srgba::hex("00ff00")?))),
        // ],
        // master_particles_trails: [
        //     particles.add(create_particles_trails(Color::from(Srgba::hex("ff0000")?))),
        //     particles.add(create_particles_trails(Color::from(Srgba::hex("00ff00")?))),
        // ],
    });

    Ok(())
}

#[auto_system(plugin = NpcPlugin, schedule = Update)]
fn spawn_name_tags(
    mut commands: Commands,
    asset: Res<NameTagAssets>,
    mut available_names: Option<ResMut<AvailableNames>>,
    entities: Query<Entity, With<SpawnNameTag>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut font_mesh_generator: ResMut<FontMeshGenerator>,
) -> Result {
    for entity in entities.iter() {
        let name_tag = get_name(available_names.as_mut()).unwrap_or_default();

        let (mesh_text, mesh) = match name_tag.tier {
            None => font_mesh_generator.generate_regular(&name_tag.name),
            Some(SupporterTier::Past) => font_mesh_generator.generate_regular(&name_tag.name),
            Some(SupporterTier::Pgo) => font_mesh_generator.generate_regular(&name_tag.name),
            Some(SupporterTier::Captcha) => font_mesh_generator.generate_regular(&name_tag.name),
            Some(SupporterTier::Alchemiter) => font_mesh_generator.generate_bold(&name_tag.name),
            Some(SupporterTier::Denizen) => font_mesh_generator.generate_regular(&name_tag.name),
            Some(SupporterTier::Master) => font_mesh_generator.generate_bold(&name_tag.name),
        };
        let material = match name_tag.tier {
            None => NameTagShader::Standard(asset.generated_material.clone()),
            Some(SupporterTier::Past) => NameTagShader::Standard(asset.past_material.clone()),
            Some(SupporterTier::Pgo) => NameTagShader::Standard(asset.pgo_material.clone()),
            Some(SupporterTier::Captcha) => NameTagShader::Standard(asset.captcha_material.clone()),
            Some(SupporterTier::Alchemiter) => {
                NameTagShader::Standard(asset.alchemiter_material.clone())
            }
            Some(SupporterTier::Denizen) => NameTagShader::Standard(
                asset
                    .denizen_materials
                    .choose(&mut rand::rng())
                    .ok_or("No denizen-level materials")?
                    .clone(),
            ),
            // Some(SupporterTier::Master) => NameTagShader::Candy(asset.master_material.clone()), // FIXME: custom materials broken
            Some(SupporterTier::Master) => NameTagShader::Standard(asset.master_material.clone()),
        };
        let scale = match name_tag.tier {
            None => 0.2,
            Some(SupporterTier::Past) => 0.2,
            Some(SupporterTier::Pgo) => 0.2,
            Some(SupporterTier::Captcha) => 0.2,
            Some(SupporterTier::Alchemiter) => 0.2,
            Some(SupporterTier::Denizen) => 0.3,
            Some(SupporterTier::Master) => 0.3,
        };

        let mut text_entity = commands.spawn((
            Mesh3d(meshes.add(mesh)),
            Transform::from_xyz(mesh_text.bbox.size().x * scale * 0.5, 1.1, 0.0)
                .with_rotation(Quat::from_rotation_y(PI))
                .with_scale(Vec3::splat(scale)),
            ChildOf(entity),
        ));
        match material {
            NameTagShader::Standard(material) => {
                text_entity.insert(MeshMaterial3d(material));
            } // NameTagShader::Candy(material) => {
              //     text_entity.insert(MeshMaterial3d(material));
              // }
        }
        // let text_entity = text_entity.id();

        // FIXME: idk why they dont work but it makes a bunch of errors. wait until release?
        // let (particles, particle_trails) = match name_tag.tier {
        //     None => (vec![], vec![]),
        //     Some(ContributorTier::Past) => (vec![], vec![]),
        //     Some(ContributorTier::Pgo) => (vec![], vec![]),
        //     Some(ContributorTier::Captcha) => (vec![], vec![]),
        //     Some(ContributorTier::Alchemiter) => (vec![], vec![]),
        //     Some(ContributorTier::Denizen) => (
        //         vec![asset.denizen_particles.clone()],
        //         vec![asset.denizen_particles_trails.clone()],
        //     ),
        //     Some(ContributorTier::Master) => (
        //         asset.master_particles.to_vec(),
        //         asset.master_particles_trails.to_vec(),
        //     ),
        // };
        // if !particles.is_empty() {
        //     let distance = 0.5;
        //     let num_instances = (mesh_text.bbox.size().x / distance).floor().max(1.0);
        //     let start_x = mesh_text.bbox.size().x * 0.5 - (num_instances - 1.0) * distance * 0.5;
        //     for i in 0..num_instances as usize {
        //         let particle = particles[i % particles.len()].clone();
        //         let particle_trail = particle_trails[i % particles.len()].clone();
        //         commands
        //             .spawn((
        //                 ParticleEffect::new(particle),
        //                 Transform::from_xyz(start_x + i as f32 * distance, 0.2, 0.0),
        //                 ChildOf(text_entity),
        //             ))
        //             .with_child((ParticleEffect::new(particle_trail),));
        //     }
        // }

        commands
            .entity(entity)
            .remove::<SpawnNameTag>()
            .insert(NameTagged(name_tag));
    }

    Ok(())
}

#[auto_component(plugin = NpcPlugin, derive, reflect, register)]
struct HasCandyMaterial;

fn get_name(available_names: Option<&mut ResMut<AvailableNames>>) -> Option<NameTag> {
    let available_names = available_names?;

    let opt = available_names
        .names
        .iter()
        .enumerate()
        .choose(&mut rand::rng())
        .map(|(i, name)| (i, name.clone()));
    if let Some((i, _)) = opt {
        available_names.names.swap_remove(i);
    }
    opt.map(|(_, name_tag)| name_tag)
}

#[auto_system(plugin = NpcPlugin, schedule = Update, config(
	run_if = not(resource_exists::<AvailableNames>),
))]
fn add_available_names(
    mut commands: Commands,
    assets: Res<NameTagAssets>,
    names: Res<Assets<Supporters>>,
) {
    let names = some_or_return!(names.get(&assets.names));
    commands.insert_resource(AvailableNames::from(names.clone()));
}

#[auto_observer(plugin = NpcPlugin)]
fn add_killed_name_back(
    kill: On<Kill>,
    mut names: Option<ResMut<AvailableNames>>,
    name_tagged: Query<&NameTagged>,
) -> Result {
    if let Ok(name_tagged) = name_tagged.get(kill.victim)
        && name_tagged.0.tier.is_some()
    {
        names
            .as_mut()
            .ok_or("Names not loaded (this should be impossible???)")?
            .names
            .push(name_tagged.0.clone());
    }

    Ok(())
}

// #[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
// pub struct CandyMaterial {}

// impl Material for CandyMaterial {
//     fn fragment_shader() -> ShaderRef {
//         "candy shader.wgsl".into()
//     }
// }

enum NameTagShader {
    Standard(Handle<StandardMaterial>),
    // Candy(Handle<CandyMaterial>),
}
