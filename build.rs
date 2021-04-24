use anyhow::Result;
use gltf::mesh::Reader;
use gltf::Buffer;
use parry3d::math::Point;
use parry3d::transformation::vhacd::VHACD;
use rayon::prelude::*;
use sha2::Digest;
use std::array::IntoIter;
use std::collections::HashMap;
use std::convert::TryInto;
use std::path::{Component, Path, PathBuf};

fn get_files(path: &Path, files: &mut Vec<PathBuf>, filter_glb: bool) -> std::io::Result<()> {
    for file in std::fs::read_dir(path)?.filter_map(|f| f.ok()) {
        if file.file_type()?.is_dir() {
            get_files(&file.path(), files, filter_glb)?;
        }
        if file.file_type()?.is_file() {
            let path = file.path();
            if !filter_glb || path.extension().map_or(false, |ext| ext == "glb") {
                files.push(path);
            }
        }
    }
    Ok(())
}

fn get_vhacd<'a, 's, F>(reader: Reader<'a, 's, F>) -> Option<Vec<(Vec<Point<f32>>, Vec<[u32; 3]>)>>
where
    F: Clone + Fn(Buffer<'a>) -> Option<&'s [u8]>,
{
    let pos: Vec<_> = reader.read_positions()?.map(Point::from).collect();
    let mut index_reader = reader.read_indices()?.into_u32();
    let mut indices = Vec::with_capacity(index_reader.len());
    while let (Some(a), Some(b), Some(c)) = (
        index_reader.next(),
        index_reader.next(),
        index_reader.next(),
    ) {
        indices.push([a, b, c]);
    }
    let res = VHACD::decompose(&Default::default(), &pos, &indices, true)
        .compute_exact_convex_hulls(&pos, &indices);
    Some(res)
}

fn main() -> Result<()> {
    let assets_modified = std::fs::metadata("assets")?.modified()?;
    let _ = std::fs::remove_dir_all("assets/vhacd");
    let mut paths = vec![];
    get_files("assets".as_ref(), &mut paths, true)?;

    std::fs::create_dir("assets/vhacd")?;
    paths.par_iter().try_for_each(|file| -> Result<()> {
        let (gltf, buffers, _) = gltf::import(&file)?;
        let get_buffer_data = |buffer: gltf::Buffer| buffers.get(buffer.index()).map(|x| &*x.0);
        let mut map = HashMap::new();

        for mesh in gltf.meshes() {
            for primitive in mesh.primitives() {
                let name = format!("Mesh{}/Primitive{}", mesh.index(), primitive.index());
                let reader = primitive.reader(get_buffer_data);
                if let Some(mesh) = get_vhacd(reader) {
                    map.insert(name, mesh);
                }
            }
        }

        let mut new_name: PathBuf = IntoIter::new([
            Component::Normal("assets".as_ref()),
            Component::Normal("vhacd".as_ref()),
        ])
        .chain(file.components().skip(1))
        .collect();

        new_name.set_extension("custom");
        std::fs::write(&new_name, &postcard::to_stdvec(&map)?)?;
        filetime::set_file_mtime(&new_name, assets_modified.into())?;
        Ok(())
    })?;
    filetime::set_file_mtime("assets/vhacd", assets_modified.into())?;
    filetime::set_file_mtime("assets", assets_modified.into())?;
    println!("cargo:rerun-if-changed=assets/");

    if std::env::var("CARGO_CFG_TARGET_ARCH").map_or(false, |s| s == "wasm32") {
        let mut paths = vec![];
        get_files("assets".as_ref(), &mut paths, false)?;
        let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
        let assets = out_dir.join("assets");
        let _ = std::fs::remove_dir_all(&assets);
        std::fs::create_dir(assets)?;
        let _ = std::fs::remove_dir_all("web/static/assets");
        std::fs::create_dir_all("web/static/assets")?;
        for path in &paths {
            let asset = out_dir.join(path);
            let new_name = {
                let mut components = path.components();
                // remove the assets/
                components.next();
                let mut sha = sha2::Sha256::default();
                std::io::copy(&mut std::fs::File::open(path)?, &mut sha)?;
                let mut p = components.as_path().to_owned();
                let mut file_name = p.file_stem().unwrap_or_default().to_owned();
                let file_ext = p.extension().map(ToOwned::to_owned);
                let hash = u32::from_be_bytes(sha.finalize()[..4].try_into().unwrap());
                file_name.push(format!("-{:x}", hash));
                p.set_file_name(file_name);
                if let Some(ext) = file_ext {
                    p.set_extension(ext);
                }
                p
            };
            let web_asset = Path::new("web/static/assets").join(new_name.as_path());
            std::fs::create_dir_all(asset.parent().unwrap())?;
            std::fs::create_dir_all(web_asset.parent().unwrap())?;
            std::fs::write(asset, new_name.as_path().to_str().unwrap())?;
            std::fs::copy(path, web_asset)?;
        }
    }
    Ok(())
}
