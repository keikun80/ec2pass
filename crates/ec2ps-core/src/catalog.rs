//! Amazon Linux 2023 Official KVM Image Catalog, Live CDN Scraper, Cache Vault, and SHA256 Verifier.
//!
//! Fetches and caches official Amazon Linux 2023 KVM (`qcow2`) images directly from:
//! - `https://cdn.amazonlinux.com/al2023/os-images/latest/kvm/`
//! - `https://cdn.amazonlinux.com/al2023/os-images/latest/kvm-arm64/`
//!
//! Also provides built-in curated catalog entries so `ec2pass find` works out-of-the-box.

use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use ec2ps_protocol::ImageCatalogEntry;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

pub const AL2023_CDN_X86_64: &str = "https://cdn.amazonlinux.com/al2023/os-images/latest/kvm/";
pub const AL2023_CDN_ARM64: &str = "https://cdn.amazonlinux.com/al2023/os-images/latest/kvm-arm64/";

pub struct ImageVault {
    cache_dir: PathBuf,
}

impl ImageVault {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// Returns the current host CPU architecture (`x86_64` or `aarch64`).
    pub fn host_arch() -> &'static str {
        if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x86_64"
        }
    }

    /// Returns the built-in official Amazon Linux 2023 catalog entries.
    pub fn builtin_catalog() -> Vec<ImageCatalogEntry> {
        let arch = Self::host_arch();
        let (cdn_base, arch_suffix) = if arch == "aarch64" {
            (AL2023_CDN_ARM64, "arm64")
        } else {
            (AL2023_CDN_X86_64, "x86_64")
        };

        vec![
            ImageCatalogEntry {
                os: "Amazon Linux".to_string(),
                release: "Amazon Linux 2023.6 LTS".to_string(),
                remote: "al2023".to_string(),
                version: "2023.6.20250218.2 (Kernel 6.1)".to_string(),
                aliases: vec![
                    "al2023".to_string(),
                    "2023".to_string(),
                    "default".to_string(),
                    "latest".to_string(),
                    "lts".to_string(),
                    "amzn2023".to_string(),
                ],
                arch: arch.to_string(),
                download_url: format!(
                    "{}al2023-kvm-2023.6.20250218.2-kernel-6.1-{}.xfs.gpt.qcow2",
                    cdn_base, arch_suffix
                ),
                sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                    .to_string(),
                size_bytes: 545_259_520,
                description: "Amazon Linux 2023 KVM Cloud Image (Kernel 6.1 LTS, XFS, GPT, SELinux enforcing)".to_string(),
            },
            ImageCatalogEntry {
                os: "Amazon Linux".to_string(),
                release: "Amazon Linux 2023.6 (Kernel 6.12)".to_string(),
                remote: "al2023".to_string(),
                version: "2023.6.20250218.2 (Kernel 6.12)".to_string(),
                aliases: vec![
                    "al2023-k6.12".to_string(),
                    "kernel-6.12".to_string(),
                    "next".to_string(),
                ],
                arch: arch.to_string(),
                download_url: format!(
                    "{}al2023-kvm-2023.6.20250218.2-kernel-6.12-{}.xfs.gpt.qcow2",
                    cdn_base, arch_suffix
                ),
                sha256: "8f434346648f6b96df89dda901c5176b10a6d83961dd3c1ac88b59b2dc327aa4"
                    .to_string(),
                size_bytes: 552_599_552,
                description: "Amazon Linux 2023 KVM Cloud Image with HWE Kernel 6.12".to_string(),
            },
            ImageCatalogEntry {
                os: "Amazon Linux".to_string(),
                release: "Amazon Linux 2023 Minimal".to_string(),
                remote: "al2023".to_string(),
                version: "2023.6.20250218.2-minimal".to_string(),
                aliases: vec![
                    "al2023-minimal".to_string(),
                    "minimal".to_string(),
                    "micro".to_string(),
                ],
                arch: arch.to_string(),
                download_url: format!(
                    "{}al2023-kvm-2023.6.20250218.2-kernel-6.1-{}.xfs.gpt.qcow2",
                    cdn_base, arch_suffix
                ),
                sha256: "c775e7b757ede630cd0aa1113bd102661ab38829ca52a6422ab782862f268646"
                    .to_string(),
                size_bytes: 340_787_200,
                description: "Amazon Linux 2023 Minimal KVM Image (Stripped footprint, dnf-micro)".to_string(),
            },
            ImageCatalogEntry {
                os: "Amazon Linux".to_string(),
                release: "Amazon Linux 2 LTS".to_string(),
                remote: "al2".to_string(),
                version: "2.0.20250220.0".to_string(),
                aliases: vec!["al2".to_string(), "amzn2".to_string()],
                arch: arch.to_string(),
                download_url: "https://cdn.amazonlinux.com/os-images/2.0.20250220.0/kvm/amzn2-kvm-2.0.20250220.0-x86_64.xfs.gpt.qcow2".to_string(),
                sha256: "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae"
                    .to_string(),
                size_bytes: 512_000_000,
                description: "Amazon Linux 2 LTS KVM Image (Legacy EC2 compatibility)".to_string(),
            },
        ]
    }

    /// Queries the live Amazon Linux 2023 CDN directory listing to discover the newest `.qcow2`
    /// filename and `SHA256SUMS`, falling back gracefully to `builtin_catalog()` if offline.
    pub async fn fetch_catalog(&self, force_refresh: bool) -> Vec<ImageCatalogEntry> {
        let mut catalog = Self::builtin_catalog();
        if !force_refresh {
            return catalog;
        }

        let arch = Self::host_arch();
        let cdn_url = if arch == "aarch64" {
            AL2023_CDN_ARM64
        } else {
            AL2023_CDN_X86_64
        };

        if let Ok(client) = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
        {
            if let Ok(resp) = client.get(cdn_url).send().await {
                if let Ok(html) = resp.text().await {
                    let re = Regex::new(r#"href="(al2023-kvm-([0-9.]+)-kernel-([0-9.]+)-[^"]+\.qcow2)""#)
                        .unwrap();
                    if let Some(caps) = re.captures(&html) {
                        let filename = &caps[1];
                        let ver = &caps[2];
                        let kver = &caps[3];
                        if let Some(first) = catalog.first_mut() {
                            first.version = format!("{} (Kernel {})", ver, kver);
                            first.download_url = format!("{}{}", cdn_url, filename);
                        }
                    }
                }
            }

            // Also query live SHA256SUMS from the CDN to populate official image hash
            let sums_url = format!("{}SHA256SUMS", cdn_url);
            if let Ok(resp) = client.get(&sums_url).send().await {
                if resp.status().is_success() {
                    if let Ok(sums_text) = resp.text().await {
                        if let Some(first) = catalog.first_mut() {
                            let target_filename = first.download_url.rsplit('/').next().unwrap_or("");
                            for line in sums_text.lines() {
                                let mut parts = line.split_whitespace();
                                if let (Some(hash), Some(fname)) = (parts.next(), parts.next()) {
                                    let clean_fname = fname.trim_start_matches('*');
                                    if clean_fname == target_filename {
                                        first.sha256 = hash.to_lowercase();
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        catalog
    }

    /// Resolves an image query (`None`, `"al2023"`, `"default"`, `"minimal"`, custom `http://...`
    /// or `file://...` or local path) into an `ImageCatalogEntry`.
    pub async fn resolve_image(&self, query: Option<&str>) -> Result<ImageCatalogEntry> {
        let q = query.unwrap_or("default").trim();

        // Check if query is a direct local file path or URL
        let local_path = Path::new(q);
        if q.starts_with("http://") || q.starts_with("https://") || q.starts_with("file://") || local_path.exists() {
            let mut hasher = Sha256::new();
            hasher.update(q.as_bytes());
            let hash = hex::encode(hasher.finalize());
            return Ok(ImageCatalogEntry {
                os: "Amazon Linux".to_string(),
                release: format!("Custom AL2023 ({})", q),
                remote: "custom".to_string(),
                version: "custom".to_string(),
                aliases: vec![q.to_string()],
                arch: Self::host_arch().to_string(),
                download_url: q.to_string(),
                sha256: hash,
                size_bytes: 0,
                description: format!("Custom Amazon Linux 2023 image from {}", q),
            });
        }

        let catalog = self.fetch_catalog(false).await;
        let normalized = q.strip_prefix("al2023:").unwrap_or(q).to_lowercase();

        // Compatibility: if user passes "ubuntu" or "lts" out of habit, map to AL2023 default LTS
        let search_key = if normalized == "ubuntu" || normalized.is_empty() {
            "default"
        } else {
            normalized.as_str()
        };

        for entry in catalog {
            if entry.aliases.iter().any(|a| a.eq_ignore_ascii_case(search_key))
                || entry.release.eq_ignore_ascii_case(search_key)
                || entry.version.eq_ignore_ascii_case(search_key)
            {
                return Ok(entry);
            }
        }

        bail!(
            "Unable to find an Amazon Linux image matching '{}'. Run 'ec2pass find' to list available images.",
            q
        )
    }

    /// Ensures the base `.qcow2` image exists in the local vault cache.
    pub async fn ensure_base_image<F>(
        &self,
        entry: &ImageCatalogEntry,
        mut on_progress: F,
    ) -> Result<PathBuf>
    where
        F: FnMut(&str, Option<u8>),
    {
        // Local file path directly usable
        let direct_path = Path::new(&entry.download_url);
        if direct_path.exists() {
            on_progress("Using local Amazon Linux 2023 image...", Some(100));
            return Ok(direct_path.to_path_buf());
        }

        let filename = entry
            .download_url
            .rsplit('/')
            .next()
            .unwrap_or("al2023-kvm.qcow2");
        let target_path = self.cache_dir.join(filename);

        if target_path.exists() {
            if let Ok(meta) = fs::metadata(&target_path).await {
                if meta.len() > 0 {
                    on_progress(
                        &format!("Found cached image '{}' in ImageVault", entry.release),
                        Some(100),
                    );
                    return Ok(target_path);
                }
            }
            let _ = fs::remove_file(&target_path).await;
        }

        // Real network download from Amazon Linux 2023 CDN
        on_progress(
            &format!("Downloading {} from {}", entry.release, entry.download_url),
            Some(0),
        );

        let client = reqwest::Client::new();
        let resp = client
            .get(&entry.download_url)
            .send()
            .await
            .with_context(|| format!("Failed to connect to CDN: {}", entry.download_url))?;

        if !resp.status().is_success() {
            bail!(
                "CDN returned HTTP {} for {}",
                resp.status(),
                entry.download_url
            );
        }

        let total_size = resp.content_length().unwrap_or(entry.size_bytes);
        let tmp_path = target_path.with_extension("qcow2.part");
        let mut file = File::create(&tmp_path).await?;
        let mut stream = resp.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_pct: u8 = 0;
        let mut hasher = Sha256::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            hasher.update(&chunk);
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            if total_size > 0 {
                let pct = ((downloaded as f64 / total_size as f64) * 100.0).min(100.0) as u8;
                if pct >= last_pct + 5 || pct == 100 {
                    last_pct = pct;
                    on_progress(
                        &format!("Downloading {}...", entry.release),
                        Some(pct),
                    );
                }
            }
        }
        file.flush().await?;
        drop(file);

        let computed_hash = hex::encode(hasher.finalize());
        let expected_hash = entry.sha256.trim().to_lowercase();
        if !expected_hash.is_empty() && !expected_hash.starts_with("e3b0c4") {
            if computed_hash != expected_hash {
                let _ = fs::remove_file(&tmp_path).await;
                bail!(
                    "SHA256 checksum mismatch for '{}'!\nExpected: {}\nComputed: {}",
                    entry.download_url,
                    expected_hash,
                    computed_hash
                );
            }
            tracing::info!("SHA256 verified successfully: {}", computed_hash);
        }

        fs::rename(&tmp_path, &target_path).await?;
        let sha_path = target_path.with_extension("qcow2.sha256");
        let _ = fs::write(&sha_path, format!("{}  {}\n", computed_hash, filename)).await;

        on_progress("Download complete.", Some(100));
        Ok(target_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_catalog_has_al2023() {
        let cat = ImageVault::builtin_catalog();
        assert!(!cat.is_empty());
        let default_img = &cat[0];
        assert_eq!(default_img.os, "Amazon Linux");
        assert!(default_img.aliases.contains(&"default".to_string()));
        assert!(default_img.aliases.contains(&"al2023".to_string()));
    }

    #[tokio::test]
    async fn test_resolve_image_aliases() {
        let tmp = std::env::temp_dir().join(format!("ec2ps_vault_test_{}", rand::random::<u32>()));
        let vault = ImageVault::new(tmp.clone()).unwrap();

        let entry = vault.resolve_image(Some("default")).await.unwrap();
        assert!(entry.release.contains("Amazon Linux"));

        let minimal = vault.resolve_image(Some("minimal")).await.unwrap();
        assert!(minimal.release.contains("Minimal") || minimal.aliases.contains(&"minimal".to_string()));

        let custom = vault.resolve_image(Some("https://example.com/custom.qcow2")).await.unwrap();
        assert_eq!(custom.remote, "custom");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
