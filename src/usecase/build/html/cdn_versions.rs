use serde::Serialize;

/// CDN dependency versions for HTML templates.
/// Managed by Renovate via regex custom manager.
#[derive(Clone, Debug, Serialize)]
pub struct CdnVersions {
    pub highlightjs: &'static str,
    pub mermaid: &'static str,
    pub fusejs: &'static str,
}

pub const CDN_VERSIONS: CdnVersions = CdnVersions {
    highlightjs: "11.12.0", // renovate: datasource=npm depName=highlight.js
    mermaid: "11.17.2",     // renovate: datasource=npm depName=mermaid
    fusejs: "7.5.0",        // renovate: datasource=npm depName=fuse.js
};
