//! シェーダーコンパイラ
//! Magpie の EffectCompiler.cpp を Rust で再実装

use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::path::Path;

use super::effect_desc::*;
use super::include_handler::IncludeHandler;

/// MagpieFX バージョン
const MAGPIE_FX_VERSION: u32 = 4;

/// メタデータ指示子
const META_INDICATOR: &str = "//!";

/// シェーダーコンパイラ
pub struct ShaderCompiler {
    base_path: std::path::PathBuf,
}

/// ブロックタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockType {
    Header,
    Parameter,
    Texture,
    Sampler,
    Common,
    Pass,
}

impl ShaderCompiler {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// コンピュートシェーダーをコンパイル
    pub fn compile_compute_shader_source(&self, source_path: &Path) -> Result<(String, String)> {
        let source_content = std::fs::read_to_string(source_path)?;

        // インクルードを展開
        let parent_dir = source_path.parent().unwrap_or(Path::new("."));
        let mut include_handler = IncludeHandler::new(parent_dir);
        let expanded_source = include_handler.expand_includes(&source_content)?;

        // エフェクトをパース
        let effect_name = source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Effect");

        let desc = self.parse_effect(effect_name, &expanded_source)?;

        if desc.passes.is_empty() {
            return Err(anyhow!("No passes found in effect"));
        }

        // 最初のパスのソースを生成
        let (source, _macros) = self.generate_pass_source(&desc, 1, &expanded_source)?;

        Ok((source, "__M".to_string()))
    }

    /// 全パスのソースを生成（マルチパスエフェクト用）
    pub fn compile_all_passes(&self, source_path: &Path) -> Result<(EffectDesc, Vec<String>)> {
        let source_content = std::fs::read_to_string(source_path)?;

        // インクルードを展開
        let parent_dir = source_path.parent().unwrap_or(Path::new("."));
        let mut include_handler = IncludeHandler::new(parent_dir);
        let expanded_source = include_handler.expand_includes(&source_content)?;

        // エフェクトをパース
        let effect_name = source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Effect");

        let desc = self.parse_effect(effect_name, &expanded_source)?;

        if desc.passes.is_empty() {
            return Err(anyhow!("No passes found in effect"));
        }

        // 全パスのソースを生成
        let mut sources = Vec::new();
        for pass_idx in 1..=desc.passes.len() {
            let (source, _macros) = self.generate_pass_source(&desc, pass_idx, &expanded_source)?;
            sources.push(source);
        }

        Ok((desc, sources))
    }

    /// エフェクトをパース（公開版）
    pub fn parse_effect_public(&self, source_path: &Path) -> Result<EffectDesc> {
        let source_content = std::fs::read_to_string(source_path)?;

        // インクルードを展開
        let parent_dir = source_path.parent().unwrap_or(Path::new("."));
        let mut include_handler = IncludeHandler::new(parent_dir);
        let expanded_source = include_handler.expand_includes(&source_content)?;

        // エフェクトをパース
        let effect_name = source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Effect");

        self.parse_effect(effect_name, &expanded_source)
    }

    /// エフェクトをパース
    fn parse_effect(&self, name: &str, source: &str) -> Result<EffectDesc> {
        // コメントを除去
        let source = self.remove_comments(source)?;

        // マジックをチェック
        let source_view = source.trim();
        if !self.check_magic(source_view) {
            return Err(anyhow!("Invalid MagpieFX header"));
        }

        // ブロックに分割
        let (
            header_block,
            param_blocks,
            texture_blocks,
            sampler_blocks,
            common_blocks,
            pass_blocks,
        ) = self.split_blocks(&source)?;

        let mut desc = EffectDesc::new(name);

        // ヘッダーを解析
        self.resolve_header(&header_block, &mut desc)?;

        // パラメータを解析
        for (i, block) in param_blocks.iter().enumerate() {
            self.resolve_parameter(block, &mut desc)
                .map_err(|e| anyhow!("Failed to parse Parameter#{}: {}", i + 1, e))?;
        }

        // テクスチャを解析 (INPUT と OUTPUT は既に追加済み)
        for (i, block) in texture_blocks.iter().enumerate() {
            self.resolve_texture(block, &mut desc)
                .map_err(|e| anyhow!("Failed to parse Texture#{}: {}", i + 1, e))?;
        }

        // サンプラーを解析
        for (i, block) in sampler_blocks.iter().enumerate() {
            self.resolve_sampler(block, &mut desc)
                .map_err(|e| anyhow!("Failed to parse Sampler#{}: {}", i + 1, e))?;
        }

        // パスを解析
        self.resolve_passes(&pass_blocks, &common_blocks, &mut desc)?;

        // 名前の重複チェック
        self.check_duplicate_names(&desc)?;

        Ok(desc)
    }

