use anyhow::{anyhow, Result};
use serde::Serialize;
use std::ffi::CString;
use std::path::Path;
use windows::{
    core::PCSTR,
    Win32::Graphics::Direct3D::Fxc::{
        D3DCompile, D3DCOMPILE_DEBUG, D3DCOMPILE_ENABLE_STRICTNESS, D3DCOMPILE_OPTIMIZATION_LEVEL3,
        D3DCOMPILE_PACK_MATRIX_COLUMN_MAJOR,
    },
    Win32::Graphics::Direct3D11::ID3D11Texture2D,
};

use crate::infrastructure::windowsimpl::scaling::effect_runtime::{
    CompiledEffect, CompiledPass, EffectCacheData, EffectTextureDesc,
};
use crate::infrastructure::windowsimpl::scaling::shader_compiler::ShaderCompiler;
use once_cell::sync::Lazy;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Mutex;

use rayon::prelude::*;

const COMPILER_VERSION: u32 = 4;

static MEMORY_CACHE: Lazy<Mutex<HashMap<PathBuf, (u64, CompiledEffect)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub struct ShaderManager {
    compiler: ShaderCompiler,
}

impl ShaderManager {
    pub fn new(resources_path: impl AsRef<Path>) -> Self {
        Self {
            compiler: ShaderCompiler::new(resources_path),
        }
    }

    pub fn list_available_shaders(&self, root_path: &Path) -> Result<Vec<ShaderOption>> {
        self.list_shaders_recursive(root_path, root_path)
    }

