use crate::{model, object, texture, light};
use object::Object;

use std::io::{BufReader, Cursor};

use wgpu::util::DeviceExt;

pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(file_name)?;
    texture::Texture::from_bytes(device, queue, &data, file_name)
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let txt = std::fs::read_to_string(path)?;

    Ok(txt)
}

pub fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let data = std::fs::read(path)?;

    Ok(data)
}

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<crate::model::Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    ).await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        let diffuse_texture = load_texture(&m.diffuse_texture, device, queue).await?;

        materials.push(model::Material::new(device, &m.name, diffuse_texture, layout));
    }
    
    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| model::ModelVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [
                        m.mesh.texcoords[i * 2],
                        m.mesh.texcoords[i * 2 + 1],
                    ],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                    depth: 1000.0,
                }
            ).collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{} vertex buffer", file_name)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );

            let index_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{} index buffer", file_name)),
                    contents: bytemuck::cast_slice(&m.mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                }
            );
            model::ModelMesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        }
    ).collect::<Vec<_>>();

    Ok(crate::model::Model { meshes, materials })
}

pub async fn load_object(
    file_name: &str,
    device: &wgpu::Device,
    // next fields needs to be added if object need textures or materials
    // queue: &wgpu::Queue,
    // layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<Object> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);
    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            println!("Loading material: {}", p);
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    ).await?;

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| model::ModelVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [
                        m.mesh.texcoords[i * 2],
                        m.mesh.texcoords[i * 2 + 1],
                    ],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                    depth: 1000.0,
                }
            ).collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{} vertex buffer", file_name)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );

            let index_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{} index buffer", file_name)),
                    contents: bytemuck::cast_slice(&m.mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                }
            );
            object::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
            }
        }
    ).collect::<Vec<_>>();

    Ok(Object { meshes })
}

pub async fn load_light_obj(
    file_name: &str,
    device: &wgpu::Device,
    // next fields needs to be added if object need textures or materials
    // queue: &wgpu::Queue,
    // layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<Object> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);
    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            println!("Loading material: {}", p);
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    ).await?;

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| light::LightVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                }
            ).collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{} vertex buffer", file_name)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );

            let index_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{} index buffer", file_name)),
                    contents: bytemuck::cast_slice(&m.mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                }
            );
            object::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
            }
        }
    ).collect::<Vec<_>>();

    Ok(Object { meshes })
}