    /// コメントを除去 (//! は保持)
    fn remove_comments(&self, source: &str) -> Result<String> {
        let mut result = String::with_capacity(source.len());
        let chars: Vec<char> = source.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if i + 1 < chars.len() && chars[i] == '/' {
                if chars[i + 1] == '/' {
                    // //! は保持
                    if i + 2 < chars.len() && chars[i + 2] == '!' {
                        // //! の行全体を保持
                        while i < chars.len() && chars[i] != '\n' {
                            result.push(chars[i]);
                            i += 1;
                        }
                    } else {
                        // 通常の行コメント - スキップ
                        while i < chars.len() && chars[i] != '\n' {
                            i += 1;
                        }
                        result.push('\n');
                    }
                    continue;
                } else if chars[i + 1] == '*' {
                    // ブロックコメント
                    i += 2;
                    while i + 1 < chars.len() {
                        if chars[i] == '*' && chars[i + 1] == '/' {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                    continue;
                }
            }
            result.push(chars[i]);
            i += 1;
        }

        // 末尾に改行を追加
        if !result.ends_with('\n') {
            result.push('\n');
        }

        Ok(result)
    }

    /// マジックをチェック
    fn check_magic(&self, source: &str) -> bool {
        let trimmed = source.trim_start();
        if !trimmed.starts_with(META_INDICATOR) {
            return false;
        }
        let after_indicator = trimmed[META_INDICATOR.len()..].trim_start();
        after_indicator.starts_with("MAGPIE")
            && after_indicator[6..].trim_start().starts_with("EFFECT")
    }

    /// ソースをブロックに分割
    fn split_blocks(
        &self,
        source: &str,
    ) -> Result<(
        String,
        Vec<String>,
        Vec<String>,
        Vec<String>,
        Vec<String>,
        Vec<String>,
    )> {
        let mut header_block = String::new();
        let mut param_blocks = Vec::new();
        let mut texture_blocks = Vec::new();
        let mut sampler_blocks = Vec::new();
        let mut common_blocks = Vec::new();
        let mut pass_blocks = Vec::new();

        let mut current_block = String::new();
        let mut current_type = BlockType::Header;

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with(META_INDICATOR) {
                let after = trimmed[META_INDICATOR.len()..].trim();
                let block_type = after.split_whitespace().next().map(|s| s.to_uppercase());

                let new_type = match block_type.as_deref() {
                    Some("PARAMETER") => Some(BlockType::Parameter),
                    Some("TEXTURE") => Some(BlockType::Texture),
                    Some("SAMPLER") => Some(BlockType::Sampler),
                    Some("COMMON") => Some(BlockType::Common),
                    Some("PASS") => Some(BlockType::Pass),
                    _ => None,
                };

                if let Some(new_type) = new_type {
                    // 現在のブロックを保存
                    self.save_block(
                        &mut header_block,
                        &mut param_blocks,
                        &mut texture_blocks,
                        &mut sampler_blocks,
                        &mut common_blocks,
                        &mut pass_blocks,
                        current_type,
                        std::mem::take(&mut current_block),
                    );
                    current_type = new_type;
                }
            }

            current_block.push_str(line);
            current_block.push('\n');
        }

        // 最後のブロックを保存
        self.save_block(
            &mut header_block,
            &mut param_blocks,
            &mut texture_blocks,
            &mut sampler_blocks,
            &mut common_blocks,
            &mut pass_blocks,
            current_type,
            current_block,
        );

        Ok((
            header_block,
            param_blocks,
            texture_blocks,
            sampler_blocks,
            common_blocks,
            pass_blocks,
        ))
    }

