use std::{
    io,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tracing::info;

use crate::AppState;

static NEXT_TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);

struct TemporaryFile(PathBuf);

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

pub(crate) fn validated_relative_path(path: &str) -> Option<PathBuf> {
    let path = Path::new(path);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return None;
    }
    Some(path.to_owned())
}

pub(crate) fn requires_refresh(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("raw_catalog.hash" | "raw_catalog.csv")
    )
}

pub(crate) fn catalog_override(path: &Path) -> Option<PathBuf> {
    let override_name = match path.file_name()?.to_str()? {
        "raw_catalog.csv" => "raw_catalog.override.csv",
        "raw_catalog.hash" => "raw_catalog.override.hash",
        _ => return None,
    };
    Some(path.with_file_name(override_name))
}

pub(crate) fn version_from_catalog(remote: &Path) -> Option<String> {
    let name = remote.file_name()?.to_str()?;
    let stem = name
        .strip_suffix(".hash")
        .or_else(|| name.strip_suffix(".json"))?;
    let version = stem.strip_prefix("catalog_")?;
    (!version.is_empty()
        && version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte)))
    .then(|| version.to_owned())
}

pub(crate) fn latest_cached_version(assets: &Path) -> Option<String> {
    std::fs::read_dir(assets.join("hotpatch/android/versions"))
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .max()
}

pub(crate) fn cache_path(remote: &Path, version: &str) -> PathBuf {
    let name = remote
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let root = Path::new("hotpatch/android");

    if name == "raw_catalog.hash" || name == "raw_catalog.csv" {
        return root
            .join("versions")
            .join(version)
            .join("catalogs")
            .join(name);
    }
    if name.starts_with("catalog_") && (name.ends_with(".hash") || name.ends_with(".json")) {
        return root
            .join("versions")
            .join(version)
            .join("catalogs")
            .join(name);
    }
    if name.len() == 32 && name.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return root.join("objects/raw").join(&name[..2]).join(name);
    }
    if let Some(hash) = name
        .strip_suffix(".bundle")
        .and_then(|stem| stem.rsplit('_').next())
        .filter(|hash| hash.len() == 32 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
    {
        return root.join("objects/bundles").join(&hash[..2]).join(name);
    }
    root.join("versions")
        .join(version)
        .join("files")
        .join(remote)
}

pub(crate) async fn cache_from_upstream(
    state: &AppState,
    path: &str,
    target: &Path,
    replace_existing: bool,
) -> anyhow::Result<()> {
    let url = format!("{}/{}", state.upstream, path.replace('\\', "/"));
    info!(path, url, "fetching hotpatch asset from upstream");
    let mut upstream = state.client.get(&url).send().await?.error_for_status()?;
    let parent = target.parent().expect("validated asset path has a parent");
    fs::create_dir_all(parent).await?;
    let temporary = TemporaryFile(target.with_extension(format!(
        "part-{}-{}",
        std::process::id(),
        NEXT_TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed)
    )));
    let mut file = File::create(&temporary.0).await?;
    let mut downloaded_bytes = 0_u64;
    while let Some(chunk) = upstream.chunk().await? {
        downloaded_bytes += chunk.len() as u64;
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    drop(file);
    let changed = publish_download(&temporary.0, target, replace_existing).await?;
    info!(path, downloaded_bytes, changed, file = %target.display(), "finished fetching hotpatch asset from upstream");
    Ok(())
}

pub(crate) fn remove_stale_downloads(directory: &Path) -> io::Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            remove_stale_downloads(&path)?;
        } else if entry.file_name().to_string_lossy().contains(".part-") {
            std::fs::remove_file(path)?;
        }
    }
    Ok(())
}

async fn publish_download(
    temporary: &Path,
    target: &Path,
    replace_existing: bool,
) -> anyhow::Result<bool> {
    if replace_existing && target.is_file() {
        if fs::read(temporary).await? == fs::read(target).await? {
            fs::remove_file(temporary).await?;
            return Ok(false);
        }
        let backup = target.with_extension(format!(
            "old-{}-{}",
            std::process::id(),
            NEXT_TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::rename(target, &backup).await?;
        if let Err(error) = fs::rename(temporary, target).await {
            let _ = fs::rename(&backup, target).await;
            return Err(error.into());
        }
        fs::remove_file(backup).await?;
        return Ok(true);
    }
    match fs::rename(temporary, target).await {
        Ok(()) => Ok(true),
        Err(_) if target.is_file() => {
            fs::remove_file(temporary).await?;
            Ok(false)
        }
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_paths() {
        assert_eq!(
            validated_relative_path("raw_catalog.hash"),
            Some("raw_catalog.hash".into())
        );
        assert_eq!(validated_relative_path("../secret"), None);
    }

    #[test]
    fn categorizes_assets() {
        assert_eq!(
            cache_path(Path::new("raw_catalog.csv"), "2026.08.03"),
            Path::new("hotpatch/android/versions/2026.08.03/catalogs/raw_catalog.csv")
        );
        assert_eq!(
            cache_path(Path::new("89c58f663376fb04a8c199a833046089"), "2026.08.03"),
            Path::new("hotpatch/android/objects/raw/89/89c58f663376fb04a8c199a833046089")
        );
        assert_eq!(
            cache_path(
                Path::new("ui_0123456789abcdef0123456789abcdef.bundle"),
                "2026.08.03"
            ),
            Path::new(
                "hotpatch/android/objects/bundles/01/ui_0123456789abcdef0123456789abcdef.bundle"
            )
        );
        assert_eq!(
            version_from_catalog(Path::new("catalog_2026.08.03.07.45.42.hash")),
            Some("2026.08.03.07.45.42".into())
        );
        assert_eq!(
            catalog_override(Path::new("cache/raw_catalog.csv")),
            Some(Path::new("cache/raw_catalog.override.csv").into())
        );
        assert_eq!(catalog_override(Path::new("cache/catalog.json")), None);
    }
}
