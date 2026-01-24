use serde::Serialize;

/// CDN dependency versions for HTML templates.
/// Managed by Renovate via regex custom manager.
#[derive(Clone, Debug, Serialize)]
pub struct CdnVersions {
    pub highlightjs: &'static str,
    pub mermaid: &'static str,
    pub nord_highlightjs: &'static str,
    pub fusejs: &'static str,
}

pub const CDN_VERSIONS: CdnVersions = CdnVersions {
    highlightjs: "11.11.1",    // renovate: datasource=npm depName=highlight.js
    mermaid: "11.12.2",         // renovate: datasource=npm depName=mermaid
    nord_highlightjs: "0.2.0", // renovate: datasource=npm depName=nord-highlightjs
    fusejs: "7.1.0",           // renovate: datasource=npm depName=fuse.js
};
