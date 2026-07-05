use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RendererContract {
    pub api_version: u32,
    pub backend: RendererBackend,
    pub surface: RendererSurfaceContract,
    pub terminal: TerminalGridContract,
    pub neovim: NeovimRendererContract,
    pub cursor: CursorAnimationContract,
    pub scroll: ScrollAnimationContract,
    pub images: ImageProtocolContract,
}

impl RendererContract {
    pub fn current() -> Self {
        Self {
            api_version: 1,
            backend: RendererBackend::Metal,
            surface: RendererSurfaceContract {
                view: "MTKView".to_owned(),
                pixel_format: "bgra8Unorm".to_owned(),
                preferred_frames_per_second: 120,
            },
            terminal: TerminalGridContract {
                text_source: "rust-core-terminal-snapshot".to_owned(),
                cell_metrics_owner: "renderer".to_owned(),
                supports_dirty_rows: true,
            },
            neovim: NeovimRendererContract {
                event_source: "neovim-ext-multigrid".to_owned(),
                command_source: "rust-neovide-derived-command-batch".to_owned(),
                retained_model_ffi: "nvterm_nvim_renderer_model_json".to_owned(),
                retained_model_schema_version: 1,
                surface_input: "NeovideRendererModelSnapshot".to_owned(),
                compatibility_frame_ffi: "nvterm_nvim_frame_json".to_owned(),
                target_surface: "skia-metal".to_owned(),
            },
            cursor: CursorAnimationContract {
                style_source: "terminal-core".to_owned(),
                neovide_like_trail: true,
                duration_seconds: 0.150,
            },
            scroll: ScrollAnimationContract {
                smooth_history_scroll: true,
                output_shift_animation: true,
                spring_owner: "rust-core-scroll-model".to_owned(),
            },
            images: ImageProtocolContract {
                kitty_graphics_protocol: true,
                iterm2_inline_images: false,
                texture_owner: "renderer".to_owned(),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RendererBackend {
    Metal,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RendererSurfaceContract {
    pub view: String,
    pub pixel_format: String,
    pub preferred_frames_per_second: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TerminalGridContract {
    pub text_source: String,
    pub cell_metrics_owner: String,
    pub supports_dirty_rows: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NeovimRendererContract {
    pub event_source: String,
    pub command_source: String,
    pub retained_model_ffi: String,
    pub retained_model_schema_version: u32,
    pub surface_input: String,
    pub compatibility_frame_ffi: String,
    pub target_surface: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CursorAnimationContract {
    pub style_source: String,
    pub neovide_like_trail: bool,
    pub duration_seconds: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ScrollAnimationContract {
    pub smooth_history_scroll: bool,
    pub output_shift_animation: bool,
    pub spring_owner: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ImageProtocolContract {
    pub kitty_graphics_protocol: bool,
    pub iterm2_inline_images: bool,
    pub texture_owner: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_contract_targets_native_metal_and_kitty_images() {
        let contract = RendererContract::current();

        assert_eq!(contract.backend, RendererBackend::Metal);
        assert_eq!(contract.surface.view, "MTKView");
        assert!(contract.images.kitty_graphics_protocol);
        assert!(contract.scroll.output_shift_animation);
        assert_eq!(
            contract.neovim.retained_model_ffi,
            "nvterm_nvim_renderer_model_json"
        );
        assert_eq!(contract.neovim.retained_model_schema_version, 1);
        assert_eq!(
            contract.neovim.surface_input,
            "NeovideRendererModelSnapshot"
        );
    }
}
