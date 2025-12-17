//! インクルードファイル解決
//! D3DCompile の ID3DInclude インターフェース相当の機能

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// インクルードファイルを解決するハンドラ
pub struct IncludeHandler {
    base_dir: PathBuf,
    cache: HashMap<String, String>,
}

impl IncludeHandler {
    /// 新しいインクルードハンドラを作成
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            cache: HashMap::new(),
        }
    }

    /// インクルードファイルを読み込む
    pub fn open(&mut self, filename: &str) -> Result<&str> {
        if !self.cache.contains_key(filename) {
            let path = self.base_dir.join(filename);
            let content = std::fs::read_to_string(&path)
                .map_err(|e| anyhow!("Failed to read include file '{}': {}", path.display(), e))?;
            self.cache.insert(filename.to_string(), content);
        }
        Ok(self.cache.get(filename).unwrap())
    }

    /// ソース内のすべての #include を展開
    pub fn expand_includes(&mut self, source: &str) -> Result<String> {
        let mut result = String::with_capacity(source.len() * 2);

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("#include") {
                // #include "filename" または #include <filename> を解析
                let filename = self.parse_include_directive(trimmed)?;

                // インクルードファイルの内容を取得
                let include_content = self.open(&filename)?.to_string();

                // 再帰的にインクルードを展開
                let expanded = self.expand_includes(&include_content)?;
                result.push_str(&expanded);
                result.push('\n');
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }

        Ok(result)
    }

    /// #include ディレクティブからファイル名を解析
    fn parse_include_directive(&self, line: &str) -> Result<String> {
        // #include "filename" または #include <filename>
        let line = line.trim_start_matches("#include").trim();

        if line.starts_with('"') && line.ends_with('"') {
            Ok(line[1..line.len() - 1].to_string())
        } else if line.starts_with('<') && line.ends_with('>') {
            Ok(line[1..line.len() - 1].to_string())
        } else {
            Err(anyhow!("Invalid include directive: {}", line))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_include_directive() {
        let handler = IncludeHandler::new(".");

        assert_eq!(
            handler
                .parse_include_directive(r#"#include "test.hlsli""#)
                .unwrap(),
            "test.hlsli"
        );

        assert_eq!(
            handler
                .parse_include_directive(r#"#include <test.hlsli>"#)
                .unwrap(),
            "test.hlsli"
        );
    }
}
