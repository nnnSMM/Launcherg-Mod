use anyhow::{anyhow, Result};
use std::ffi::CString;
use std::path::Path;
use windows::{
    core::PCSTR,
    Win32::Graphics::Direct3D::Fxc::{
        D3DCompile, D3DCOMPILE_DEBUG, D3DCOMPILE_ENABLE_STRICTNESS,
        D3DCOMPILE_PACK_MATRIX_COLUMN_MAJOR,
    },
    Win32::Graphics::Direct3D11::{ID3D11ClassLinkage, ID3D11ComputeShader, ID3D11Device},
};

use crate::infrastructure::windowsimpl::scaling::shader_compiler::ShaderCompiler;

pub struct ShaderManager {
    device: ID3D11Device,
    compiler: ShaderCompiler,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MagpieConstants {
    pub input_size: [u32; 2],
    pub output_size: [u32; 2],
    pub input_pt: [f32; 2],
    pub output_pt: [f32; 2],
    pub scale: [f32; 2],
    pub src_rect_offset: [f32; 2], // Offset for pseudo-borderless (x, y)
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

    fn compile(&self, source: &str, entry_point: &str, target: &str) -> Result<Vec<u8>> {
        let entry_point_cstr = CString::new(entry_point)?;
        let target_cstr = CString::new(target)?;
        let source_cstr = CString::new(source)?;

        let mut flags = D3DCOMPILE_ENABLE_STRICTNESS | D3DCOMPILE_PACK_MATRIX_COLUMN_MAJOR;
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
