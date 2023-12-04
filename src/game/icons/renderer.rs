use bevy::core_pipeline::core_2d::Transparent2d;
use bevy::ecs::query::QueryItem;
use bevy::ecs::query::ROQueryItem;
use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::lifetimeless::*;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::mesh::{GpuBufferInfo, MeshVertexBufferLayout};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::AddRenderCommand;
use bevy::render::render_phase::{
    DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, RenderPhase, SetItemPipeline,
    TrackedRenderPass,
};

use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::view::{ExtractedView, VisibleEntities};
use bevy::render::{Render, RenderApp, RenderSet};
use bevy::sprite::{
    Mesh2dPipeline, Mesh2dPipelineKey, RenderMesh2dInstance, RenderMesh2dInstances,
    SetMesh2dBindGroup, SetMesh2dViewBindGroup,
};
use bevy::utils::FloatOrd;

use super::IconInstanceData;

#[derive(Component, Debug)]
pub struct GpuIconInstanceData {
    pub data: Vec<u8>,
    pub n_instances: usize,
    pub texture: Handle<Image>,
}

impl ExtractComponent for IconInstanceData {
    type Query = &'static IconInstanceData;
    type Filter = ();
    type Out = GpuIconInstanceData;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<GpuIconInstanceData> {
        Some(GpuIconInstanceData {
            data: item.instances_data(),
            n_instances: item.n_instances as usize,
            texture: item.texture.clone(),
        })
    }
}

pub struct IconRendererPlugin;

impl Plugin for IconRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<IconInstanceData>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent2d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .init_resource::<TextureBindGroup>()
            .add_systems(
                Render,
                (
                    queue_custom.in_set(RenderSet::QueueMeshes),
                    prepare_instance_buffers.in_set(RenderSet::PrepareResources),
                    prepare_texture_bind_group.in_set(RenderSet::PrepareBindGroups),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).init_resource::<CustomPipeline>();
    }
}

/// Queue the 2d meshes marked with [`ColoredMesh2d`] using our custom pipeline and draw function
#[allow(clippy::too_many_arguments)]
pub fn queue_custom(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    colored_mesh2d_pipeline: Res<CustomPipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CustomPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    msaa: Res<Msaa>,
    meshes: Res<RenderAssets<Mesh>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    render_mesh_instances: Res<RenderMesh2dInstances>,
    texture_bind_group: Res<TextureBindGroup>,
    mut views: Query<(
        &VisibleEntities,
        &mut RenderPhase<Transparent2d>,
        &ExtractedView,
    )>,
) {
    if render_mesh_instances.is_empty() {
        return;
    }
    if texture_bind_group.bind_group.is_none() {
        return;
    }
    // Iterate each view (a camera is a view)
    for (visible_entities, mut transparent_phase, view) in &mut views {
        let draw_colored_mesh2d = transparent_draw_functions.read().id::<DrawCustom>();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples())
            | Mesh2dPipelineKey::from_hdr(view.hdr);

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            if let Some(mesh_instance) = render_mesh_instances.get(visible_entity) {
                let mesh2d_handle = mesh_instance.mesh_asset_id;
                let mesh2d_transforms = &mesh_instance.transforms;
                let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                    continue;
                };
                // Get our specialized pipeline
                let mut mesh2d_key = mesh_key;
                if let Some(mesh) = render_meshes.get(mesh2d_handle) {
                    mesh2d_key |=
                        Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);
                }

                let pipeline_id = pipelines
                    .specialize(
                        &pipeline_cache,
                        &colored_mesh2d_pipeline,
                        mesh2d_key,
                        &mesh.layout,
                    )
                    .unwrap();

                let mesh_z = mesh2d_transforms.transform.translation.z;
                transparent_phase.add(Transparent2d {
                    entity: *visible_entity,
                    draw_function: draw_colored_mesh2d,
                    pipeline: pipeline_id,
                    // The 2d render items are sorted according to their z value before rendering,
                    // in order to get correct transparency
                    sort_key: FloatOrd(mesh_z),
                    // This material is not batched
                    batch_range: 0..1,
                    dynamic_offset: None,
                });
            }
        }
    }
}

#[derive(Component)]
pub struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &GpuIconInstanceData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: &instance_data.data,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.n_instances,
        });
    }
}

