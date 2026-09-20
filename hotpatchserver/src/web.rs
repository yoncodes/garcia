use std::path::Path;

use axum::{
    body::Body,
    extract::{Path as UrlPath, State},
    http::{HeaderMap, StatusCode, header},
    response::Response,
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, SeekFrom},
};
use tokio_util::io::ReaderStream;
use tracing::{info, warn};

use crate::{AppState, cache};

pub(crate) async fn serve_asset(
    State(state): State<AppState>,
    UrlPath(path): UrlPath<String>,
    headers: HeaderMap,
) -> Response<Body> {
    let Some(relative) = cache::validated_relative_path(&path) else {
        return text_response(StatusCode::BAD_REQUEST, "invalid asset path");
    };
    let version = if let Some(version) = cache::version_from_catalog(&relative) {
        *state.version.write().expect("version lock poisoned") = version.clone();
        version
    } else {
        state.version.read().expect("version lock poisoned").clone()
    };
    let cached_target = state.assets.join(cache::cache_path(&relative, &version));
    let local_override = cache::catalog_override(&cached_target).filter(|path| path.is_file());
    let has_local_override = local_override.is_some();
    let refresh = local_override.is_none() && cache::requires_refresh(&relative);
    let target = local_override.unwrap_or(cached_target);
    let refresh_guard = if refresh {
        Some(state.refresh.lock().await)
    } else {
        None
    };
    let cached = target.is_file();
    let mut source = if has_local_override {
        "local_override"
    } else {
        "local_cache"
    };
    if !cached || refresh {
        match cache::cache_from_upstream(&state, &path, &target, refresh).await {
            Ok(()) => source = "upstream",
            Err(error) => {
                if !cached {
                    warn!(path, %error, "hotpatch cache miss failed");
                    return text_response(
                        StatusCode::BAD_GATEWAY,
                        "upstream unavailable and asset is not cached",
                    );
                }
                warn!(path, %error, "upstream unavailable; serving cached asset");
            }
        }
    }
    drop(refresh_guard);

    match serve_cached_file(&target, headers.get(header::RANGE)).await {
        Ok(response) => {
            let status = response.status().as_u16();
            let bytes = response
                .headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("unknown");
            info!(path, source, status, bytes, file = %target.display(), "served hotpatch asset");
            response
        }
        Err(error) => {
            warn!(path, %error, "cached asset read failed");
            text_response(StatusCode::INTERNAL_SERVER_ERROR, "asset read failed")
        }
    }
}

async fn serve_cached_file(
    path: &Path,
    range: Option<&axum::http::HeaderValue>,
) -> anyhow::Result<Response<Body>> {
    let mut file = File::open(path).await?;
    let size = file.metadata().await?.len();
    let requested = range
        .map(|value| value.to_str().map_err(anyhow::Error::from))
        .transpose()?
        .map(|value| parse_byte_range(value, size))
        .transpose();
    let (status, start, end) = match requested {
        Ok(Some((start, end))) => (StatusCode::PARTIAL_CONTENT, start, end),
        Ok(None) => (StatusCode::OK, 0, size.saturating_sub(1)),
        Err(()) => {
            return Ok(text_response(
                StatusCode::RANGE_NOT_SATISFIABLE,
                "invalid byte range",
            ));
        }
    };
    let length = if size == 0 { 0 } else { end - start + 1 };
    file.seek(SeekFrom::Start(start)).await?;
    let stream = ReaderStream::new(file.take(length));
    let mut builder = Response::builder()
        .status(status)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_LENGTH, length)
        .header(header::CONTENT_TYPE, "application/octet-stream");
    if status == StatusCode::PARTIAL_CONTENT {
        builder = builder.header(header::CONTENT_RANGE, format!("bytes {start}-{end}/{size}"));
    }
    Ok(builder.body(Body::from_stream(stream))?)
}

fn parse_byte_range(value: &str, size: u64) -> Result<(u64, u64), ()> {
    let value = value.strip_prefix("bytes=").ok_or(())?;
    if size == 0 || value.contains(',') {
        return Err(());
    }
    let (start, end) = value.split_once('-').ok_or(())?;
    if start.is_empty() {
        let suffix = end.parse::<u64>().map_err(|_| ())?.min(size);
        if suffix == 0 {
            return Err(());
        }
        return Ok((size - suffix, size - 1));
    }
    let start = start.parse::<u64>().map_err(|_| ())?;
    let end = if end.is_empty() {
        size - 1
    } else {
        end.parse::<u64>().map_err(|_| ())?.min(size - 1)
    };
    (start <= end && start < size)
        .then_some((start, end))
        .ok_or(())
}

fn text_response(status: StatusCode, message: &'static str) -> Response<Body> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(message))
        .expect("static response is valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ranges() {
        assert_eq!(parse_byte_range("bytes=2-5", 10), Ok((2, 5)));
        assert_eq!(parse_byte_range("bytes=-3", 10), Ok((7, 9)));
        assert_eq!(parse_byte_range("bytes=10-", 10), Err(()));
    }
}
