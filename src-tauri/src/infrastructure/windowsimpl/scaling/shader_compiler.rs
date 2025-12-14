use anyhow::{anyhow, Result};
use std::path::Path;

pub struct ShaderCompiler {
    base_path: std::path::PathBuf,
}

#[derive(Debug, Clone)]
struct EffectPassDesc {
    flags: u32,
    _block_size: (u32, u32),
    _num_threads: [u32; 3],
    _inputs: Vec<usize>,
    _outputs: Vec<usize>,
    _code: String,
}

#[derive(Debug, Clone)]
struct EffectTextureDesc {
    _name: String,
    _source: Option<String>,
    _format: String, // Simplified for now
}

#[derive(Debug, Clone)]
struct EffectDesc {
    _name: String,
    passes: Vec<EffectPassDesc>,
    _textures: Vec<EffectTextureDesc>,
    params: Vec<String>,    // Placeholder for param parsing
    _samplers: Vec<String>, // Placeholder
}

const EFFECT_PASS_FLAGS_PS_STYLE: u32 = 1 << 0;

impl ShaderCompiler {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    pub fn compile_compute_shader_source(&self, source_path: &Path) -> Result<(String, String)> {
        let source_content = std::fs::read_to_string(source_path)?;
        let desc = self.parse_effect(&source_content)?;

        // For now, we only support single pass PS style shaders for Bilinear/Bicubic
        if desc.passes.is_empty() {
            return Err(anyhow!("No passes found in effect"));
        }

        // Assume single pass for initial implementation
        let pass_idx = 0;
        let pass = &desc.passes[pass_idx];

        let mut result = String::new();

        // 1. StubDefs
        let stub_defs_path = self.base_path.join("StubDefs.hlsli");
        if stub_defs_path.exists() {
            result.push_str(&std::fs::read_to_string(stub_defs_path)?);
            result.push('\n');
        }

        // 2. Constants (Hardcoded for single pass PS style for now)
        // cbuffer definition is already in StubDefs.hlsli we modified?
        // Actually StubDefs has GlobalConstants.
        // We need to ensure consistency.

        // 3. Resources (Textures/Samplers)
        // Hardcoded for Bilinear/Bicubic which use INPUT/OUTPUT/sam
        result.push_str("Texture2D INPUT : register(t0);\n");
        result.push_str("RWTexture2D<unorm float4> OUTPUT : register(u0);\n"); // Assuming format
        result.push_str("SamplerState sam : register(s0);\n");

        if !desc.params.is_empty() {
            // Inject params buffer
            result.push_str("cbuffer Params : register(b1) {\n");
            for param in &desc.params {
                // Very naive validation/injection
                if let Some(line) = source_content
                    .lines()
                    .find(|l| l.contains(param) && l.contains("float"))
                {
                    result.push_str(line);
                    result.push_str(";\n");
                }
            }
            result.push_str("};\n");
        }

        // 4. Pass Code
        // Extract Pass1 function from source.
        // Naive extraction: copy everything that is not metadata
        let code_body = self.extract_code(&source_content);
        result.push_str(&code_body);

        // 5. Entry Point wrapper (__M)
        if (pass.flags & EFFECT_PASS_FLAGS_PS_STYLE) != 0 {
            let template = r#"
[numthreads(64, 1, 1)]
void __M(uint3 tid : SV_GroupThreadID, uint3 gid : SV_GroupID) {
    uint2 gxy = (gid.xy << 4u) + Rmp8x8(tid.x);
    if (gxy.x >= _outputSize.x || gxy.y >= _outputSize.y) {
        return;
    }
    float2 outputUV = (gxy + 0.5f) * _outputPt;
    float2 pos = outputUV * _scale + _srcRectOffset;
    float2 step = 8 * _outputPt * _scale; // Also scale step needed? No, step is for loop unrolling in Output Space.
    // Wait, step is used for pos.x += step.x.
    // pos is InputUV. step should be Input Step (Output Step * Scale).
    // Original: float2 step = 8 * _outputPt; -> This was delta OutputUV.
    // We want delta InputUV.
    // So yes, step = 8 * _outputPt * _scale.

    OUTPUT[gxy] = Pass1(pos);

    gxy.x += 8u;
    pos.x += step.x;
    if (gxy.x < _outputSize.x && gxy.y < _outputSize.y) {
        OUTPUT[gxy] = Pass1(pos);
    }
    
    gxy.y += 8u;
    pos.y += step.y;
    if (gxy.x < _outputSize.x && gxy.y < _outputSize.y) {
        OUTPUT[gxy] = Pass1(pos);
    }
    
    gxy.x -= 8u;
    pos.x -= step.x;
    if (gxy.x < _outputSize.x && gxy.y < _outputSize.y) {
        OUTPUT[gxy] = Pass1(pos);
    }
}
"#;
            result.push_str(template);
        }

        Ok((result, "__M".to_string()))
    }

    fn parse_effect(&self, source: &str) -> Result<EffectDesc> {
        // Limited parser for Bilinear/Bicubic style
        let mut passes = Vec::new();
        // Detect PS style pass
        if source.contains("//!STYLE PS") {
            passes.push(EffectPassDesc {
                flags: EFFECT_PASS_FLAGS_PS_STYLE,
                _block_size: (16, 16),
                _num_threads: [64, 1, 1],
                _inputs: vec![0],  // INPUT
                _outputs: vec![1], // OUTPUT
                _code: String::new(),
            });
        }

        let mut params = Vec::new();
        // Detect parameters like "float paramB;"
        for line in source.lines() {
            if line.trim().starts_with("float param") {
                if let Some(name) = line.split_whitespace().nth(1) {
                    let name = name.trim_matches(';');
                    params.push(name.to_string());
                }
            }
        }

        Ok(EffectDesc {
            _name: "Effect".to_string(),
            passes,
            _textures: vec![],
            params,
            _samplers: vec![],
        })
    }

    fn extract_code(&self, source: &str) -> String {
        let mut result = String::new();
        for line in source.lines() {
            if !line.trim().starts_with("//!") && !line.trim().starts_with("#include") {
                let trimmed = line.trim();
                if trimmed.starts_with("Texture2D INPUT")
                    || trimmed.starts_with("Texture2D OUTPUT")
                    || trimmed.starts_with("SamplerState sam")
                {
                    continue;
                }
                result.push_str(line);
                result.push('\n');
            }
        }
        result
    }
}