    fn save_block(
        &self,
        header: &mut String,
        params: &mut Vec<String>,
        textures: &mut Vec<String>,
        samplers: &mut Vec<String>,
        commons: &mut Vec<String>,
        passes: &mut Vec<String>,
        block_type: BlockType,
        content: String,
    ) {
        if content.trim().is_empty() {
            return;
        }
        match block_type {
            BlockType::Header => *header = content,
            BlockType::Parameter => params.push(content),
            BlockType::Texture => textures.push(content),
            BlockType::Sampler => samplers.push(content),
            BlockType::Common => commons.push(content),
            BlockType::Pass => passes.push(content),
        }
    }

    /// ヘッダーを解析
    fn resolve_header(&self, block: &str, desc: &mut EffectDesc) -> Result<()> {
        let mut version_found = false;

        for line in block.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with(META_INDICATOR) {
                continue;
            }

            let after = trimmed[META_INDICATOR.len()..].trim();
            let mut parts = after.split_whitespace();
            let directive = parts.next().map(|s| s.to_uppercase());

            match directive.as_deref() {
                Some("VERSION") => {
                    let version: u32 = parts
                        .next()
                        .ok_or_else(|| anyhow!("VERSION requires a number"))?
                        .parse()
                        .map_err(|_| anyhow!("Invalid VERSION number"))?;

                    if version != MAGPIE_FX_VERSION {
                        return Err(anyhow!("Unsupported MagpieFX version: {}", version));
                    }
                    version_found = true;
                }
                Some("SORT_NAME") => {
                    desc.sort_name = parts.collect::<Vec<_>>().join(" ");
                }
                Some("USE") => {
                    let flags_str = parts.collect::<Vec<_>>().join("");
                    for flag in flags_str.split(',') {
                        match flag.trim().to_uppercase().as_str() {
                            "MULADD" => desc.flags |= effect_flags::USE_MUL_ADD,
                            "_DYNAMIC" => desc.flags |= effect_flags::USE_DYNAMIC,
                            _ => {}
                        }
                    }
                }
                Some("CAPABILITY") => {
                    let flags_str = parts.collect::<Vec<_>>().join("");
                    for flag in flags_str.split(',') {
                        if flag.trim().to_uppercase() == "FP16" {
                            desc.flags |= effect_flags::SUPPORT_FP16;
                        }
                    }
                }
                _ => {}
            }
        }

        if !version_found {
            return Err(anyhow!("VERSION is required in header"));
        }

