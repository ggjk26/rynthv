//! UI graphics-backend configuration and live adapter diagnostics.
//! Backend selection must happen before eframe creates its wgpu instance.

use super::UiLanguage;
use eframe::CreationContext;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GraphicsBackend {
    #[default]
    Auto,
    DirectX12,
    Vulkan,
}

impl GraphicsBackend {
    pub const ALL: [Self; 3] = [Self::Auto, Self::DirectX12, Self::Vulkan];

    pub fn label(self, lang: UiLanguage) -> &'static str {
        match self {
            Self::Auto => lang.text("自动选择", "Automatic", "自動選択"),
            Self::DirectX12 => "DirectX 12",
            Self::Vulkan => "Vulkan",
        }
    }

    pub fn supported(self) -> bool {
        match self {
            Self::Auto => true,
            Self::DirectX12 => cfg!(windows),
            Self::Vulkan => cfg!(any(windows, target_os = "linux")),
        }
    }

    /// A backend not supported by the current operating system cannot be
    /// applied. A project copied from another OS remains launchable.
    pub fn on_this_platform(self) -> Self {
        if self.supported() { self } else { Self::Auto }
    }

    pub fn backends(self) -> wgpu::Backends {
        #[cfg(windows)]
        {
            match self {
                Self::DirectX12 => wgpu::Backends::DX12,
                Self::Vulkan => wgpu::Backends::VULKAN,
                Self::Auto => wgpu::Backends::DX12 | wgpu::Backends::VULKAN,
            }
        }
        #[cfg(target_os = "linux")]
        {
            match self {
                Self::Vulkan => wgpu::Backends::VULKAN,
                Self::Auto | Self::DirectX12 => wgpu::Backends::VULKAN | wgpu::Backends::GL,
            }
        }
        #[cfg(not(any(windows, target_os = "linux")))]
        {
            wgpu::Backends::PRIMARY
        }
    }

    /// Optional startup override, e.g. `--graphics=vulkan`. An invalid flag
    /// produces an error instead of silently selecting a different backend.
    pub fn parse_flag(text: &str) -> Option<Self> {
        match text.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "dx12" | "directx12" | "directx-12" => Some(Self::DirectX12),
            "vulkan" | "vk" => Some(Self::Vulkan),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct GraphicsInfo {
    /// Backend requested at application startup (not the potentially edited
    /// setting, which only takes effect on the next process launch).
    pub configured: GraphicsBackend,
    /// The persisted preference as read when this process started. A temporary
    /// CLI override must not produce a false unsaved/restart warning.
    pub preference_at_start: GraphicsBackend,
    /// Actual adapter backend reported by wgpu, not merely the request.
    pub active_backend: String,
    pub gpu_name: String,
    pub gpu_type: String,
    pub vendor_device: String,
    pub driver: String,
}

impl GraphicsInfo {
    pub fn from_context(
        cc: &CreationContext<'_>,
        configured: GraphicsBackend,
        preference_at_start: GraphicsBackend,
    ) -> Self {
        let Some(state) = cc.wgpu_render_state.as_ref() else {
            return Self {
                configured,
                preference_at_start,
                active_backend: "wgpu (unknown)".into(),
                gpu_name: "Unknown".into(),
                gpu_type: "Unknown".into(),
                vendor_device: "—".into(),
                driver: "—".into(),
            };
        };
        let adapter = state.adapter.get_info();
        let backend = match adapter.backend {
            wgpu::Backend::Dx12 => "DirectX 12",
            wgpu::Backend::Vulkan => "Vulkan",
            wgpu::Backend::Gl => "OpenGL",
            _ => "Other wgpu backend",
        };
        Self {
            configured,
            preference_at_start,
            active_backend: backend.into(),
            gpu_name: adapter.name,
            gpu_type: format!("{:?}", adapter.device_type),
            vendor_device: format!("{:04X}:{:04X}", adapter.vendor, adapter.device),
            driver: if adapter.driver.is_empty() {
                "—".into()
            } else if adapter.driver_info.is_empty() {
                adapter.driver
            } else {
                format!("{} ({})", adapter.driver, adapter.driver_info)
            },
        }
    }
}
