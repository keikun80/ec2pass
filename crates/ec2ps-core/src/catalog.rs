//! Amazon Linux 2023 Official KVM Image Catalog, Live CDN Scraper, Cache Vault, and SHA256 Verifier.
//!
//! Fetches and caches official Amazon Linux 2023 KVM (`qcow2`) images directly from:
//! - `https://cdn.amazonlinux.com/al2023/os-images/latest/kvm/`
//! - `https://cdn.amazonlinux.com/al2023/os-images/latest/kvm-arm64/`
//!
//! Also provides built-in curated catalog entries so `ec2pass find` works out-of-the-box.

use anyhow::{bail, Context, Result};
use ec2ps_protocol::ImageCatalogEntry;
use futures_util::StreamExt;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

pub const AL2023_CDN_X86_64: &str = "https://cdn.amazonlinux.com/al2023/os-images/latest/kvm/";
pub const AL2023_CDN_ARM64: &str = "https://cdn.amazonlinux.com/al2023/os-images/latest/kvm-arm64/";
pub const AL2023_CDN_BASE: &str = "https://cdn.amazonlinux.com/al2023/os-images/";
pub const AL2023_RELEASE_NOTES_RSS: &str =
    "https://docs.aws.amazon.com/linux/al2023/release-notes/linux-release-notes.rss";

/// Per-architecture string pair (image URL or checksum) for one distro.
pub struct ArchImageUrls {
    pub x86_64: &'static str,
    pub aarch64: &'static str,
}

impl ArchImageUrls {
    /// Returns the value for `arch` (`aarch64` or anything else = `x86_64`).
    pub fn for_arch(&self, arch: &str) -> String {
        if arch == "aarch64" {
            self.aarch64.to_string()
        } else {
            self.x86_64.to_string()
        }
    }
}

// Latest releases verified on 2026-11-24. Bump these when a distro ships a new release.

pub const UBUNTU_LATEST_IMAGE: ArchImageUrls = ArchImageUrls {
    x86_64: "https://cloud-images.ubuntu.com/releases/26.04/release/ubuntu-26.04-server-cloudimg-amd64.img",
    aarch64: "https://cloud-images.ubuntu.com/releases/26.04/release/ubuntu-26.04-server-cloudimg-arm64.img",
};

/// Debian `latest/` tracks the newest point release of the stable series.
pub const DEBIAN_LATEST_IMAGE: ArchImageUrls = ArchImageUrls {
    x86_64: "https://cloud.debian.org/images/cloud/trixie/latest/debian-13-generic-amd64.qcow2",
    aarch64: "https://cloud.debian.org/images/cloud/trixie/latest/debian-13-generic-arm64.qcow2",
};

/// Rocky `*.latest.*` tracks the newest minor release of the major series.
pub const ROCKY_LATEST_IMAGE: ArchImageUrls = ArchImageUrls {
    x86_64: "https://download.rockylinux.org/pub/rocky/10/images/x86_64/Rocky-10-GenericCloud-Base.latest.x86_64.qcow2",
    aarch64: "https://download.rockylinux.org/pub/rocky/10/images/aarch64/Rocky-10-GenericCloud-Base.latest.aarch64.qcow2",
};

pub const FEDORA_LATEST_IMAGE: ArchImageUrls = ArchImageUrls {
    x86_64: "https://download.fedoraproject.org/pub/fedora/linux/releases/45/Cloud/x86_64/images/Fedora-Cloud-Base-Generic-45-1.4.x86_64.qcow2",
    aarch64: "https://download.fedoraproject.org/pub/fedora/linux/releases/45/Cloud/aarch64/images/Fedora-Cloud-Base-Generic-45-1.4.aarch64.qcow2",
};

pub const AL2_LATEST_VERSION: &str = "2.0.20261120.0";