        Ok(())
    }

    /// パラメータを解析
    fn resolve_parameter(&self, block: &str, desc: &mut EffectDesc) -> Result<()> {
        let mut param = EffectParameterDesc::default();
        let mut default_str = String::new();
        let mut min_str = String::new();
        let mut max_str = String::new();
        let mut step_str = String::new();
        let mut param_type = "float";

        for line in block.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with(META_INDICATOR) {
                let after = trimmed[META_INDICATOR.len()..].trim();
                let mut parts = after.splitn(2, char::is_whitespace);
                let directive = parts.next().map(|s| s.to_uppercase());
                let value = parts.next().unwrap_or("").trim();

                match directive.as_deref() {
                    Some("DEFAULT") => default_str = value.to_string(),
                    Some("LABEL") => param.label = value.to_string(),
                    Some("MIN") => min_str = value.to_string(),
                    Some("MAX") => max_str = value.to_string(),
                    Some("STEP") => step_str = value.to_string(),
                    _ => {}
                }
            } else if trimmed.starts_with("float ") || trimmed.starts_with("int ") {
                // 変数宣言を解析
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    param_type = if trimmed.starts_with("int") {
                        "int"
                    } else {
                        "float"
                    };
                    param.name = parts[1].trim_end_matches(';').to_string();
                }
            }
        }

        // 定数を設定
        if param_type == "int" {
            param.constant = EffectConstant::Int {
                default_value: default_str.parse().unwrap_or(0),
                min_value: min_str.parse().unwrap_or(0),
                max_value: max_str.parse().unwrap_or(100),
                step: step_str.parse().unwrap_or(1),
            };
        } else {
            param.constant = EffectConstant::Float {
                default_value: default_str.parse().unwrap_or(0.0),
                min_value: min_str.parse().unwrap_or(0.0),
                max_value: max_str.parse().unwrap_or(1.0),
                step: step_str.parse().unwrap_or(0.01),
            };
        }

        desc.params.push(param);
        Ok(())
    }

    /// テクスチャを解析
    fn resolve_texture(&self, block: &str, desc: &mut EffectDesc) -> Result<()> {
        let mut tex = EffectIntermediateTextureDesc::default();
        let mut source = String::new();
        let mut format = EffectIntermediateTextureFormat::Unknown;
        let mut width_expr = String::new();
        let mut height_expr = String::new();

        for line in block.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with(META_INDICATOR) {
                let after = trimmed[META_INDICATOR.len()..].trim();
                let mut parts = after.splitn(2, char::is_whitespace);
                let directive = parts.next().map(|s| s.to_uppercase());
                let value = parts.next().unwrap_or("").trim();

                match directive.as_deref() {
                    Some("SOURCE") => source = value.to_string(),
                    Some("FORMAT") => {
                        format = EffectIntermediateTextureFormat::from_str(value)
                            .unwrap_or(EffectIntermediateTextureFormat::Unknown);
                    }
                    Some("WIDTH") => width_expr = value.to_string(),
                    Some("HEIGHT") => height_expr = value.to_string(),
                    _ => {}
                }
            } else if trimmed.starts_with("Texture2D") {
                // Texture2D 宣言を解析
                let name = trimmed
                    .trim_start_matches("Texture2D")
                    .trim()
                    .trim_end_matches(';')
                    .split('<')
                    .next()
                    .unwrap_or("")
                    .trim();
                tex.name = name.to_string();
            }
        }

        // INPUT または OUTPUT の場合は既存のエントリを更新
        if tex.name == "INPUT" || tex.name == "OUTPUT" {
            let idx = if tex.name == "INPUT" { 0 } else { 1 };
            if !width_expr.is_empty() && !height_expr.is_empty() {
                desc.textures[idx].size_expr = (width_expr, height_expr);
            }
            return Ok(());
        }

        tex.format = format;
        tex.source = source;
        tex.size_expr = (width_expr, height_expr);
        desc.textures.push(tex);

        Ok(())
    }

    /// サンプラーを解析
    fn resolve_sampler(&self, block: &str, desc: &mut EffectDesc) -> Result<()> {
        let mut sampler = EffectSamplerDesc::default();

        for line in block.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with(META_INDICATOR) {
                let after = trimmed[META_INDICATOR.len()..].trim();
                let mut parts = after.splitn(2, char::is_whitespace);
                let directive = parts.next().map(|s| s.to_uppercase());
                let value = parts.next().unwrap_or("").trim().to_uppercase();

                match directive.as_deref() {
                    Some("FILTER") => {
                        sampler.filter_type = match value.as_str() {
                            "POINT" => EffectSamplerFilterType::Point,
                            _ => EffectSamplerFilterType::Linear,
                        };
                    }
                    Some("ADDRESS") => {
                        sampler.address_type = match value.as_str() {
                            "WRAP" => EffectSamplerAddressType::Wrap,
                            _ => EffectSamplerAddressType::Clamp,
                        };
                    }
                    _ => {}
                }
            } else if trimmed.starts_with("SamplerState") {
                // SamplerState 宣言を解析
                let name = trimmed
                    .trim_start_matches("SamplerState")
                    .trim()
                    .trim_end_matches(';')
                    .to_string();
                sampler.name = name;
            }
        }

        desc.samplers.push(sampler);
        Ok(())
    }

    /// パスを解析
    fn resolve_passes(
        &self,
        pass_blocks: &[String],
        common_blocks: &[String],
        desc: &mut EffectDesc,
    ) -> Result<()> {
        // Common ブロックをまとめる
        desc.common_code = common_blocks.join("\n");

        // パス番号と内容のペアを作成
        let mut pass_pairs: Vec<(u32, EffectPassDesc)> = Vec::new();

        for block in pass_blocks {
            let mut pass = EffectPassDesc::default();
            let mut pass_num = 0u32;

            for line in block.lines() {
                let trimmed = line.trim();

                if !trimmed.starts_with(META_INDICATOR) {
                    continue;
                }

                let after = trimmed[META_INDICATOR.len()..].trim();
                let mut parts = after.splitn(2, char::is_whitespace);
                let directive = parts.next().map(|s| s.to_uppercase());
                let value = parts.next().unwrap_or("").trim();

                match directive.as_deref() {
                    Some("PASS") => {
                        pass_num = value.parse().unwrap_or(1);
                    }
                    Some("IN") => {
                        for input in value.split(',') {
                            let input = input.trim();
                            if let Some(idx) = desc.textures.iter().position(|t| t.name == input) {
                                pass.inputs.push(idx);
                            }
                        }
                    }
                    Some("OUT") => {
                        for output in value.split(',') {
                            let output = output.trim();
                            if let Some(idx) = desc.textures.iter().position(|t| t.name == output) {
                                pass.outputs.push(idx);
                            }
                        }
                    }
                    Some("STYLE") => {
                        if value.to_uppercase() == "PS" {
                            pass.flags |= effect_pass_flags::PS_STYLE;
                            pass.block_size = (16, 16);
                            pass.num_threads = [64, 1, 1];
                        }
                    }
                    Some("BLOCK_SIZE") => {
                        let parts: Vec<u32> = value
                            .split(',')
                            .filter_map(|s| s.trim().parse().ok())
                            .collect();
                        if parts.len() >= 2 {
                            pass.block_size = (parts[0], parts[1]);
                        } else if parts.len() == 1 {
                            pass.block_size = (parts[0], parts[0]);
                        }
                    }
                    Some("NUM_THREADS") => {
                        let parts: Vec<u32> = value
                            .split(',')
                            .filter_map(|s| s.trim().parse().ok())
                            .collect();

                        for i in 0..3 {
                            if i < parts.len() {
                                pass.num_threads[i] = parts[i];
                            } else {
                                pass.num_threads[i] = 1;
                            }
                        }
                    }
                    Some("DESC") => {
                        pass.desc = value.to_string();
                    }
                    _ => {}
                }
            }

            if pass.desc.is_empty() {
                pass.desc = format!("Pass {}", pass_num);
            }

            pass_pairs.push((pass_num, pass));
        }

        // パス番号でソート
        pass_pairs.sort_by_key(|(num, _)| *num);

        // パスを追加
        for (_, pass) in pass_pairs {
            desc.passes.push(pass);
        }

        Ok(())
    }

    /// 名前の重複チェック
    fn check_duplicate_names(&self, desc: &EffectDesc) -> Result<()> {
        let mut names = HashSet::new();

        for param in &desc.params {
            if !names.insert(&param.name) {
                return Err(anyhow!("Duplicate identifier: {}", param.name));
            }
        }

        for tex in &desc.textures {
            if !names.insert(&tex.name) {
                return Err(anyhow!("Duplicate identifier: {}", tex.name));
            }
        }

        for sampler in &desc.samplers {
            if !names.insert(&sampler.name) {
                return Err(anyhow!("Duplicate identifier: {}", sampler.name));
            }
        }

        Ok(())
    }

    /// パスソースを生成
    fn generate_pass_source(
        &self,
        desc: &EffectDesc,
        pass_idx: usize,
        original_source: &str,
    ) -> Result<(String, Vec<(String, String)>)> {
        let pass = desc
            .passes
            .get(pass_idx - 1)
            .ok_or_else(|| anyhow!("Pass {} not found", pass_idx))?;

        let mut result = String::with_capacity(4096);
        let mut macros = Vec::new();

        // 定数バッファ
        result.push_str(
            r#"cbuffer __CB1 : register(b0) {
    uint2 __inputSize;
    uint2 __outputSize;
    float2 __inputPt;
    float2 __outputPt;
    float2 __scale;
    float2 __srcRectOffset;
"#,
        );

        // パラメータを追加
        for param in &desc.params {
            let type_name = match &param.constant {
                EffectConstant::Float { .. } => "float",
                EffectConstant::Int { .. } => "int",
            };
            result.push_str(&format!("    {} {};\n", type_name, param.name));
        }

        result.push_str("};\n\n");

        // MF マクロ定義 (FP16/FP32切り替え用)
        if desc.flags & effect_flags::SUPPORT_FP16 != 0 {
            result.push_str(
                r#"#define MF min16float
#define MF1 min16float
#define MF2 min16float2
#define MF3 min16float3
#define MF4 min16float4
#define MF1x1 min16float1x1
#define MF1x2 min16float1x2
#define MF1x3 min16float1x3
#define MF1x4 min16float1x4
#define MF2x1 min16float2x1
#define MF2x2 min16float2x2
#define MF2x3 min16float2x3
#define MF2x4 min16float2x4
#define MF3x1 min16float3x1
#define MF3x2 min16float3x2
#define MF3x3 min16float3x3
#define MF3x4 min16float3x4
#define MF4x1 min16float4x1
#define MF4x2 min16float4x2
#define MF4x3 min16float4x3
#define MF4x4 min16float4x4

"#,
            );
        } else {
            result.push_str(
                r#"#define MF float
#define MF1 float1
#define MF2 float2
#define MF3 float3
#define MF4 float4
#define MF1x1 float1x1
#define MF1x2 float1x2
#define MF1x3 float1x3
#define MF1x4 float1x4
#define MF2x1 float2x1
#define MF2x2 float2x2
#define MF2x3 float2x3
#define MF2x4 float2x4
#define MF3x1 float3x1
#define MF3x2 float3x2
#define MF3x3 float3x3
#define MF3x4 float3x4
#define MF4x1 float4x1
#define MF4x2 float4x2
#define MF4x3 float4x3
#define MF4x4 float4x4

"#,
            );
        }

        // Dynamic フラグ
        if desc.flags & effect_flags::USE_DYNAMIC != 0 {
            result.push_str("cbuffer __CB2 : register(b1) { uint __frameCount; };\n\n");
        }

        // SRV
        for (i, &input_idx) in pass.inputs.iter().enumerate() {
            let tex = &desc.textures[input_idx];
            let format_desc = tex.format.desc();
            result.push_str(&format!(
                "Texture2D<{}> {} : register(t{});\n",
                format_desc.srv_texel_type, tex.name, i
            ));
        }

        // UAV
        for (i, &output_idx) in pass.outputs.iter().enumerate() {
            let tex = &desc.textures[output_idx];
            let format_desc = tex.format.desc();
            result.push_str(&format!(
                "RWTexture2D<{}> {} : register(u{});\n",
                format_desc.uav_texel_type, tex.name, i
            ));
        }

        // サンプラー
        for (i, sampler) in desc.samplers.iter().enumerate() {
            result.push_str(&format!(
                "SamplerState {} : register(s{});\n",
                sampler.name, i
            ));
        }

        result.push('\n');

        // マクロ
        macros.push(("MP_BLOCK_WIDTH".to_string(), pass.block_size.0.to_string()));
        macros.push(("MP_BLOCK_HEIGHT".to_string(), pass.block_size.1.to_string()));
        macros.push((
            "MP_NUM_THREADS_X".to_string(),
            pass.num_threads[0].to_string(),
        ));
        macros.push((
            "MP_NUM_THREADS_Y".to_string(),
            pass.num_threads[1].to_string(),
        ));
        macros.push((
            "MP_NUM_THREADS_Z".to_string(),
            pass.num_threads[2].to_string(),
        ));

        if pass.is_ps_style() {
            macros.push(("MP_PS_STYLE".to_string(), String::new()));
        }

        // Common コード
        result.push_str(&desc.common_code);
        result.push_str("\n");

        // 内蔵関数
        result.push_str(r#"uint __Bfe(uint src, uint off, uint bits) { uint mask = (1u << bits) - 1; return (src >> off) & mask; }
uint __BfiM(uint src, uint ins, uint bits) { uint mask = (1u << bits) - 1; return (ins & mask) | (src & (~mask)); }
uint2 Rmp8x8(uint a) { return uint2(__Bfe(a, 1u, 3u), __BfiM(__Bfe(a, 3u, 3u), a, 1u)); }
uint2 GetInputSize() { return __inputSize; }
float2 GetInputPt() { return __inputPt; }
uint2 GetOutputSize() { return __outputSize; }
float2 GetOutputPt() { return __outputPt; }
float2 GetScale() { return __scale; }
float2 GetSrcRectOffset() { return __srcRectOffset; }

"#);

        if desc.flags & effect_flags::USE_MUL_ADD != 0 {
            result.push_str(
                r#"
MF2 MulAdd(MF2 x, MF2x2 y, MF2 a) {
    MF2 result = a;
    result = mad(x.x, y._m00_m01, result);
    result = mad(x.y, y._m10_m11, result);
    return result;
}
MF3 MulAdd(MF2 x, MF2x3 y, MF3 a) {
    MF3 result = a;
    result = mad(x.x, y._m00_m01_m02, result);
    result = mad(x.y, y._m10_m11_m12, result);
    return result;
}
MF4 MulAdd(MF2 x, MF2x4 y, MF4 a) {
    MF4 result = a;
    result = mad(x.x, y._m00_m01_m02_m03, result);
    result = mad(x.y, y._m10_m11_m12_m13, result);
    return result;
}
MF2 MulAdd(MF3 x, MF3x2 y, MF2 a) {
    MF2 result = a;
    result = mad(x.x, y._m00_m01, result);
    result = mad(x.y, y._m10_m11, result);
    result = mad(x.z, y._m20_m21, result);
    return result;
}
MF3 MulAdd(MF3 x, MF3x3 y, MF3 a) {
    MF3 result = a;
    result = mad(x.x, y._m00_m01_m02, result);
    result = mad(x.y, y._m10_m11_m12, result);
    result = mad(x.z, y._m20_m21_m22, result);
    return result;
}
MF4 MulAdd(MF3 x, MF3x4 y, MF4 a) {
    MF4 result = a;
    result = mad(x.x, y._m00_m01_m02_m03, result);
    result = mad(x.y, y._m10_m11_m12_m13, result);
    result = mad(x.z, y._m20_m21_m22_m23, result);
    return result;
}
MF2 MulAdd(MF4 x, MF4x2 y, MF2 a) {
    MF2 result = a;
    result = mad(x.x, y._m00_m01, result);
    result = mad(x.y, y._m10_m11, result);
    result = mad(x.z, y._m20_m21, result);
    result = mad(x.w, y._m30_m31, result);
    return result;
}
MF3 MulAdd(MF4 x, MF4x3 y, MF3 a) {
    MF3 result = a;
    result = mad(x.x, y._m00_m01_m02, result);
    result = mad(x.y, y._m10_m11_m12, result);
    result = mad(x.z, y._m20_m21_m22, result);
    result = mad(x.w, y._m30_m31_m32, result);
    return result;
}
MF4 MulAdd(MF4 x, MF4x4 y, MF4 a) {
    MF4 result = a;
    result = mad(x.x, y._m00_m01_m02_m03, result);
    result = mad(x.y, y._m10_m11_m12_m13, result);
    result = mad(x.z, y._m20_m21_m22_m23, result);
    result = mad(x.w, y._m30_m31_m32_m33, result);
    return result;
}
"#,
            );
        }

        // エフェクトコードを抽出して追加
        let code = self.extract_pass_code(original_source, pass_idx);
        result.push_str(&code);
        result.push_str("\n\n");

        // エントリポイント
        if pass.is_ps_style() {
            let output_name = if !pass.outputs.is_empty() {
                &desc.textures[pass.outputs[0]].name
            } else {
                "OUTPUT"
            };

            let (output_size_var, output_pt_var) = if pass_idx == desc.passes.len() {
                ("__outputSize".to_string(), "__outputPt".to_string())
            } else {
                (
                    format!("__pass{}OutputSize", pass_idx - 1),
                    format!("__pass{}OutputPt", pass_idx - 1),
                )
            };

            result.push_str(&format!(
                r#"[numthreads(64, 1, 1)]
void __M(uint3 tid : SV_GroupThreadID, uint3 gid : SV_GroupID) {{
    uint2 gxy = (gid.xy << 4u) + Rmp8x8(tid.x);
    if (gxy.x >= {size_var}.x || gxy.y >= {size_var}.y) {{
        return;
    }}
    float2 pos = (gxy + 0.5f) * {pt_var};
    float2 step = 8 * {pt_var};

    {output}[gxy] = Pass{pass_idx}(pos);

    gxy.x += 8u;
    pos.x += step.x;
    if (gxy.x < {size_var}.x && gxy.y < {size_var}.y) {{
        {output}[gxy] = Pass{pass_idx}(pos);
    }}
    
    gxy.y += 8u;
    pos.y += step.y;
    if (gxy.x < {size_var}.x && gxy.y < {size_var}.y) {{
        {output}[gxy] = Pass{pass_idx}(pos);
    }}
    
    gxy.x -= 8u;
    pos.x -= step.x;
    if (gxy.x < {size_var}.x && gxy.y < {size_var}.y) {{
        {output}[gxy] = Pass{pass_idx}(pos);
    }}
}}
"#,
                output = output_name,
                pass_idx = pass_idx,
                size_var = output_size_var,
                pt_var = output_pt_var
            ));
        } else {
            // CS スタイル
            let block_start =
                if pass.block_size.0 == pass.block_size.1 && pass.block_size.0.is_power_of_two() {
                    let shift = (pass.block_size.0 as f32).log2() as u32;
                    format!("(gid.xy << {})", shift)
                } else {
                    format!(
                        "gid.xy * uint2({}, {})",
                        pass.block_size.0, pass.block_size.1
                    )
                };

            result.push_str(&format!(
                r#"[numthreads({}, {}, {})]
void __M(uint3 tid : SV_GroupThreadID, uint3 gid : SV_GroupID) {{
    Pass{}({}, tid);
}}
"#,
                pass.num_threads[0],
                pass.num_threads[1],
                pass.num_threads[2],
                pass_idx,
                block_start
            ));
        }

        Ok((result, macros))
    }

    /// パスコードを抽出
    fn extract_pass_code(&self, source: &str, pass_idx: usize) -> String {
        let mut result = String::new();
        let mut in_target_pass = false;
        let mut current_pass = 0u32;

        let mut brace_depth = 0;
        let mut collecting_function = false;

        for line in source.lines() {
            let trimmed = line.trim();

            // パスの開始を検出
            if trimmed.starts_with(META_INDICATOR) {
                let after = trimmed[META_INDICATOR.len()..].trim();
                if after.to_uppercase().starts_with("PASS") {
                    let num_str = after[4..].trim();
                    if let Ok(num) = num_str.parse::<u32>() {
                        current_pass = num;
                        in_target_pass = current_pass == pass_idx as u32;
                    }
                }
                continue;
            }

            // メタデータ行をスキップ
            if trimmed.starts_with(META_INDICATOR) {
                continue;
            }

            // #include をスキップ (既に展開済み)
            if trimmed.starts_with("#include") {
                continue;
            }

            // リソース宣言をスキップ
            if trimmed.starts_with("Texture2D")
                || trimmed.starts_with("SamplerState")
                || trimmed.starts_with("RWTexture2D")
            {
                continue;
            }

            // 対象パス内の関数とヘルパーコードを収集
            if in_target_pass {
                // 関数定義の開始を検出
                if (trimmed.starts_with("float")
                    || trimmed.starts_with("int")
                    || trimmed.starts_with("void")
                    || trimmed.starts_with("uint")
                    || trimmed.starts_with("half")
                    || trimmed.starts_with("MF"))
                    && trimmed.contains('(')
                    && !trimmed.contains(';')
                {
                    collecting_function = true;
                    brace_depth = 0;
                }

                if collecting_function {
                    result.push_str(line);
                    result.push('\n');

                    // ブレースを追跡
                    for c in line.chars() {
                        match c {
                            '{' => brace_depth += 1,
                            '}' => {
                                brace_depth -= 1;
                                if brace_depth == 0 {
                                    collecting_function = false;
                                    result.push('\n');
                                }
                            }
                            _ => {}
                        }
                    }
                } else if !trimmed.is_empty()
                    && !trimmed.starts_with("//")
                    && (trimmed.starts_with("#define") || trimmed.starts_with("static"))
                {
                    // マクロやstatic定数も収集
                    result.push_str(line);
                    result.push('\n');
                }
            }
        }

        result
    }
}