    fn list_shaders_recursive(
        &self,
        current_path: &Path,
        root_path: &Path,
    ) -> Result<Vec<ShaderOption>> {
        let mut options = Vec::new();
        if !current_path.exists() {
            return Ok(options);
        }

        for entry in fs::read_dir(current_path)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if path.is_dir() {
                let children = self.list_shaders_recursive(&path, root_path)?;
                if !children.is_empty() {
                    let rel_path = path
                        .strip_prefix(root_path)?
                        .to_string_lossy()
                        .replace("\\", "/");
                    options.push(ShaderOption {
                        label: name.to_string(),
                        value: rel_path,
                        children: Some(children),
                    });
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("hlsl") {
                if name == "StandardVS.hlsl" {
                    continue;
                }
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let rel_path = path
                    .strip_prefix(root_path)?
                    .to_string_lossy()
                    .replace("\\", "/");
                let value = rel_path.trim_end_matches(".hlsl").to_string();

                // Get metadata if possible
                let label = if let Ok(desc) = self.compiler.parse_effect_public(&path) {
                    if !desc.sort_name.is_empty() {
                        desc.sort_name.clone()
                    } else {
                        stem.to_string()
                    }
                } else {
                    stem.to_string()
                };

                options.push(ShaderOption {
                    label,
                    value,
                    children: None,
                });
            }
        }

        // Sort: directories first, then files, both alphabetically
        options.sort_by(|a, b| {
            let a_is_dir = a.children.is_some();
            let b_is_dir = b.children.is_some();
            if a_is_dir != b_is_dir {
                b_is_dir.cmp(&a_is_dir)
            } else {
                a.label.cmp(&b.label)
            }
        });

        Ok(options)
    }

    // compile_compute_shader removed as it required device and was unused.

    /// マルチパスエフェクトをコンパイル
    pub fn compile_effect(&self, shader_path: &Path) -> Result<CompiledEffect> {
        // キャッシュチェック
        let mut source_hash = 0;
        if let Ok(source_content) = fs::read(shader_path) {
            let mut hasher = DefaultHasher::new();
            source_content.hash(&mut hasher);
            source_hash = hasher.finish();
        }

        // メモリキャッシュチェック
        {
            let cache = MEMORY_CACHE.lock().unwrap();
            if let Some((cached_hash, effect)) = cache.get(shader_path) {
                if *cached_hash == source_hash {
                    println!("Loaded effect from memory cache: {:?}", shader_path);
                    return Ok(effect.clone());
                }
            }
        }

        let cache_path = shader_path.with_extension("hlsl.compiled.bin");

        if cache_path.exists() {
            if let Ok(cache_data) = self.load_cache(&cache_path) {
                if cache_data.source_hash == source_hash
                    && cache_data.compiler_version == COMPILER_VERSION
                {
                    println!("Loaded effect from cache: {:?}", cache_path);

                    // メモリキャッシュにも保存
                    {
                        let mut cache = MEMORY_CACHE.lock().unwrap();
                        cache.insert(
                            shader_path.to_path_buf(),
                            (source_hash, cache_data.effect.clone()),
                        );
                    }

                    return Ok(cache_data.effect);
                }
                println!("Cache mismatch or stale: {:?}", cache_path);
            }
        }

        let (desc, sources) = self.compiler.compile_all_passes(shader_path)?;

        // 並列コンパイル
        let compiled_results: Result<Vec<_>> = sources
            .par_iter()
            .enumerate()
            .map(|(i, source)| {
                let bytecode = self
                    .compile(source, "__M", "cs_5_0")
                    .map_err(|e| anyhow!("Failed to compile pass {}: {}", i + 1, e))?;

                // Note: desc is shared reference, safe to access
                let pass_desc = &desc.passes[i];
                Ok(CompiledPass {
                    cso: bytecode,
                    inputs: pass_desc.inputs.clone(),
                    outputs: pass_desc.outputs.clone(),
                    block_size: pass_desc.block_size,
                    num_threads: pass_desc.num_threads,
                    is_ps_style: pass_desc.is_ps_style(),
                    desc: pass_desc.desc.clone(),
                })
            })
            .collect();

        let compiled_passes = compiled_results?;

        // テクスチャ記述子を変換
        let textures: Vec<EffectTextureDesc> = desc
            .textures
            .iter()
            .map(|t| EffectTextureDesc {
                name: t.name.clone(),
                format: t.format,
                width_expr: t.size_expr.0.clone(),
                height_expr: t.size_expr.1.clone(),
                source: t.source.clone(),
                is_input: t.name == "INPUT",
                is_output: t.name == "OUTPUT",
            })
            .collect();

        let effect = CompiledEffect {
            name: desc.name.clone(),
            passes: compiled_passes,
            textures,
            samplers: desc.samplers.clone(),
            params: desc.params.clone(),
            flags: desc.flags,
        };

        // キャッシュ保存
        let cache_data = EffectCacheData {
            source_hash,
            compiler_version: COMPILER_VERSION,
            effect: effect.clone(),
        };
        if let Err(e) = self.save_cache(&cache_path, &cache_data) {
            println!("Failed to save cache: {:?}", e);
        }

        // メモリキャッシュに保存
        {
            let mut cache = MEMORY_CACHE.lock().unwrap();
            cache.insert(shader_path.to_path_buf(), (source_hash, effect.clone()));
        }

        Ok(effect)
    }

    fn load_cache(&self, path: &Path) -> Result<EffectCacheData> {
        let file = fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let data = bincode::deserialize_from(reader)?;
        Ok(data)
    }

    fn save_cache(&self, path: &Path, data: &EffectCacheData) -> Result<()> {
        let file = fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        bincode::serialize_into(writer, data)?;
        Ok(())
    }

    fn compile(&self, source: &str, entry_point: &str, target: &str) -> Result<Vec<u8>> {
        let entry_point_cstr = CString::new(entry_point)?;
        let target_cstr = CString::new(target)?;
        let source_cstr = CString::new(source)?;

        let mut flags = D3DCOMPILE_ENABLE_STRICTNESS
            | D3DCOMPILE_OPTIMIZATION_LEVEL3
            | D3DCOMPILE_PACK_MATRIX_COLUMN_MAJOR;
        if cfg!(debug_assertions) {
            flags |= D3DCOMPILE_DEBUG;
        }

        let mut code_blob = None;
        let mut error_blob = None;

        unsafe {
            let result = D3DCompile(
                source_cstr.as_ptr() as *const _,
                source.len(),
                PCSTR::null(),
                None, // Defines
                None, // Includes - might need to implement this for StubDefs.hlsli
                PCSTR(entry_point_cstr.as_ptr() as *const _),
                PCSTR(target_cstr.as_ptr() as *const _),
                flags,
                0,
                &mut code_blob,
                Some(&mut error_blob),
            );

            if let Err(e) = result {
                if let Some(error_blob) = error_blob {
                    let buffer_ptr = error_blob.GetBufferPointer();
                    let buffer_size = error_blob.GetBufferSize();
                    let slice = std::slice::from_raw_parts(buffer_ptr as *const u8, buffer_size);
                    let error_msg = String::from_utf8_lossy(slice);
                    return Err(anyhow!("Shader compilation failed: {}\n{}", e, error_msg));
                }
                return Err(anyhow!("Shader compilation failed: {}", e));
            }

            let code_blob = code_blob.ok_or_else(|| anyhow!("No code blob returned"))?;
            let slice = std::slice::from_raw_parts(
                code_blob.GetBufferPointer() as *const u8,
                code_blob.GetBufferSize(),
            );
            Ok(slice.to_vec())
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShaderOption {
    pub label: String,
    pub value: String,
    pub children: Option<Vec<ShaderOption>>,
}