#[derive(Resource, Default)]
pub struct TextureBindGroup {
    pub bind_group: Option<BindGroup>,
}

pub fn prepare_texture_bind_group(
    render_device: Res<RenderDevice>,
    instance: Query<Option<&GpuIconInstanceData>>,
    images: Res<RenderAssets<Image>>,
    pipeline: Res<CustomPipeline>,
    mut texture_bind_group: ResMut<TextureBindGroup>,
) {
    for instance in instance.iter().flatten() {
        let gpu_image = images.get(instance.texture.id()).unwrap();
        let bind_group = render_device.create_bind_group(
            "icon_texture_bind_group",
            &pipeline.texture_bind_group_layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&gpu_image.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&gpu_image.sampler),
                },
            ],
        );
        texture_bind_group.bind_group = Some(bind_group);
    }
}

#[derive(Resource)]
pub struct CustomPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: Mesh2dPipeline,
    texture_bind_group_layout: BindGroupLayout,
}

impl FromWorld for CustomPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shader.wgsl");

        let render_device = world.resource::<RenderDevice>();
        let texture_uniform_layout_entry = BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                multisampled: false,
                view_dimension: TextureViewDimension::D2Array,
                sample_type: TextureSampleType::Float { filterable: true },
            },
            count: None,
        };
        let texture_sampler_layout_entry = BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        };
        let texture_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[texture_uniform_layout_entry, texture_sampler_layout_entry],
                label: Some("icons_texture_array"),
            });

        CustomPipeline {
            shader,
            mesh_pipeline: Mesh2dPipeline::from_world(world),
            texture_bind_group_layout,
        }
    }
}

impl SpecializedMeshPipeline for CustomPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

        descriptor
            .vertex
            .shader_defs
            .push("MESH_BINDGROUP_1".into());

        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: IconInstanceData::INSTANCE_LEN, // std::mem::size_of::<InstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                // transform (x and y offset + rotation angle)
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 3, // shader locations 0-2 are taken up by Position, Normal and UV attributes
                },
                // icon sheet index (index in 2d texture array)
                VertexAttribute {
                    format: VertexFormat::Uint32,
                    offset: (3 * 4),
                    shader_location: 4, // shader locations 0-2 are taken up by Position, Normal and UV attributes
                },
                // uv coordinate offset in icon sheet
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: (3 * 4 + 4),
                    shader_location: 5, // shader locations 0-2 are taken up by Position, Normal and UV attributes
                },
            ],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();

        descriptor
            .layout
            .push(self.texture_bind_group_layout.clone());

        // println!("Descriptor: {:#?}", descriptor);

        Ok(descriptor)
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetMesh2dViewBindGroup<0>,
    SetMesh2dBindGroup<1>,
    SetTextureBindGroup<2>,
    DrawMesh2dInstanced,
);

pub struct SetTextureBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetTextureBindGroup<I> {
    type Param = SRes<TextureBindGroup>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        _item_query: ROQueryItem<'w, Self::ItemWorldQuery>,
        texture_bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let texture_bind_group = texture_bind_group.into_inner();
        if let Some(bind_group) = &texture_bind_group.bind_group {
            pass.set_bind_group(I, bind_group, &[]);
        }
        RenderCommandResult::Success
    }
}

pub struct DrawMesh2dInstanced;

impl<P: PhaseItem> RenderCommand<P> for DrawMesh2dInstanced {
    type Param = (SRes<RenderAssets<Mesh>>, SRes<RenderMesh2dInstances>);
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<InstanceBuffer>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        instance_buffer: &'w InstanceBuffer,
        (meshes, render_mesh2d_instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let meshes = meshes.into_inner();
        let render_mesh2d_instances = render_mesh2d_instances.into_inner();

        let Some(RenderMesh2dInstance { mesh_asset_id, .. }) =
            render_mesh2d_instances.get(&item.entity())
        else {
            return RenderCommandResult::Failure;
        };
        let Some(gpu_mesh) = meshes.get(*mesh_asset_id) else {
            return RenderCommandResult::Failure;
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
            }
            GpuBufferInfo::NonIndexed => {
                pass.draw(0..gpu_mesh.vertex_count, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}
