use anyhow::{anyhow, Result};
use std::ffi::CString;
use std::path::Path;
use windows::{
    core::PCSTR,
    Win32::Graphics::Direct3D::Fxc::{
        D3DCompile, D3DCOMPILE_DEBUG, D3DCOMPILE_ENABLE_STRICTNESS, D3DCOMPILE_OPTIMIZATION_LEVEL3,
        D3DCOMPILE_PACK_MATRIX_COLUMN_MAJOR,
    },
    Win32::Graphics::Direct3D11::{ID3D11ClassLinkage, ID3D11ComputeShader, ID3D11Device},
};

use crate::infrastructure::windowsimpl::scaling::effect_runtime::{
    CompiledEffect, CompiledPass, EffectCacheData, EffectTextureDesc,
};
use crate::infrastructure::windowsimpl::scaling::shader_compiler::ShaderCompiler;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};

const COMPILER_VERSION: u32 = 3;

pub struct ShaderManager {
    device: ID3D11Device,
    compiler: ShaderCompiler,
}

impl ShaderManager {
    pub fn new(device: ID3D11Device, resources_path: impl AsRef<Path>) -> Self {
        Self {
            device,
            compiler: ShaderCompiler::new(resources_path),
        }
    }

    // Unused methods removed

    pub fn compile_compute_shader(&self, shader_path: &Path) -> Result<ID3D11ComputeShader> {
        let (source, entry_point) = self.compiler.compile_compute_shader_source(shader_path)?;
        let bytecode = self.compile(&source, &entry_point, "cs_5_0")?;

        let mut shader = None;
        unsafe {
            self.device.CreateComputeShader(
                &bytecode,
                Option::<&ID3D11ClassLinkage>::None,
                Some(&mut shader),
            )?;
        }
        shader.ok_or_else(|| anyhow!("Failed to create compute shader"))
    }

    /// マルチパスエフェクトをコンパイル
    pub fn compile_effect(&self, shader_path: &Path) -> Result<CompiledEffect> {
        // キャッシュチェック
        let mut source_hash = 0;
        if let Ok(source_content) = fs::read(shader_path) {
            let mut hasher = DefaultHasher::new();
            source_content.hash(&mut hasher);
            source_hash = hasher.finish();
        }

        let cache_path = shader_path.with_extension("hlsl.compiled.json");

        if cache_path.exists() {
            if let Ok(cache_data) = self.load_cache(&cache_path) {
                if cache_data.source_hash == source_hash
                    && cache_data.compiler_version == COMPILER_VERSION
                {
                    println!("Loaded effect from cache: {:?}", cache_path);
                    return Ok(cache_data.effect);
                }
                println!("Cache mismatch or stale: {:?}", cache_path);
            }
        }

        let (desc, sources) = self.compiler.compile_all_passes(shader_path)?;

        let mut compiled_passes = Vec::new();
        for (i, source) in sources.iter().enumerate() {
            let bytecode = self
                .compile(source, "__M", "cs_5_0")
                .map_err(|e| anyhow!("Failed to compile pass {}: {}", i + 1, e))?;

            let pass_desc = &desc.passes[i];
            compiled_passes.push(CompiledPass {
                cso: bytecode,
                inputs: pass_desc.inputs.clone(),
                outputs: pass_desc.outputs.clone(),
                block_size: pass_desc.block_size,
                num_threads: pass_desc.num_threads,
                is_ps_style: pass_desc.is_ps_style(),
                desc: pass_desc.desc.clone(),
            });
        }

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

        Ok(effect)
    }

    fn load_cache(&self, path: &Path) -> Result<EffectCacheData> {
        let file = fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let data = serde_json::from_reader(reader)?;
        Ok(data)
    }

    fn save_cache(&self, path: &Path, data: &EffectCacheData) -> Result<()> {
        let file = fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer(writer, data)?;
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