pub const AL2_LATEST_IMAGE: ArchImageUrls = ArchImageUrls {
    x86_64: "https://cdn.amazonlinux.com/os-images/2.0.20261120.0/kvm/amzn2-kvm-2.0.20261120.0-x86_64.xfs.gpt.qcow2",
    aarch64: "https://cdn.amazonlinux.com/os-images/2.0.20261120.0/kvm-arm64/amzn2-kvm-2.0.20261120.0-arm64.xfs.gpt.qcow2",
};

/// SHA-256 checksums from the AL2 CDN `SHA256SUMS` files for `AL2_LATEST_IMAGE`.
pub const AL2_LATEST_SHA256: ArchImageUrls = ArchImageUrls {
    x86_64: "ab4a95533c38dcd3876e0017341ea4abaefa962458930fe878c34b06e1677f0f",
    aarch64: "1a7a4d33011b7377b81f9f6388635bad545cc061fb40c4e3fad5e6f22183b8a1",
};

/// Curated baseline list of known Amazon Linux 2023 release versions on AWS CDN
pub const KNOWN_AL2023_VERSIONS: &[&str] = &[
    "2023.12.20261120.0",
    "2023.12.20261119.0",
    "2023.12.20261118.0",
    "2023.12.20261117.0",
    "2023.12.20261112.0",
    "2023.12.20261109.0",
    "2023.12.20261102.0",
    "2023.6.20250218.2",
    "2023.5.20240916.0",
    "2023.4.20240401.1",
    "2023.3.20240131.0",
    "2023.2.20231122.0",
    "2023.1.20230912.0",
    "2023.0.20230315.0",
];

