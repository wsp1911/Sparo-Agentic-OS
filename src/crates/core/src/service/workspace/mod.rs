//! Workspace service module
//!
//! Full workspace management system: open, manage, scan, statistics, etc.

pub mod factory;
pub mod manager;
pub mod provider;
pub mod service;

// Re-export main components
pub use factory::WorkspaceFactory;
pub use manager::{
    WorkspaceIdentity, WorkspaceInfo, WorkspaceKind, WorkspaceManager, WorkspaceManagerConfig,
    WorkspaceManagerStatistics, WorkspaceOpenOptions, WorkspaceStatus, WorkspaceSummary,
};
pub use provider::{WorkspaceCleanupResult, WorkspaceProvider, WorkspaceSystemSummary};
pub use service::{
    get_global_workspace_service, set_global_workspace_service, BatchImportResult,
    BatchRemoveResult, WorkspaceCreateOptions, WorkspaceExport, WorkspaceHealthStatus,
    WorkspaceImportResult, WorkspaceQuickSummary, WorkspaceService,
};