#[derive(Clone)]
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
                release: "Amazon Linux 2023.12 LTS".to_string(),
                remote: "al2023".to_string(),
                version: "2023.12.20261120.0 (Kernel 6.1)".to_string(),
                aliases: vec![
                    "al2023".to_string(),
                    "2023".to_string(),
                    "default".to_string(),
                    "latest".to_string(),
                    "lts".to_string(),
                    "amzn2023".to_string(),
                    "al2023-k6.12".to_string(),
                    "kernel-6.12".to_string(),
                ],
                arch: arch.to_string(),
                download_url: format!(
                    "{}al2023-kvm-2023.12.20261120.0-kernel-6.1-{}.xfs.gpt.qcow2",
                    cdn_base, arch_suffix
                ),
                sha256: if arch == "aarch64" {
                    "6d7fb166db05e40dacb5fcf90b5e5184de1ce2bb5956bec694c6dbba70e7d11d".to_string()
                } else {
                    "be3240954918ce36bb293481eecdcb9a8943b5af2b4a74fd2b3dddf0cdb8a9af".to_string()
                },
                size_bytes: 1_770_520_576,
                description: "Amazon Linux 2023 KVM Cloud Image (Kernel 6.1 LTS, XFS, GPT, SELinux enforcing)".to_string(),
            },
            ImageCatalogEntry {
                os: "Amazon Linux".to_string(),
                release: "Amazon Linux 2023 Minimal".to_string(),
                remote: "al2023-minimal".to_string(),
                version: "2023.12.20261120.0".to_string(),
                aliases: vec![
                    "minimal".to_string(),
                    "al2023-minimal".to_string(),
                    "micro".to_string(),
                ],
                arch: arch.to_string(),
                download_url: format!(
                    "{}al2023-kvm-minimal-2023.12.20261120.0-kernel-6.1-{}.xfs.gpt.qcow2",
                    cdn_base.replace("/kvm/", "/kvm-minimal/"), arch_suffix
                ),
                sha256: "".to_string(),
                size_bytes: 750_000_000,
                description: "Amazon Linux 2023 Minimal KVM Image (Lightweight runtime)".to_string(),
            },
            ImageCatalogEntry {
                os: "Ubuntu".to_string(),
                release: "Ubuntu 26.04 LTS (Resolute Raccoon)".to_string(),
                remote: "ubuntu".to_string(),
                version: "26.04 LTS".to_string(),
                aliases: vec![
                    "ubuntu".to_string(),
                    "resolute".to_string(),
                    "ubuntu-26.04".to_string(),
                    "ubuntu-resolute".to_string(),
                    "ubuntu-lts".to_string(),
                ],
                arch: arch.to_string(),
                download_url: UBUNTU_LATEST_IMAGE.for_arch(arch),
                sha256: "".to_string(),
                size_bytes: 861_563_904,
                description: "Ubuntu 26.04 LTS (Resolute Raccoon) Official Cloud Image".to_string(),
            },
            ImageCatalogEntry {
                os: "Debian".to_string(),
                release: "Debian 13 (Trixie)".to_string(),
                remote: "debian".to_string(),
                version: "13 (Trixie)".to_string(),
                aliases: vec![
                    "debian".to_string(),
                    "trixie".to_string(),
                    "debian-13".to_string(),
                ],
                arch: arch.to_string(),
                download_url: DEBIAN_LATEST_IMAGE.for_arch(arch),
                sha256: "".to_string(),
                size_bytes: 439_484_416,
                description: "Debian 13 (Trixie) Official Generic Cloud Image".to_string(),
            },
            ImageCatalogEntry {
                os: "Rocky Linux".to_string(),
                release: "Rocky Linux 10".to_string(),
                remote: "rocky".to_string(),
                version: "10".to_string(),
                aliases: vec![
                    "rocky".to_string(),
                    "rocky-10".to_string(),
                    "rockylinux".to_string(),
                ],
                arch: arch.to_string(),
                download_url: ROCKY_LATEST_IMAGE.for_arch(arch),
                sha256: "".to_string(),
                size_bytes: 568_000_512,
                description: "Rocky Linux 10 Generic Cloud Base Image (latest minor)".to_string(),
            },
            ImageCatalogEntry {
                os: "Fedora".to_string(),
                release: "Fedora 45 Cloud".to_string(),
                remote: "fedora".to_string(),
                version: "45".to_string(),
                aliases: vec![
                    "fedora".to_string(),
                    "fedora-45".to_string(),
                ],
                arch: arch.to_string(),
                download_url: FEDORA_LATEST_IMAGE.for_arch(arch),
                sha256: "".to_string(),
                size_bytes: 606_732_288,
                description: "Fedora 45 Cloud Base Image".to_string(),
            },
            ImageCatalogEntry {
                os: "Amazon Linux".to_string(),
                release: "Amazon Linux 2 LTS".to_string(),
                remote: "al2".to_string(),
                version: AL2_LATEST_VERSION.to_string(),
                aliases: vec!["al2".to_string(), "amzn2".to_string()],
                arch: arch.to_string(),
                download_url: AL2_LATEST_IMAGE.for_arch(arch),
                sha256: AL2_LATEST_SHA256.for_arch(arch),
                size_bytes: 1_467_875_328,
                description: "Amazon Linux 2 LTS KVM Image (Legacy EC2 compatibility)".to_string(),
            },
        ]
    }

    /// Discovers all available Amazon Linux 2023 release versions from the AWS CDN and official release feeds,
    /// falling back to known curated releases when offline.
    pub async fn fetch_available_versions(&self) -> Vec<String> {
        let mut versions: Vec<String> = Vec::new();

        let client = match reqwest::Client::builder()
            .timeout(crate::defaults::timeouts::HTTP_PROBE)
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
        {
            Ok(c) => c,
            Err(_) => {
                return KNOWN_AL2023_VERSIONS
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
            }
        };

        let arch = Self::host_arch();
        let cdn_url = if arch == "aarch64" {
            AL2023_CDN_ARM64
        } else {
            AL2023_CDN_X86_64
        };

        // 1. Discover latest version from AWS CDN redirect or index HTML
        if let Ok(resp) = client.get(cdn_url).send().await {
            let final_url = resp.url().to_string();
            let re_ver = Regex::new(r#"/os-images/([^/]+)/kvm"#).unwrap();
            if let Some(caps) = re_ver.captures(&final_url) {
                let ver = caps[1].to_string();
                if ver != "latest" && !versions.contains(&ver) {
                    versions.push(ver);
                }
            }
            if let Ok(html) = resp.text().await {
                let re_file = Regex::new(r#"al2023-kvm-([0-9.]+)-kernel"#).unwrap();
                if let Some(caps) = re_file.captures(&html) {
                    let ver = caps[1].to_string();
                    if !versions.contains(&ver) {
                        versions.push(ver);
                    }
                }
            }
        }

        // 2. Discover recent releases from Amazon Linux 2023 Release Notes RSS
        if let Ok(resp) = client.get(AL2023_RELEASE_NOTES_RSS).send().await {
            if resp.status().is_success() {
                if let Ok(rss_text) = resp.text().await {
                    let re_rss =
                        Regex::new(r#"<title>AL2023\s+([0-9.]+)\s+released</title>"#).unwrap();
                    for cap in re_rss.captures_iter(&rss_text) {
                        let mut ver = cap[1].to_string();
                        let dot_count = ver.chars().filter(|c| *c == '.').count();
                        if dot_count == 2 {
                            ver.push_str(".0");
                        }
                        if !versions.contains(&ver) {
                            versions.push(ver);
                        }
                    }
                }
            }
        }

        // 3. Merge known historical versions
        for &known in KNOWN_AL2023_VERSIONS {
            let k = known.to_string();
            if !versions.contains(&k) {
                versions.push(k);
            }
        }

        versions
    }

    /// Queries the live Amazon Linux 2023 CDN directory listing to discover the newest `.qcow2`
    /// filename and `SHA256SUMS`, optionally listing all available release versions.
    pub async fn fetch_catalog_extended(
        &self,
        show_versions: bool,
        force_refresh: bool,
    ) -> Vec<ImageCatalogEntry> {
        let arch = Self::host_arch();
        let kvm_dir = if arch == "aarch64" {
            "kvm-arm64"
        } else {
            "kvm"
        };
        let arch_suffix = if arch == "aarch64" { "arm64" } else { "x86_64" };

        if show_versions {
            let versions = self.fetch_available_versions().await;
            let mut entries = Vec::with_capacity(versions.len());

            for (idx, ver) in versions.iter().enumerate() {
                let is_latest = idx == 0;
                let aliases = if is_latest {
                    vec![
                        "al2023".to_string(),
                        "2023".to_string(),
                        "default".to_string(),
                        "latest".to_string(),
                        "lts".to_string(),
                        format!("al2023:{}", ver),
                        ver.clone(),
                    ]
                } else {
                    vec![format!("al2023:{}", ver), ver.clone()]
                };

                let tag_prefix = if is_latest { "[latest] [default] " } else { "" };
                let description = format!(
                    "{}Amazon Linux 2023 KVM Cloud Image (Release {}) from AWS CDN",
                    tag_prefix, ver
                );

                entries.push(ImageCatalogEntry {
                    os: "Amazon Linux".to_string(),
                    release: format!("Amazon Linux {}", ver),
                    remote: "al2023".to_string(),
                    version: format!("{} (Kernel 6.1)", ver),
                    aliases,
                    arch: arch.to_string(),
                    download_url: format!(
                        "{}{}/{}/al2023-kvm-{}-kernel-6.1-{}.xfs.gpt.qcow2",
                        AL2023_CDN_BASE, ver, kvm_dir, ver, arch_suffix
                    ),
                    sha256: String::new(),
                    size_bytes: 1_770_520_576,
                    description,
                });
            }

            return entries;
        }

        let mut catalog = Self::builtin_catalog();
        if !force_refresh {
            return catalog;
        }

        let cdn_url = if arch == "aarch64" {
            AL2023_CDN_ARM64
        } else {
            AL2023_CDN_X86_64
        };

        if let Ok(client) = reqwest::Client::builder()
            .timeout(crate::defaults::timeouts::HTTP_PROBE)
            .build()
        {
            if let Ok(resp) = client.get(cdn_url).send().await {
                if let Ok(html) = resp.text().await {
                    let re = Regex::new(
                        r#"href="(al2023-kvm-([0-9.]+)-kernel-([0-9.]+)-[^"]+\.qcow2)""#,
                    )
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
                            let target_filename =
                                first.download_url.rsplit('/').next().unwrap_or("");
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

    /// Fetches the standard catalog of images (default AL2023 LTS, Minimal, Kernel 6.12, AL2).
    pub async fn fetch_catalog(&self, force_refresh: bool) -> Vec<ImageCatalogEntry> {
        self.fetch_catalog_extended(false, force_refresh).await
    }

    /// Resolves a specific Amazon Linux 2023 release version directly against the AWS CDN.
    pub async fn resolve_specific_version(&self, req_ver: &str) -> Result<ImageCatalogEntry> {
        let arch = Self::host_arch();
        let kvm_dir = if arch == "aarch64" {
            "kvm-arm64"
        } else {
            "kvm"
        };
        let arch_suffix = if arch == "aarch64" { "arm64" } else { "x86_64" };

        // Normalize version: if user specified e.g. "2023.12.20261119", try as-is and with ".0"
        let candidate_versions = if req_ver.chars().filter(|c| *c == '.').count() == 2 {
            vec![format!("{}.0", req_ver), req_ver.to_string()]
        } else {
            vec![req_ver.to_string()]
        };

        let client = reqwest::Client::builder()
            .timeout(crate::defaults::timeouts::HTTP_RSS)
            .build()
            .unwrap_or_default();

        for candidate in &candidate_versions {
            let sums_url = format!("{}{}/{}/SHA256SUMS", AL2023_CDN_BASE, candidate, kvm_dir);

            if let Ok(resp) = client.get(&sums_url).send().await {
                if resp.status().is_success() {
                    if let Ok(sums_text) = resp.text().await {
                        for line in sums_text.lines() {
                            let mut parts = line.split_whitespace();
                            if let (Some(hash), Some(fname)) = (parts.next(), parts.next()) {
                                let clean_fname = fname.trim_start_matches('*');
                                if clean_fname.ends_with(".qcow2")
                                    && clean_fname.contains(arch_suffix)
                                {
                                    let download_url = format!(
                                        "{}{}/{}/{}",
                                        AL2023_CDN_BASE, candidate, kvm_dir, clean_fname
                                    );
                                    return Ok(ImageCatalogEntry {
                                        os: "Amazon Linux".to_string(),
                                        release: format!("Amazon Linux {}", candidate),
                                        remote: "al2023".to_string(),
                                        version: format!("{} (Kernel 6.1)", candidate),
                                        aliases: vec![
                                            format!("al2023:{}", candidate),
                                            candidate.clone(),
                                        ],
                                        arch: arch.to_string(),
                                        download_url,
                                        sha256: hash.to_lowercase(),
                                        size_bytes: 1_770_520_576,
                                        description: format!(
                                            "Amazon Linux 2023 KVM Cloud Image release {} from AWS CDN",
                                            candidate
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // If offline or CDN unavailable, check if the candidate matches a built-in image
        for entry in Self::builtin_catalog() {
            if entry.version.contains(req_ver) || entry.aliases.iter().any(|a| a == req_ver) {
                return Ok(entry);
            }
        }

        // Offline fallback for known release versions
        for candidate in &candidate_versions {
            if KNOWN_AL2023_VERSIONS.contains(&candidate.as_str()) {
                let filename = format!(
                    "al2023-kvm-{}-kernel-6.1-{}.xfs.gpt.qcow2",
                    candidate, arch_suffix
                );
                return Ok(ImageCatalogEntry {
                    os: "Amazon Linux".to_string(),
                    release: format!("Amazon Linux {}", candidate),
                    remote: "al2023".to_string(),
                    version: format!("{} (Kernel 6.1)", candidate),
                    aliases: vec![format!("al2023:{}", candidate), candidate.clone()],
                    arch: arch.to_string(),
                    download_url: format!(
                        "{}{}/{}/{}",
                        AL2023_CDN_BASE, candidate, kvm_dir, filename
                    ),
                    sha256: String::new(),
                    size_bytes: 1_770_520_576,
                    description: format!(
                        "Amazon Linux 2023 KVM Cloud Image release {} from AWS CDN",
                        candidate
                    ),
                });
            }
        }

        bail!(
            "Amazon Linux 2023 version '{}' was not found on AWS CDN (checked {}). Run 'ec2pass find --show-versions' to list available releases.",
            req_ver,
            AL2023_CDN_BASE
        )
    }

    /// Resolves an image query (`None`, `"al2023"`, `"default"`, `"minimal"`, custom `http://...`
    /// or `file://...` or local path), optionally with an explicit version override,
    /// into an `ImageCatalogEntry`.
    pub async fn resolve_image_with_version(
        &self,
        query: Option<&str>,
        version_override: Option<&str>,
    ) -> Result<ImageCatalogEntry> {
        let q = query.unwrap_or("default").trim();

        // Check if query is a direct local file path or URL
        let local_path = Path::new(q);
        if q.starts_with("http://")
            || q.starts_with("https://")
            || q.starts_with("file://")
            || local_path.exists()
        {
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

        // Determine if a specific version was requested
        let target_version = if let Some(ver) = version_override {
            Some(ver.trim().to_string())
        } else if let Some(ver) = q.strip_prefix("al2023:") {
            Some(ver.trim().to_string())
        } else if q.starts_with("2023.") && q.chars().filter(|c| *c == '.').count() >= 2 {
            Some(q.trim().to_string())
        } else {
            None
        };

        if let Some(req_ver) = target_version {
            return self.resolve_specific_version(&req_ver).await;
        }

        // Default resolution: dynamically resolve latest version from AWS CDN
        let catalog = self.fetch_catalog(true).await;
        let normalized = q.strip_prefix("al2023:").unwrap_or(q).to_lowercase();

        let search_key = if normalized.is_empty() {
            "default"
        } else {
            normalized.as_str()
        };

        for entry in catalog {
            if entry
                .aliases
                .iter()
                .any(|a| a.eq_ignore_ascii_case(search_key))
                || entry.release.eq_ignore_ascii_case(search_key)
                || entry.version.eq_ignore_ascii_case(search_key)
            {
                return Ok(entry);
            }
        }

        bail!(
            "Unable to find an image matching '{}'. Available images/aliases: al2023, al2, minimal, ubuntu, debian, rocky, fedora. Run 'ec2pass find' to list available images.",
            q
        )
    }

    /// Resolves an image query (`None`, `"al2023"`, `"default"`, `"minimal"`, custom `http://...`
    /// or `file://...` or local path) into an `ImageCatalogEntry` using the latest version as default.
    pub async fn resolve_image(&self, query: Option<&str>) -> Result<ImageCatalogEntry> {
        self.resolve_image_with_version(query, None).await
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
            on_progress("Using local Amazon Linux 2023 image...", None);
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
                // Ensure the cached image is a full valid image (real AL2023 images are > 100MB)
                // rather than an interrupted download or a 4KB stub.
                if meta.len() > 10_000_000 {
                    on_progress(
                        &format!("Found cached image '{}' in ImageVault", entry.release),
                        None,
                    );
                    return Ok(target_path);
                }
            }
            let _ = fs::remove_file(&target_path).await;
        }

        // Real network download from Amazon Linux 2023 CDN
        on_progress(
            &format!("Downloading {} from {}", entry.release, entry.download_url),
            None,
        );

        let client = reqwest::Client::new();
        let resp = client
            .get(&entry.download_url)
            .send()
            .await
            .with_context(|| format!("Failed to connect to CDN: {}", entry.download_url))?;

        if !resp.status().is_success() {
            if resp.status() == reqwest::StatusCode::FORBIDDEN
                || resp.status() == reqwest::StatusCode::NOT_FOUND
            {
                bail!(
                    "CDN returned HTTP {} for '{}'. This image or release version does not exist on AWS CDN. \
                    Run 'ec2pass find --show-versions' to list available releases.",
                    resp.status(),
                    entry.download_url
                );
            }
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

        if total_size > 0 {
            on_progress(
                &format!(
                    "Downloading {} (0B / {})",
                    entry.release,
                    ec2ps_protocol::format_bytes(total_size)
                ),
                Some(0),
            );
        }

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            hasher.update(&chunk);
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            if total_size > 0 {
                let pct = ((downloaded as f64 / total_size as f64) * 100.0).min(100.0) as u8;
                if pct > last_pct || (downloaded == total_size && pct == 100) {
                    last_pct = pct;
                    on_progress(
                        &format!(
                            "Downloading {} ({} / {})",
                            entry.release,
                            ec2ps_protocol::format_bytes(downloaded),
                            ec2ps_protocol::format_bytes(total_size)
                        ),
                        Some(pct),
                    );
                }
            } else {
                on_progress(
                    &format!(
                        "Downloading {} ({})",
                        entry.release,
                        ec2ps_protocol::format_bytes(downloaded)
                    ),
                    None,
                );
            }
        }
        file.flush().await?;
        drop(file);

        on_progress("Download complete. Verifying SHA256 checksum...", Some(100));

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
            on_progress("SHA256 verified successfully.", Some(100));
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
        assert!(minimal.release.contains("Minimal"));

        let ubuntu = vault.resolve_image(Some("ubuntu")).await.unwrap();
        assert_eq!(ubuntu.os, "Ubuntu");
        assert!(ubuntu.release.contains("Ubuntu 26.04"));

        let debian = vault.resolve_image(Some("debian")).await.unwrap();
        assert_eq!(debian.os, "Debian");
        assert!(debian.release.contains("Debian 13"));

        let rocky = vault.resolve_image(Some("rocky")).await.unwrap();
        assert_eq!(rocky.os, "Rocky Linux");

        let fedora = vault.resolve_image(Some("fedora")).await.unwrap();
        assert_eq!(fedora.os, "Fedora");

        let custom = vault
            .resolve_image(Some("https://example.com/custom.qcow2"))
            .await
            .unwrap();
        assert_eq!(custom.remote, "custom");

        // Verify that completely nonexistent aliases fail
        let nonexistent = vault.resolve_image(Some("nonexistent-os-xyz")).await;
        assert!(nonexistent.is_err());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn test_fetch_available_versions_and_catalog() {
        let tmp = std::env::temp_dir().join(format!("ec2ps_vault_test_{}", rand::random::<u32>()));
        let vault = ImageVault::new(tmp.clone()).unwrap();

        let versions = vault.fetch_available_versions().await;
        assert!(!versions.is_empty(), "Should discover AL2023 versions");
        // Verify default version is latest
        assert!(versions[0].starts_with("2023."));

        // Test extended catalog with versions
        let catalog = vault.fetch_catalog_extended(true, false).await;
        assert!(!catalog.is_empty());
        assert!(catalog[0].description.contains("[latest]"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn test_resolve_specific_version() {
        let tmp = std::env::temp_dir().join(format!("ec2ps_vault_test_{}", rand::random::<u32>()));
        let vault = ImageVault::new(tmp.clone()).unwrap();

        // Test resolve with version override
        let entry = vault
            .resolve_image_with_version(Some("al2023"), Some("2023.12.20261119.0"))
            .await
            .unwrap();
        assert!(entry.version.contains("2023.12.20261119.0"));
        assert!(entry.download_url.contains("2023.12.20261119.0"));

        // Test resolve with al2023:<version> syntax
        let entry2 = vault
            .resolve_image(Some("al2023:2023.6.20250218.2"))
            .await
            .unwrap();
        assert!(entry2.version.contains("2023.6.20250218.2"));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
