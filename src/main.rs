#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Typesafe Editor
use crossbeam_channel::{unbounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use image;
use egui::{
    Align, Color32, FontId, Frame, Layout, Rounding, Stroke, TextStyle, Vec2, Visuals,
};
use egui::epaint::Shadow;
use egui::text::{CCursor, CCursorRange};
use eframe::egui;
use sha2::{Digest, Sha256};
use pdfium_render::prelude::*;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Style};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use flate2::read::GzDecoder;
use std::io::Read;
use std::io::Write;
use unicode_segmentation::UnicodeSegmentation;

const INDENT_UNIT: &str = "    ";

// Themes

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemePreset {
    // Modern
    Serendipity,
    TokyoNight,
    Nord,
    Catppuccin,
    GruvboxDark,
    OneDark,
    // Classic
    SolarizedDark,
    SolarizedLight,
    Dracula,
    Light,
    Dark,
}

impl ThemePreset {
    pub fn name(&self) -> &'static str {
        match self {
            ThemePreset::Serendipity => "Serendipity",
            ThemePreset::TokyoNight => "Tokyo Night",
            ThemePreset::Nord => "Nord",
            ThemePreset::Catppuccin => "Catppuccin",
            ThemePreset::GruvboxDark => "Gruvbox Dark",
            ThemePreset::OneDark => "One Dark",
            ThemePreset::SolarizedDark => "Solarized Dark",
            ThemePreset::SolarizedLight => "Solarized Light",
            ThemePreset::Dracula => "Dracula",
            ThemePreset::Light => "Light",
            ThemePreset::Dark => "Dark",
        }
    }

    pub fn all() -> &'static [ThemePreset] {
        &[
            ThemePreset::Serendipity,
            ThemePreset::TokyoNight,
            ThemePreset::Nord,
            ThemePreset::Catppuccin,
            ThemePreset::GruvboxDark,
            ThemePreset::OneDark,
            ThemePreset::SolarizedDark,
            ThemePreset::SolarizedLight,
            ThemePreset::Dracula,
            ThemePreset::Light,
            ThemePreset::Dark,
        ]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub bg: Color32,              // Main background
    pub bg_secondary: Color32,    // Secondary background (panels)
    pub bg_tertiary: Color32,     // Tertiary background (fields, buttons)
    pub text_primary: Color32,    // Primary text
    pub text_secondary: Color32,  // Secondary text (labels, hints)
    pub border: Color32,          // Border color
    pub accent: Color32,          // Accent color
    pub accent_hover: Color32,    // Accent hover
    pub error: Color32,           // Error/red
    pub warning: Color32,         // Warning/yellow
    pub success: Color32,         // Success/green
    pub disabled: Color32,        // Disabled text
}

impl ThemeColors {
    pub fn from_preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::Serendipity => Self::serendipity(),
            ThemePreset::TokyoNight => Self::tokyo_night(),
            ThemePreset::Nord => Self::nord(),
            ThemePreset::Catppuccin => Self::catppuccin(),
            ThemePreset::GruvboxDark => Self::gruvbox_dark(),
            ThemePreset::OneDark => Self::one_dark(),
            ThemePreset::SolarizedDark => Self::solarized_dark(),
            ThemePreset::SolarizedLight => Self::solarized_light(),
            ThemePreset::Dracula => Self::dracula(),
            ThemePreset::Light => Self::light(),
            ThemePreset::Dark => Self::dark(),
        }
    }

    // Serendipity
    fn serendipity() -> Self {
        Self {
            bg: Color32::from_rgb(0x1a, 0x1f, 0x3a),           // Deep navy
            bg_secondary: Color32::from_rgb(0x24, 0x2a, 0x4a), // Navy blue
            bg_tertiary: Color32::from_rgb(0x2d, 0x33, 0x55),  // Lighter navy
            text_primary: Color32::from_rgb(0xf0, 0xf0, 0xf5), // Almost white
            text_secondary: Color32::from_rgb(0xa0, 0xa5, 0xb8), // Light gray-blue
            border: Color32::from_rgb(0x3d, 0x43, 0x65),       // Purple-ish border
            accent: Color32::from_rgb(0x6c, 0x9f, 0xff),       // Vibrant blue
            accent_hover: Color32::from_rgb(0x8b, 0xb8, 0xff), // Lighter blue
            error: Color32::from_rgb(0xff, 0x6b, 0x6b),        // Coral red
            warning: Color32::from_rgb(0xff, 0xc9, 0x42),      // Amber
            success: Color32::from_rgb(0x51, 0xcf, 0x66),      // Soft green
            disabled: Color32::from_rgb(0x60, 0x65, 0x78),     // Muted gray
        }
    }

    // Tokyo Night
    fn tokyo_night() -> Self {
        Self {
            bg: Color32::from_rgb(0x1a, 0x1b, 0x26),
            bg_secondary: Color32::from_rgb(0x24, 0x28, 0x3b),
            bg_tertiary: Color32::from_rgb(0x2e, 0x30, 0x46),
            text_primary: Color32::from_rgb(0xc0, 0xcf, 0xf7),
            text_secondary: Color32::from_rgb(0x86, 0x8e, 0xb8),
            border: Color32::from_rgb(0x3b, 0x40, 0x58),
            accent: Color32::from_rgb(0x7a, 0xa2, 0xf7),       // Periwinkle blue
            accent_hover: Color32::from_rgb(0x9f, 0xc8, 0xff),
            error: Color32::from_rgb(0xf7, 0x76, 0x8e),
            warning: Color32::from_rgb(0xe0, 0xaf, 0x68),
            success: Color32::from_rgb(0x9e, 0xcd, 0xc8),
            disabled: Color32::from_rgb(0x54, 0x56, 0x6a),
        }
    }

    // Nord
    fn nord() -> Self {
        Self {
            bg: Color32::from_rgb(0x2e, 0x34, 0x40),
            bg_secondary: Color32::from_rgb(0x3b, 0x42, 0x52),
            bg_tertiary: Color32::from_rgb(0x43, 0x4c, 0x5e),
            text_primary: Color32::from_rgb(0xec, 0xef, 0xf4),
            text_secondary: Color32::from_rgb(0xd0, 0xd4, 0xde),
            border: Color32::from_rgb(0x4c, 0x56, 0x6a),
            accent: Color32::from_rgb(0x88, 0xc0, 0xd0),       // Frost cyan
            accent_hover: Color32::from_rgb(0xa3, 0xd5, 0xdd),
            error: Color32::from_rgb(0xbf, 0x61, 0x6b),
            warning: Color32::from_rgb(0xeb, 0xcb, 0x8b),
            success: Color32::from_rgb(0xa3, 0xbe, 0x8c),
            disabled: Color32::from_rgb(0x54, 0x5e, 0x6b),
        }
    }

    // Catppuccin
    fn catppuccin() -> Self {
        Self {
            bg: Color32::from_rgb(0x1e, 0x1e, 0x2e),
            bg_secondary: Color32::from_rgb(0x2d, 0x2d, 0x40),
            bg_tertiary: Color32::from_rgb(0x36, 0x36, 0x4f),
            text_primary: Color32::from_rgb(0xcd, 0xe6, 0xf6),
            text_secondary: Color32::from_rgb(0xa5, 0xad, 0xc8),
            border: Color32::from_rgb(0x44, 0x44, 0x5a),
            accent: Color32::from_rgb(0x89, 0xdc, 0xeb),       // Sky blue
            accent_hover: Color32::from_rgb(0xa5, 0xf3, 0xfc),
            error: Color32::from_rgb(0xf3, 0x85, 0x8e),
            warning: Color32::from_rgb(0xf9, 0xe2, 0xaf),
            success: Color32::from_rgb(0xa6, 0xe3, 0xa1),
            disabled: Color32::from_rgb(0x58, 0x58, 0x6c),
        }
    }

    // Gruvbox Dark
    fn gruvbox_dark() -> Self {
        Self {
            bg: Color32::from_rgb(0x28, 0x28, 0x28),
            bg_secondary: Color32::from_rgb(0x32, 0x30, 0x2f),
            bg_tertiary: Color32::from_rgb(0x3c, 0x38, 0x36),
            text_primary: Color32::from_rgb(0xeb, 0xdb, 0xb2),
            text_secondary: Color32::from_rgb(0xa8, 0x99, 0x84),
            border: Color32::from_rgb(0x50, 0x49, 0x45),
            accent: Color32::from_rgb(0x83, 0xa6, 0x98),       // Aqua
            accent_hover: Color32::from_rgb(0xa8, 0xd0, 0xc8),
            error: Color32::from_rgb(0xfb, 0x49, 0x34),
            warning: Color32::from_rgb(0xfe, 0xb0, 0x27),
            success: Color32::from_rgb(0xb8, 0xbb, 0x26),
            disabled: Color32::from_rgb(0x68, 0x60, 0x59),
        }
    }

    // OneDark
    fn one_dark() -> Self {
        Self {
            bg: Color32::from_rgb(0x28, 0x2c, 0x34),
            bg_secondary: Color32::from_rgb(0x32, 0x37, 0x44),
            bg_tertiary: Color32::from_rgb(0x3e, 0x44, 0x52),
            text_primary: Color32::from_rgb(0xab, 0xb2, 0xbf),
            text_secondary: Color32::from_rgb(0x82, 0x8a, 0x97),
            border: Color32::from_rgb(0x4a, 0x53, 0x62),
            accent: Color32::from_rgb(0x61, 0xaf, 0xef),       // Blue
            accent_hover: Color32::from_rgb(0x88, 0xc6, 0xff),
            error: Color32::from_rgb(0xe0, 0x6c, 0x75),
            warning: Color32::from_rgb(0xe5, 0xc0, 0x7b),
            success: Color32::from_rgb(0x98, 0xc3, 0x79),
            disabled: Color32::from_rgb(0x59, 0x60, 0x6b),
        }
    }

    // Solarized Dark
    fn solarized_dark() -> Self {
        Self {
            bg: Color32::from_rgb(0x00, 0x2b, 0x36),
            bg_secondary: Color32::from_rgb(0x01, 0x32, 0x3c),
            bg_tertiary: Color32::from_rgb(0x07, 0x36, 0x42),
            text_primary: Color32::from_rgb(0x93, 0xa1, 0xa1),
            text_secondary: Color32::from_rgb(0x65, 0x7b, 0x83),
            border: Color32::from_rgb(0x07, 0x36, 0x42),
            accent: Color32::from_rgb(0x26, 0x8b, 0xd2),       // Blue
            accent_hover: Color32::from_rgb(0x52, 0xb5, 0xff),
            error: Color32::from_rgb(0xdc, 0x32, 0x2f),
            warning: Color32::from_rgb(0xb5, 0x89, 0x00),
            success: Color32::from_rgb(0x85, 0x99, 0x00),
            disabled: Color32::from_rgb(0x58, 0x6e, 0x75),
        }
    }

    // Solarized Light
    fn solarized_light() -> Self {
        Self {
            bg: Color32::from_rgb(0xfd, 0xf6, 0xe3),
            bg_secondary: Color32::from_rgb(0xee, 0xe8, 0xd5),
            bg_tertiary: Color32::from_rgb(0xe5, 0xdc, 0xc8),
            text_primary: Color32::from_rgb(0x65, 0x7b, 0x83),
            text_secondary: Color32::from_rgb(0x93, 0xa1, 0xa1),
            border: Color32::from_rgb(0xd6, 0xce, 0xbf),
            accent: Color32::from_rgb(0x26, 0x8b, 0xd2),
            accent_hover: Color32::from_rgb(0x52, 0xb5, 0xff),
            error: Color32::from_rgb(0xdc, 0x32, 0x2f),
            warning: Color32::from_rgb(0xb5, 0x89, 0x00),
            success: Color32::from_rgb(0x85, 0x99, 0x00),
            disabled: Color32::from_rgb(0xa5, 0xaa, 0xb0),
        }
    }

    // Dracula
    fn dracula() -> Self {
        Self {
            bg: Color32::from_rgb(0x28, 0x2a, 0x36),
            bg_secondary: Color32::from_rgb(0x32, 0x34, 0x3f),
            bg_tertiary: Color32::from_rgb(0x3d, 0x3f, 0x4a),
            text_primary: Color32::from_rgb(0xf8, 0xf8, 0xf2),
            text_secondary: Color32::from_rgb(0xbd, 0xbe, 0xdb),
            border: Color32::from_rgb(0x44, 0x47, 0x55),
            accent: Color32::from_rgb(0x8b, 0xe9, 0xfd),       // Cyan
            accent_hover: Color32::from_rgb(0xa3, 0xf7, 0xff),
            error: Color32::from_rgb(0xff, 0x55, 0x55),
            warning: Color32::from_rgb(0xf1, 0xfa, 0x8c),
            success: Color32::from_rgb(0x50, 0xfa, 0x7b),
            disabled: Color32::from_rgb(0x62, 0x65, 0x7e),
        }
    }

    // Light
    fn light() -> Self {
        Self {
            bg: Color32::from_rgb(0xf5, 0xf5, 0xf5),
            bg_secondary: Color32::from_rgb(0xff, 0xff, 0xff),
            bg_tertiary: Color32::from_rgb(0xf0, 0xf0, 0xf0),
            text_primary: Color32::from_rgb(0x33, 0x33, 0x33),
            text_secondary: Color32::from_rgb(0x66, 0x66, 0x66),
            border: Color32::from_rgb(0xdd, 0xdd, 0xdd),
            accent: Color32::from_rgb(0x00, 0x98, 0xff),
            accent_hover: Color32::from_rgb(0x33, 0xb1, 0xff),
            error: Color32::from_rgb(0xff, 0x33, 0x33),
            warning: Color32::from_rgb(0xff, 0xaa, 0x00),
            success: Color32::from_rgb(0x33, 0xcc, 0x33),
            disabled: Color32::from_rgb(0x99, 0x99, 0x99),
        }
    }

    // Dark
    fn dark() -> Self {
        Self {
            bg: Color32::from_rgb(0x1e, 0x1e, 0x1e),
            bg_secondary: Color32::from_rgb(0x2d, 0x2d, 0x2d),
            bg_tertiary: Color32::from_rgb(0x38, 0x38, 0x38),
            text_primary: Color32::from_rgb(0xe0, 0xe0, 0xe0),
            text_secondary: Color32::from_rgb(0xa0, 0xa0, 0xa0),
            border: Color32::from_rgb(0x42, 0x42, 0x42),
            accent: Color32::from_rgb(0x00, 0xa8, 0xff),
            accent_hover: Color32::from_rgb(0x33, 0xb8, 0xff),
            error: Color32::from_rgb(0xff, 0x66, 0x66),
            warning: Color32::from_rgb(0xff, 0xbb, 0x33),
            success: Color32::from_rgb(0x66, 0xdd, 0x66),
            disabled: Color32::from_rgb(0x66, 0x66, 0x66),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccentColor {
    Blue,
    Purple,
    Pink,
    Orange,
    Green,
    Cyan,
    Red,
}

impl AccentColor {
    pub fn name(&self) -> &'static str {
        match self {
            AccentColor::Blue => "Blue",
            AccentColor::Purple => "Purple",
            AccentColor::Pink => "Pink",
            AccentColor::Orange => "Orange",
            AccentColor::Green => "Green",
            AccentColor::Cyan => "Cyan",
            AccentColor::Red => "Red",
        }
    }

    pub fn all() -> &'static [AccentColor] {
        &[
            AccentColor::Blue,
            AccentColor::Purple,
            AccentColor::Pink,
            AccentColor::Orange,
            AccentColor::Green,
            AccentColor::Cyan,
            AccentColor::Red,
        ]
    }

    pub fn color(&self) -> Color32 {
        match self {
            AccentColor::Blue => Color32::from_rgb(0x6c, 0x9f, 0xff),
            AccentColor::Purple => Color32::from_rgb(0xc5, 0x7f, 0xff),
            AccentColor::Pink => Color32::from_rgb(0xff, 0x6b, 0xb6),
            AccentColor::Orange => Color32::from_rgb(0xff, 0x9f, 0x43),
            AccentColor::Green => Color32::from_rgb(0x51, 0xcf, 0x66),
            AccentColor::Cyan => Color32::from_rgb(0x00, 0xff, 0xff),
            AccentColor::Red => Color32::from_rgb(0xff, 0x6b, 0x6b),
        }
    }
}

// Compilation Messages

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PdfFitMode {
    Normal,
    FitWidth,
    FitPage,
}

#[derive(Clone, Copy, PartialEq)]
enum CurrentFileType {
    Tex,
    Pdf,
    Markdown,
    Other,
}

#[derive(Clone, Debug)]
struct Diagnostic {
    message: String,
    line: usize,
    #[allow(dead_code)]
    file: String,
}

enum CompilationMsg {
    Start,
    #[allow(dead_code)]
    Log(String),
    Diagnostics(Vec<Diagnostic>),
    Success(PathBuf, String, String),
    Error(String),
}

// Settings and App State

#[derive(Clone, PartialEq, Default)]
enum SettingsTab {
    #[default]
    Appearance,
    Editor,
    Permissions,
    APIs,
    About,
    License,
}

#[derive(Clone, Serialize, Deserialize)]
struct Settings {
    pub theme: ThemePreset,
    pub accent: AccentColor,
    #[serde(skip)]
    pub show_settings_window: bool,
    #[serde(skip)]
    pub active_tab: SettingsTab,
    pub auto_compile: bool,
    #[serde(default)]
    pub last_file: Option<String>,
    #[serde(default = "default_true")]
    pub autosave_timer: bool,
    #[serde(default = "default_true")]
    pub autosave_on_compile: bool,
    #[serde(default = "default_true")]
    pub autosave_on_change: bool,
}

fn default_true() -> bool { true }

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemePreset::Dark,
            accent: AccentColor::Orange,
            show_settings_window: false,
            active_tab: SettingsTab::Appearance,
            auto_compile: true,
            last_file: None,
            autosave_timer: true,
            autosave_on_compile: true,
            autosave_on_change: true,
        }
    }
}

impl Settings {
    fn path() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "typesafe", "typesafe") {
            let config_dir = proj_dirs.config_dir();
            if !config_dir.exists() {
                let _ = std::fs::create_dir_all(config_dir);
            }
            config_dir.join("settings.json")
        } else {
            PathBuf::from("settings.json")
        }
    }

    fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(settings) = serde_json::from_str(&content) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    fn save(&self) {
        let path = Self::path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, content);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum NodeKind {
    Section,
    Figure,
    Table,
    Theorem,
    #[allow(dead_code)]
    Citation,
    Unknown,
}

#[derive(Clone, Debug)]
struct StructureNode {
    label: String,
    file_path: String,
    line: usize,
    level: usize,
    kind: NodeKind,
    children: Vec<StructureNode>,
    expanded: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LatexItem {
    trigger: String,
    completion: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LatexData {
    commands: Vec<LatexItem>,
    environments: Vec<LatexItem>,
}

struct TypesafeApp {
    // Editor
    editor_content: String,
    file_path: String,
    current_dir: std::path::PathBuf,
    root_file: Option<String>,
    show_file_panel: bool,
    show_preview_panel: bool,
    last_window_width: f32,
    outline_nodes: Vec<StructureNode>,
    labels: Vec<String>,
    bib_items: Vec<String>,
    context_menu_word: Option<String>,
    context_menu_suggestions: Vec<String>,
    context_menu_replace_range: Option<std::ops::Range<usize>>,
    dictionary: std::collections::HashSet<String>,
    user_dictionary: std::collections::HashSet<String>,
    ignored_words: std::collections::HashSet<String>,
    synonym_cache: std::collections::HashMap<String, Vec<String>>,
    pending_synonyms: std::collections::HashSet<String>,
    synonym_rx: Receiver<(String, Vec<String>)>,
    synonym_tx: Sender<(String, Vec<String>)>,
    is_dirty: bool,

    // Debounced Diagnostics
    last_edit_time: f64,
    checks_dirty: bool,
    cached_syntax_errors: Vec<std::ops::Range<usize>>,
    cached_spell_errors: Vec<std::ops::Range<usize>>,

    // Command Palette
    show_command_palette: bool,
    cmd_query: String,
    cmd_selected_index: usize,

    // Syntax Highlighting
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,

    // Preview
    preview_status: String,
    pdf_textures: std::collections::HashMap<usize, egui::TextureHandle>,
    preview_size: Option<[usize; 2]>,
    pdf_path: Option<PathBuf>,
    pdfium: Option<Pdfium>,
    page_count: usize,
    current_page: usize,
    zoom: f32,
    #[allow(dead_code)]
    fit_mode: PdfFitMode,
    #[allow(dead_code)]
    magnifier_enabled: bool,
    #[allow(dead_code)]
    magnifier_zoom: f32,
    #[allow(dead_code)]
    magnifier_size: f32,

    // Pop-out state
    popout_zoom: f32,
    popout_fit_mode: PdfFitMode,
    popout_multi_page_view: bool,

    // SyncTeX
    page_sizes: std::collections::HashMap<usize, (f32, f32)>,
    pending_scroll_target: Option<(usize, f32)>,
    pending_cursor_scroll: Option<usize>,

    // Compilation
    compilation_log: String,
    show_log: bool,
    is_compiling: bool,
    compile_rx: Receiver<CompilationMsg>,
    compile_tx: Sender<CompilationMsg>,
    pending_autocompile: bool,
    diagnostics: Vec<Diagnostic>,

    // UI state
    settings: Settings,

    // Autocomplete state
    completion_suggestions: Vec<(String, String)>,
    show_completions: bool,
    completion_popup_pos: egui::Pos2,
    completion_popup_rect: Option<egui::Rect>,
    completion_selected_index: usize,

    // Search and Replace
    show_search: bool,
    search_query: String,
    replace_query: String,
    search_case_sensitive: bool,
    search_whole_word: bool,
    search_matches: Vec<(usize, usize)>,
    search_match_index: usize,
    last_save_time: f64,
    latex_commands: Vec<LatexItem>,
    latex_environments: Vec<LatexItem>,

    // Documentation
    readme_content: &'static str,
    license_content: &'static str,
    markdown_cache: egui_commonmark::CommonMarkCache,

    // File type tracking
    current_file_type: CurrentFileType,

    // Pop-out PDF viewer
    show_pdf_popup: bool,
    pdf_popup_viewport_id: egui::ViewportId,
    pdf_multi_page_view: bool,

    // PDF search
    pdf_search_query: String,
    show_pdf_search: bool,

    // Compilation Optimization
    last_bcf_hash: String,
    last_bib_hash: String,

    // File Management
    rename_dialog_open: bool,
    rename_target_path: std::path::PathBuf,
    rename_new_name: String,
}

impl Default for TypesafeApp {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        let (syn_tx, syn_rx) = unbounded();

        // Load syntax highlighting data
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        // Ensure the latest latex_data.json is embedded, but allow runtime override
        // Order: exe directory -> current working directory -> embedded
        let latex_data: LatexData = {
            #[allow(unused_assignments)]
            let mut source = "embedded".to_string();
            let log_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("typesafe_debug.log")))
                .unwrap_or_else(|| std::path::PathBuf::from("typesafe_debug.log"));
            let log_debug = |msg: &str| {
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path)
                {
                    let _ = writeln!(f, "{}", msg);
                }
            };
            // 1) Try alongside the executable
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let candidate = exe_dir.join("latex_data.json");
                    let root_candidate = exe_dir.join("../../latex_data.json");

                    if candidate.exists() {
                        source = format!("exe_dir:{:?}", candidate);
                        println!("Loading latex_data.json from exe dir: {:?}", candidate);
                        if let Ok(content) = std::fs::read_to_string(&candidate) {
                            if let Ok(data) = serde_json::from_str::<LatexData>(&content) {
                                let samples = data.commands.iter().take(3).map(|c| c.trigger.clone()).collect::<Vec<_>>().join(", ");
                                log_debug(&format!("latex_data source={} count_cmds={} count_envs={} samples=[{}]", source, data.commands.len(), data.environments.len(), samples));
                                data
                            } else {
                                println!("Failed to parse {:?}, falling back to other sources", candidate);
                                let data: LatexData = serde_json::from_str::<LatexData>(include_str!("../latex_data.json"))
                                    .expect("Failed to parse embedded latex_data.json");
                                let samples = data.commands.iter().take(3).map(|c| c.trigger.clone()).collect::<Vec<_>>().join(", ");
                                log_debug(&format!("latex_data source={} count_cmds={} count_envs={} samples=[{}]", "embedded", data.commands.len(), data.environments.len(), samples));
                                data
                            }
                        } else {
                            let data: LatexData = serde_json::from_str(include_str!("../latex_data.json"))
                                .expect("Failed to parse embedded latex_data.json");
                            let samples = data.commands.iter().take(3).map(|c| c.trigger.clone()).collect::<Vec<_>>().join(", ");
                            log_debug(&format!("latex_data source={} count_cmds={} count_envs={} samples=[{}]", "embedded", data.commands.len(), data.environments.len(), samples));
                            data
                        }
                    } else if root_candidate.exists() {
                         source = format!("root_dir:{:?}", root_candidate);
                         println!("Loading latex_data.json from root dir: {:?}", root_candidate);
                         if let Ok(content) = std::fs::read_to_string(&root_candidate) {
                             if let Ok(data) = serde_json::from_str::<LatexData>(&content) {
                                 let samples = data.commands.iter().take(3).map(|c| c.trigger.clone()).collect::<Vec<_>>().join(", ");
                                 log_debug(&format!("latex_data source={} count_cmds={} count_envs={} samples=[{}]", source, data.commands.len(), data.environments.len(), samples));
                                 data
                             } else {
                                 let data: LatexData = serde_json::from_str(include_str!("../latex_data.json")).expect("Failed to parse embedded");
                                 data
                             }
                         } else {
                             let data: LatexData = serde_json::from_str(include_str!("../latex_data.json")).expect("Failed to parse embedded");
                             data
                         }
                    } else if let Ok(content) = std::fs::read_to_string("latex_data.json") {
                        source = "cwd".to_string();
                        println!("Loading latex_data.json from current dir: {:?}", std::env::current_dir());
                        let data: LatexData = serde_json::from_str::<LatexData>(&content).unwrap_or_else(|e| {
                            println!("Failed to parse file, falling back to embedded. Error: {}", e);
                            source = "embedded".to_string();
                            serde_json::from_str::<LatexData>(include_str!("../latex_data.json"))
                                .expect("Failed to parse embedded latex_data.json")
                        });
                        let samples = data.commands.iter().take(3).map(|c| c.trigger.clone()).collect::<Vec<_>>().join(", ");
                        log_debug(&format!("latex_data source={} count_cmds={} count_envs={} samples=[{}]", source, data.commands.len(), data.environments.len(), samples));
                        data
                    } else {
                        println!("latex_data.json not found, falling back to embedded");
                        let data: LatexData = serde_json::from_str(include_str!("../latex_data.json"))
                            .expect("Failed to parse embedded latex_data.json");
                        let samples = data.commands.iter().take(3).map(|c| c.trigger.clone()).collect::<Vec<_>>().join(", ");
                        log_debug(&format!("latex_data source={} count_cmds={} count_envs={} samples=[{}]", "embedded", data.commands.len(), data.environments.len(), samples));
                        data
                    }
                } else {
                    let data: LatexData = serde_json::from_str::<LatexData>(include_str!("../latex_data.json"))
                        .expect("Failed to parse embedded latex_data.json");
                    let samples = data.commands.iter().take(3).map(|c| c.trigger.clone()).collect::<Vec<_>>().join(", ");
                    log_debug(&format!("latex_data source={} count_cmds={} count_envs={} samples=[{}]", "embedded", data.commands.len(), data.environments.len(), samples));
                    data
                }
            } else if let Ok(content) = std::fs::read_to_string("latex_data.json") {
                source = "cwd".to_string();
                println!("Loading latex_data.json from current dir: {:?}", std::env::current_dir());
                let data: LatexData = serde_json::from_str::<LatexData>(&content).unwrap_or_else(|e| {
                    println!("Failed to parse file, falling back to embedded. Error: {}", e);
                    source = "embedded".to_string();
                    serde_json::from_str::<LatexData>(include_str!("../latex_data.json"))
                        .expect("Failed to parse embedded latex_data.json")
                });
                let samples = data.commands.iter().take(3).map(|c| c.trigger.clone()).collect::<Vec<_>>().join(", ");
                log_debug(&format!("latex_data source={} count_cmds={} count_envs={} samples=[{}]", source, data.commands.len(), data.environments.len(), samples));
                data
            } else {
                println!("latex_data.json not found, falling back to embedded");
                let data: LatexData = serde_json::from_str::<LatexData>(include_str!("../latex_data.json"))
                    .expect("Failed to parse embedded latex_data.json");
                let samples = data.commands.iter().take(3).map(|c| c.trigger.clone()).collect::<Vec<_>>().join(", ");
                log_debug(&format!("latex_data source={} count_cmds={} count_envs={} samples=[{}]", "embedded", data.commands.len(), data.environments.len(), samples));
                data
            }
        };
        println!("Loaded {} commands and {} environments", latex_data.commands.len(), latex_data.environments.len());

        let settings = Settings::load();

        let mut current_dir = if std::path::Path::new("examples").exists() {
            std::path::PathBuf::from("examples")
        } else {
            std::path::PathBuf::from(".")
        };

        // Try to find a default tex file
        let mut default_file = "test.tex".to_string();
        let mut found_file = false;

        if let Some(last) = &settings.last_file {
            let path = std::path::Path::new(last);
            if path.exists() {
                default_file = last.clone();
                found_file = true;
                if let Some(parent) = path.parent() {
                    current_dir = parent.to_path_buf();
                }
            }
        }

        if !found_file {
            if let Ok(entries) = std::fs::read_dir(&current_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "tex") {
                        default_file = path.to_string_lossy().to_string();
                        found_file = true;
                        break;
                    }
                }
            }
        }

        let default_content = std::fs::read_to_string(&default_file).unwrap_or_else(|_| {
            "\\documentclass{article}\n\\begin{document}\nHello Typesafe!\n\\end{document}".to_string()
        });

        // Initialize Dictionary
        let mut dictionary = std::collections::HashSet::new();
        let dict_path = std::path::Path::new("dictionary.txt");
        let root_dict_path = std::path::Path::new("../../dictionary.txt");

        if dict_path.exists() {
            if let Ok(content) = std::fs::read_to_string(dict_path) {
                for line in content.lines() {
                    dictionary.insert(line.trim().to_lowercase());
                }
            }
        } else if root_dict_path.exists() {
            if let Ok(content) = std::fs::read_to_string(root_dict_path) {
                for line in content.lines() {
                    dictionary.insert(line.trim().to_lowercase());
                }
            }
        } else {
             // Attempt download in a separate thread
             std::thread::spawn(|| {
                 // Using atebits/Words for a better standard english dictionary
                 if let Ok(resp) = reqwest::blocking::get("https://raw.githubusercontent.com/atebits/Words/master/Words/en.txt") {
                     if let Ok(text) = resp.text() {
                         let _ = std::fs::write("dictionary.txt", &text);
                     }
                 }
             });
        }

        // Load User Dictionary
        let mut user_dictionary = std::collections::HashSet::new();
        let user_dict_path = std::path::Path::new("user_dictionary.txt");
        if user_dict_path.exists() {
            if let Ok(content) = std::fs::read_to_string(user_dict_path) {
                for line in content.lines() {
                    user_dictionary.insert(line.trim().to_lowercase());
                }
            }
        }

        let mut app = Self {
            editor_content: default_content,
            file_path: default_file,
            current_dir,
            root_file: None,
            show_file_panel: true,
            show_preview_panel: true,
            last_window_width: 1200.0,
            outline_nodes: Vec::new(),
            labels: Vec::new(),
            bib_items: Vec::new(),
            context_menu_word: None,
            context_menu_suggestions: Vec::new(),
            context_menu_replace_range: None,
            dictionary,
            user_dictionary,
            ignored_words: std::collections::HashSet::new(),
            synonym_cache: std::collections::HashMap::new(),
            pending_synonyms: std::collections::HashSet::new(),
            synonym_rx: syn_rx,
            synonym_tx: syn_tx,
            is_dirty: false,
            last_edit_time: 0.0,
            checks_dirty: true,
            cached_syntax_errors: Vec::new(),
            cached_spell_errors: Vec::new(),
            show_command_palette: false,
            cmd_query: String::new(),
            cmd_selected_index: 0,
            syntax_set,
            theme_set,
            preview_status: "Ready to compile".to_string(),
            pdf_textures: std::collections::HashMap::new(),
            preview_size: None,
            pdf_path: None,
            pdfium: Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("."))
                .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("deps")))
                .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("../../deps")))
                .or_else(|_| Pdfium::bind_to_system_library())
                .map(Pdfium::new)
                .ok(),
            page_count: 0,
            current_page: 0,
            zoom: 1.0,
            fit_mode: PdfFitMode::Normal,
            magnifier_enabled: false,
            magnifier_zoom: 2.0,
            magnifier_size: 200.0,

            popout_zoom: 1.0,
            popout_fit_mode: PdfFitMode::Normal,
            popout_multi_page_view: false,
            compilation_log: String::new(),
            show_log: false,
            is_compiling: false,
            compile_rx: rx,
            compile_tx: tx,
            pending_autocompile: found_file,
            diagnostics: Vec::new(),
            page_sizes: std::collections::HashMap::new(),
            pending_scroll_target: None,
            pending_cursor_scroll: None,
            settings,
            completion_suggestions: Vec::new(),
            show_completions: false,
            completion_popup_pos: egui::Pos2::ZERO,
            completion_popup_rect: None,
            completion_selected_index: 0,
            latex_commands: latex_data.commands,
            latex_environments: latex_data.environments,
            readme_content: include_str!("../README.md"),
            license_content: include_str!("../LICENSE"),
            markdown_cache: egui_commonmark::CommonMarkCache::default(),

            show_search: false,
            search_query: String::new(),
            replace_query: String::new(),
            search_case_sensitive: false,
            search_whole_word: false,
            search_matches: Vec::new(),
            search_match_index: 0,
            last_save_time: 0.0,

            current_file_type: CurrentFileType::Tex,
            show_pdf_popup: false,
            pdf_popup_viewport_id: egui::ViewportId::ROOT,
            pdf_multi_page_view: false,
            pdf_search_query: String::new(),
            show_pdf_search: false,

            last_bcf_hash: String::new(),
            last_bib_hash: String::new(),

            rename_dialog_open: false,
            rename_target_path: std::path::PathBuf::new(),
            rename_new_name: String::new(),
        };

        if !app.file_path.is_empty() && !app.file_path.ends_with("untitled.tex") {
            app.update_outline();
        }

        app
    }
}

impl TypesafeApp {
    fn apply_theme(ctx: &egui::Context, theme: ThemeColors) {
        let mut visuals = Visuals::dark();

        // Main colors
        visuals.panel_fill = theme.bg_secondary;
        visuals.extreme_bg_color = theme.bg;
        visuals.faint_bg_color = theme.bg_tertiary;
        visuals.override_text_color = Some(theme.text_primary);

        // Window styling
        visuals.window_fill = theme.bg_secondary;
        visuals.window_stroke = Stroke::new(1.0, theme.border);
        visuals.window_rounding = Rounding::same(12.0);
        visuals.window_shadow = Shadow {
            offset: Vec2::new(0.0, 8.0),
            blur: 24.0,
            spread: 0.0,
            color: Color32::from_black_alpha(40),
        };

        // Widget styling (Modern/Sleek)
        visuals.widgets.noninteractive.rounding = Rounding::same(4.0);
        visuals.widgets.inactive.rounding = Rounding::same(4.0);
        visuals.widgets.hovered.rounding = Rounding::same(4.0);
        visuals.widgets.active.rounding = Rounding::same(4.0);
        visuals.widgets.open.rounding = Rounding::same(4.0);

        // Widget backgrounds
        visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
        visuals.widgets.inactive.bg_stroke = Stroke::NONE; // Cleaner look without borders on idle buttons

        // Use text color with low opacity for hover (works for both light and dark themes)
        let mut hover_color = theme.text_primary;
        hover_color = Color32::from_rgba_unmultiplied(hover_color.r(), hover_color.g(), hover_color.b(), 25);
        visuals.widgets.hovered.bg_fill = hover_color;
        visuals.widgets.hovered.bg_stroke = Stroke::NONE;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, theme.text_primary);
        visuals.widgets.hovered.expansion = 1.0; // Subtle grow effect

        visuals.widgets.active.bg_fill = theme.accent.linear_multiply(0.2);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, theme.accent);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, theme.accent);
        visuals.widgets.active.expansion = 1.0;

        visuals.widgets.open.bg_fill = theme.accent.linear_multiply(0.15);
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, theme.accent);

        // Selection
        visuals.selection.bg_fill = theme.accent.linear_multiply(0.3);
        visuals.selection.stroke = Stroke::NONE;

        // Button styling
        visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;

        // Separator
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, theme.border);

        // Interactive elements
        visuals.widgets.noninteractive.bg_fill = theme.bg;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, theme.border);

        ctx.set_visuals(visuals);

        // Modern Spacing
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(12.0);
        style.spacing.button_padding = Vec2::new(10.0, 6.0);
        style.spacing.indent = 18.0;
        style.spacing.scroll.bar_width = 10.0;
        style.spacing.scroll.handle_min_length = 24.0;
        ctx.set_style(style);
    }

    fn load_pdf_preview(&mut self, ctx: &egui::Context, pdf_path: &PathBuf) {
        self.pdf_path = Some(pdf_path.clone());
        self.preview_status = "Loading PDF...".to_string();

        let result = if let Some(pdfium) = &self.pdfium {
            pdfium
                .load_pdf_from_file(pdf_path, None)
                .map(|doc| doc.pages().len())
                .map_err(|e| e.to_string())
        } else {
            Err("PDFium not available. Please check installation.".to_string())
        };

        match result {
            Ok(pages_len) => {
                let pages: usize = pages_len.into();
                self.page_count = pages;
                self.current_page = 0;
                self.pdf_textures.clear();
                self.preview_status = format!("PDF loaded: {} pages", pages);
                // Pre-render first page(s)
                self.render_page(ctx, 0);
            }
            Err(e) => {
                self.preview_status = format!("Failed to load PDF: {}", e);
            }
        }
    }

    fn render_page(&mut self, ctx: &egui::Context, page_idx: usize) {
        if self.pdf_textures.contains_key(&page_idx) { return; }

        let Some(pdfium) = &self.pdfium else { return };
        let Some(path) = &self.pdf_path else { return };

        match render_pdf_page_to_texture(
            ctx,
            pdfium,
            path,
            page_idx,
            self.zoom,
            &mut self.preview_size,
            &mut self.page_sizes,
        ) {
            Ok(texture) => {
                self.pdf_textures.insert(page_idx, texture);
            }
            Err(e) => {
                self.preview_status = format!("Render error on page {}: {}", page_idx, e);
            }
        }
    }

    fn determine_file_type(path: &str) -> CurrentFileType {
        let path_lower = path.to_lowercase();
        if path_lower.ends_with(".pdf") {
            CurrentFileType::Pdf
        } else if path_lower.ends_with(".md") {
            CurrentFileType::Markdown
        } else if path_lower.ends_with(".tex") || path_lower.ends_with(".bib") || path_lower.ends_with(".cls") || path_lower.ends_with(".sty") {
            CurrentFileType::Tex
        } else {
            CurrentFileType::Other
        }
    }

    fn load_file(&mut self, ctx: &egui::Context, path: &str) {
        self.current_file_type = Self::determine_file_type(path);

        match self.current_file_type {
            CurrentFileType::Pdf => {
                // Load PDF file directly
                self.pdf_path = Some(std::path::PathBuf::from(path));
                self.editor_content = "".to_string();
                self.is_dirty = false;
                // PDF will be loaded on next render when pdfium is available
            }
            CurrentFileType::Markdown | CurrentFileType::Other => {
                // Load text files normally
                match std::fs::read_to_string(path) {
                    Ok(contents) => {
                        self.editor_content = contents;
                        self.is_dirty = false;
                    }
                    Err(e) => {
                        self.editor_content = format!("Error loading file: {}", e);
                    }
                }
            }
            CurrentFileType::Tex => {
                // Load LaTeX files and compile
                match std::fs::read_to_string(path) {
                    Ok(contents) => {
                        self.editor_content = contents;
                        self.is_dirty = false;
                        self.compile(ctx);
                    }
                    Err(e) => {
                        self.editor_content = format!("Error loading file: {}", e);
                    }
                }
            }
        }
    }

    fn save_file(&mut self, ctx: &egui::Context, trigger_compile: bool) {
        match std::fs::write(&self.file_path, &self.editor_content) {
            Ok(_) => {
                self.is_dirty = false;
                self.compilation_log = "File saved successfully\n".to_string();
                self.update_outline();
                if self.settings.auto_compile && trigger_compile {
                    self.compile(ctx);
                }
            }
            Err(e) => {
                self.compilation_log = format!("Error saving file: {}\n", e);
            }
        }
    }

    fn save_file_as(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new().add_filter("LaTeX", &["tex"]).save_file() {
            self.file_path = path.to_string_lossy().to_string();
            self.save_file(ctx, true);
        }
    }

    fn export_pdf(&mut self) {
        if let Some(src_path) = &self.pdf_path {
            if let Some(dst_path) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).save_file() {
                match std::fs::copy(src_path, dst_path) {
                    Ok(_) => self.compilation_log.push_str("PDF exported successfully\n"),
                    Err(e) => self.compilation_log.push_str(&format!("Error exporting PDF: {}\n", e)),
                }
            }
        } else {
            self.compilation_log.push_str("No PDF available to export. Please compile first.\n");
            self.show_log = true;
        }
    }

    fn compile(&mut self, ctx: &egui::Context) {
        if self.settings.autosave_on_compile && self.is_dirty {
            if !self.file_path.is_empty() && self.file_path != "untitled.tex" && !self.file_path.ends_with("untitled.tex") {
                 let _ = std::fs::write(&self.file_path, &self.editor_content);
                 self.is_dirty = false;
                 self.update_outline();
            }
        }
        let tx = self.compile_tx.clone();
        let ctx = ctx.clone();
        let content = self.editor_content.clone();
        let file_path = self.file_path.clone();
        let root_file = self.root_file.clone();
        let last_bcf_hash = self.last_bcf_hash.clone();
        let last_bib_hash = self.last_bib_hash.clone();

        std::thread::spawn(move || {
            let _ = tx.send(CompilationMsg::Start);
            ctx.request_repaint();

            // Determine what to compile
            let target_path = if let Some(root) = root_file {
                root
            } else if !file_path.is_empty() {
                file_path.clone()
            } else {
                "temp.tex".to_string()
            };

            // Save current file content to disk
            let save_path = if file_path.is_empty() { "temp.tex".to_string() } else { file_path.clone() };
            // Strip BOM if present to prevent "Environment document undefined" errors
            let clean_content = content.trim_start_matches('\u{feff}');
            if let Err(e) = std::fs::write(&save_path, clean_content) {
                let _ = tx.send(CompilationMsg::Error(format!("Write error: {}", e)));
                ctx.request_repaint();
                return;
            }

            // Locate tectonic binary
            let tectonic_name = if cfg!(windows) { "tectonic.exe" } else { "tectonic" };
            let mut tectonic_path = std::path::PathBuf::from(tectonic_name);

            if let Ok(current_exe) = std::env::current_exe() {
                if let Some(parent) = current_exe.parent() {
                    let candidate = parent.join(tectonic_name);
                    if candidate.exists() {
                        tectonic_path = candidate;
                    } else {
                        let candidate_deps = parent.join("deps").join(tectonic_name);
                        if candidate_deps.exists() {
                            tectonic_path = candidate_deps;
                        } else if let Some(target_dir) = parent.parent() {
                            if let Some(project_root) = target_dir.parent() {
                                let dev_candidate = project_root.join("deps").join(tectonic_name);
                                if dev_candidate.exists() {
                                    tectonic_path = dev_candidate;
                                }
                            }
                        }
                    }
                }
            }

            // Locate biber binary
            let biber_name = if cfg!(windows) { "biber.exe" } else { "biber" };
            let mut biber_path = std::path::PathBuf::from(biber_name);

            if let Ok(current_exe) = std::env::current_exe() {
                if let Some(parent) = current_exe.parent() {
                    let candidate = parent.join(biber_name);
                    if candidate.exists() {
                        biber_path = candidate;
                    } else {
                        let candidate_deps = parent.join("deps").join(biber_name);
                        if candidate_deps.exists() {
                            biber_path = candidate_deps;
                        } else if let Some(target_dir) = parent.parent() {
                            if let Some(project_root) = target_dir.parent() {
                                let dev_candidate = project_root.join("deps").join(biber_name);
                                if dev_candidate.exists() {
                                    biber_path = dev_candidate;
                                }
                            }
                        }
                    }
                }
            }

            // Determine output dir
            let parent_dir = std::path::Path::new(&target_path).parent().unwrap_or(std::path::Path::new("."));
            let file_stem = std::path::Path::new(&target_path).file_stem().unwrap_or_default().to_string_lossy();

            let run_tectonic = || {
                let mut cmd = Command::new(&tectonic_path);
                cmd.current_dir(parent_dir)
                    .arg(&target_path)
                    .arg("--synctex")
                    .arg("--keep-intermediates")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;
                    cmd.creation_flags(CREATE_NO_WINDOW);
                }
                cmd.output()
            };

            let mut output;
            let has_biblatex = content.contains("biblatex");

            // Hashing helper
            let hash_file = |path: &std::path::Path| -> String {
                if let Ok(mut file) = std::fs::File::open(path) {
                    let mut hasher = Sha256::new();
                    if let Ok(_) = std::io::copy(&mut file, &mut hasher) {
                        return format!("{:x}", hasher.finalize());
                    }
                }
                String::new()
            };

            let mut current_bcf_hash = String::new();
            let mut current_bib_hash = String::new();

            // Optimistic First Pass (Full Convergence)
            // We run Tectonic fully. Most of the time, this is all we need.
            // If we detect that Biber was needed (citations changed), we run it and then re-run Tectonic.
            let _ = tx.send(CompilationMsg::Log("Compiling document...".to_string()));
            ctx.request_repaint();
            output = run_tectonic();

            if has_biblatex {
                let bcf_path = parent_dir.join(format!("{}.bcf", file_stem));
                if output.as_ref().map(|o| o.status.success()).unwrap_or(false) && bcf_path.exists() {
                    // Calculate hashes
                    current_bcf_hash = hash_file(&bcf_path);

                    let mut bib_hasher = Sha256::new();
                    if let Ok(entries) = std::fs::read_dir(parent_dir) {
                        let mut paths: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |e| e == "bib")).collect();
                        paths.sort();
                        for p in paths {
                            if let Ok(bytes) = std::fs::read(&p) {
                                bib_hasher.update(&bytes);
                            }
                        }
                    }
                    current_bib_hash = format!("{:x}", bib_hasher.finalize());

                    let mut run_biber = true;
                    // If hashes match previous run, we assume bibliography is stable.
                    if current_bcf_hash == last_bcf_hash && current_bib_hash == last_bib_hash && !current_bcf_hash.is_empty() {
                         let _ = tx.send(CompilationMsg::Log("Citations unchanged.".to_string()));
                         ctx.request_repaint();
                         run_biber = false;
                    }

                    if run_biber {
                        let _ = tx.send(CompilationMsg::Log("Citations changed. Processing bibliography with Biber...".to_string()));
                        ctx.request_repaint();

                        let mut biber_cmd = Command::new(&biber_path);
                        biber_cmd.current_dir(parent_dir)
                            .arg(&*file_stem)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped());

                        #[cfg(windows)]
                        {
                            use std::os::windows::process::CommandExt;
                            const CREATE_NO_WINDOW: u32 = 0x08000000;
                            biber_cmd.creation_flags(CREATE_NO_WINDOW);
                        }

                        if let Ok(biber_out) = biber_cmd.output() {
                            if !biber_out.status.success() {
                                 let err = String::from_utf8_lossy(&biber_out.stderr);
                                 let _ = tx.send(CompilationMsg::Log(format!("Biber warning/error: {}", err)));
                                 ctx.request_repaint();
                            }

                            // Run Tectonic again to incorporate bibliography
                            let _ = tx.send(CompilationMsg::Log("Re-compiling document to link citations...".to_string()));
                            ctx.request_repaint();
                            output = run_tectonic();
                        } else {
                             let _ = tx.send(CompilationMsg::Log("Failed to execute Biber.".to_string()));
                             ctx.request_repaint();
                        }
                    }
                }
            }

            if let Ok(out) = &output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let mut diagnostics = Vec::new();
                // Tectonic output parsing
                let error_regex = regex::Regex::new(r"error: .+:(\d+): (.*)").unwrap();

                for line in stdout.lines().chain(stderr.lines()) {
                    if let Some(caps) = error_regex.captures(line) {
                        if let Ok(line_num) = caps[1].parse::<usize>() {
                            diagnostics.push(Diagnostic {
                                line: line_num,
                                message: caps[2].to_string(),
                                file: target_path.clone(),
                            });
                        }
                    }
                }
                if !diagnostics.is_empty() {
                     let _ = tx.send(CompilationMsg::Diagnostics(diagnostics));
                     ctx.request_repaint();
                }
            }

            match output {
                Ok(output) if output.status.success() => {
                    let stem = std::path::Path::new(&target_path).file_stem().unwrap_or_default();
                    let pdf_name = format!("{}.pdf", stem.to_string_lossy());
                    let pdf_path = parent_dir.join(&pdf_name);

                    if pdf_path.exists() {
                        let _ = tx.send(CompilationMsg::Success(pdf_path, current_bcf_hash, current_bib_hash));
                        ctx.request_repaint();
                    } else {
                        let _ = tx.send(CompilationMsg::Error(
                            "PDF file not found after compilation".to_string(),
                        ));
                        ctx.request_repaint();
                    }
                }
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let _ = tx.send(CompilationMsg::Error(format!(
                        "Compilation failed with code: {}\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
                        output.status.code().unwrap_or(-1),
                        stdout,
                        stderr
                    )));
                    ctx.request_repaint();
                }
                Err(e) => {
                    let _ = tx.send(CompilationMsg::Error(format!("Failed to run tectonic: {}", e)));
                    ctx.request_repaint();
                }
            }
        });

        self.is_compiling = true;
    }

    fn log_debug(&self, msg: &str) {
        let log_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("typesafe_debug.log")))
            .unwrap_or_else(|| std::path::PathBuf::from("typesafe_debug.log"));
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let _ = writeln!(f, "[{:.3}] {}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64(), msg);
        }
    }

    fn update_outline(&mut self) {
        self.outline_nodes.clear();
        self.labels.clear();
        self.bib_items.clear();

        let entry_file = if let Some(root) = &self.root_file {
            root.clone()
        } else {
            self.file_path.clone()
        };

        if entry_file.is_empty() { return; }

        struct FlatItem {
            label: String,
            file: String,
            line: usize,
            level: usize,
            kind: NodeKind,
        }

        let mut all_items: Vec<FlatItem> = Vec::new();
        let mut all_labels: Vec<String> = Vec::new();
        let mut visited = std::collections::HashSet::new();

        let active_path_buf = std::fs::canonicalize(std::path::Path::new(&self.file_path))
            .unwrap_or_else(|_| std::path::Path::new(&self.file_path).to_path_buf());

        struct OutlineCounters {
            chapter: usize,
            figure: usize,
            table: usize,
        }

        // Recursive processor
        fn process(
            path: std::path::PathBuf,
            display_path: String,
            visited: &mut std::collections::HashSet<std::path::PathBuf>,
            items: &mut Vec<FlatItem>,
            labels: &mut Vec<String>,
            active_path: &std::path::Path,
            active_content: &str,
            counters: &mut OutlineCounters,
        ) {
            let canon = std::fs::canonicalize(&path).unwrap_or(path.clone());
            if visited.contains(&canon) { return; }
            visited.insert(canon.clone());

            let content = if canon == active_path {
                active_content.to_string()
            } else {
                 match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(_) => return,
                 }
            };

            let parent = path.parent().unwrap_or(std::path::Path::new("."));
            let re_section = regex::Regex::new(r"\\(part|chapter|section|subsection|subsubsection)\*?\{([^}]+)\}").unwrap();
            let re_label = regex::Regex::new(r"\\label\{([^}]+)\}").unwrap();
            let re_input = regex::Regex::new(r"\\(?:input|include)\{([^}]+)\}").unwrap();

            // New regexes for Semantic Blocks
            let re_env = regex::Regex::new(r"\\begin\{(figure|table|theorem|lemma|definition)\}(?:\[.*\])?").unwrap();
            let re_caption = regex::Regex::new(r"\\caption\{([^}]+)\}").unwrap();
            let _re_cite = regex::Regex::new(r"\\cite\{([^}]+)\}").unwrap();

            for (line_idx, line) in content.lines().enumerate() {
                let clean_line = if let Some(idx) = line.find('%') { &line[..idx] } else { line };

                if let Some(cap) = re_input.captures(clean_line) {
                    if let Some(rel) = cap.get(1) {
                        let mut p_str = rel.as_str().to_string();
                        if !p_str.ends_with(".tex") { p_str.push_str(".tex"); }
                        let sub_path = parent.join(&p_str);
                        process(sub_path, p_str, visited, items, labels, active_path, active_content, counters);
                    }
                }

                if let Some(caps) = re_section.captures(clean_line) {
                    let type_str = caps.get(1).map_or("", |m| m.as_str());
                    let title = caps.get(2).map_or("", |m| m.as_str());
                    let level = match type_str {
                        "part" => 0, "chapter" => 0, "section" => 1, "subsection" => 2, "subsubsection" => 3, _ => 4,
                    };

                    if type_str == "chapter" {
                        counters.chapter += 1;
                        counters.figure = 0;
                        counters.table = 0;
                    }

                    items.push(FlatItem {
                        label: title.to_string(),
                        file: display_path.clone(),
                        line: line_idx,
                        level,
                        kind: NodeKind::Section,
                    });
                }

                // Figures/Tables/Theorems
                if let Some(caps) = re_env.captures(clean_line) {
                    let type_str = caps.get(1).map_or("block", |m| m.as_str());
                    let kind = match type_str {
                        "figure" => NodeKind::Figure,
                        "table" => NodeKind::Table,
                        "theorem" | "lemma" | "definition" => NodeKind::Theorem,
                        _ => NodeKind::Unknown,
                    };

                    // Capitalize (e.g. "Figure")
                    let mut chars = type_str.chars();
                    let capitalized_type = match chars.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                    };

                    let mut number_suffix = String::new();
                    match type_str {
                        "figure" => {
                            counters.figure += 1;
                            if counters.chapter > 0 {
                                number_suffix = format!(" {}.{}", counters.chapter, counters.figure);
                            } else {
                                number_suffix = format!(" {}", counters.figure);
                            }
                        }
                        "table" => {
                            counters.table += 1;
                            if counters.chapter > 0 {
                                number_suffix = format!(" {}.{}", counters.chapter, counters.table);
                            } else {
                                number_suffix = format!(" {}", counters.table);
                            }
                        }
                        _ => {}
                    }

                    let mut label = format!("{}{}", capitalized_type, number_suffix);
                    let mut caption_found = None;

                    // Check current line first
                    if let Some(cap_match) = re_caption.captures(clean_line) {
                        if let Some(c) = cap_match.get(1) {
                            caption_found = Some(c.as_str().to_string());
                        }
                    }

                    // If not found, scan ahead a few lines (limited scan)
                    if caption_found.is_none() {
                        let end_tag = format!("\\end{{{}}}", type_str);
                        // Skip current line, take next 50 lines to find caption
                        for ahead_line in content.lines().skip(line_idx + 1).take(50) {
                             let clean_ahead = if let Some(idx) = ahead_line.find('%') { &ahead_line[..idx] } else { ahead_line };
                             if clean_ahead.contains(&end_tag) {
                                 break;
                             }
                             if let Some(cap_match) = re_caption.captures(clean_ahead) {
                                if let Some(c) = cap_match.get(1) {
                                    caption_found = Some(c.as_str().to_string());
                                }
                                break;
                             }
                        }
                    }

                    if let Some(c) = caption_found {
                        label = format!("{}{}: {}", capitalized_type, number_suffix, c);
                    }

                    items.push(FlatItem {
                        label,
                        file: display_path.clone(),
                        line: line_idx,
                        level: 4, // Inside sections
                        kind,
                    });
                }

                // Citations - skip for outline (they clutter the panel)

                if let Some(caps) = re_label.captures(clean_line) {
                    if let Some(l) = caps.get(1) { labels.push(l.as_str().to_string()); }
                }
            }
        }

        let entry_path = if std::path::Path::new(&entry_file).is_absolute() {
             std::path::PathBuf::from(&entry_file)
        } else {
             self.current_dir.join(&entry_file)
        };

        let mut counters = OutlineCounters { chapter: 0, figure: 0, table: 0 };
        process(entry_path, entry_file, &mut visited, &mut all_items, &mut all_labels, &active_path_buf, &self.editor_content, &mut counters);

        // Build Tree
        fn insert(nodes: &mut Vec<StructureNode>, item: FlatItem) {
             if let Some(last) = nodes.last_mut() {
                 if last.level < item.level {
                     insert(&mut last.children, item);
                     return;
                 }
             }
             nodes.push(StructureNode {
                 label: item.label,
                 file_path: item.file,
                 line: item.line,
                 level: item.level,
                 kind: item.kind,
                 children: Vec::new(),
                 expanded: true,
             });
        }

        let mut roots = Vec::new();
        for item in all_items {
            insert(&mut roots, item);
        }
        self.outline_nodes = roots;
        self.labels = all_labels;

        // Scan bibliography using biblatex parser
        if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                 if entry.path().extension().map_or(false, |e| e == "bib") {
                    if let Ok(c) = std::fs::read_to_string(entry.path()) {
                        if let Ok(bibliography) = biblatex::Bibliography::parse(&c) {
                            for entry in bibliography {
                                self.bib_items.push(entry.key);
                            }
                        }
                    }
                 }
            }
        }
    }

    fn check_syntax(&self, text: &str) -> Vec<std::ops::Range<usize>> {
        let mut errors = Vec::new();
        let mut stack = Vec::new();
        let mut escaped = false;

        // Bracket balance
        for (i, c) in text.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            if c == '\\' {
                escaped = true;
                continue;
            }

            match c {
                '{' | '[' | '(' => stack.push((i, c)),
                '}' => {
                    if let Some((_, last)) = stack.pop() {
                        if last != '{' {
                            errors.push(i..i + 1);
                        }
                    } else {
                        errors.push(i..i + 1);
                    }
                }
                ']' => {
                    if let Some((_, last)) = stack.pop() {
                        if last != '[' {
                            errors.push(i..i + 1);
                        }
                    } else {
                        errors.push(i..i + 1);
                    }
                }
                ')' => {
                    if let Some((_, last)) = stack.pop() {
                        if last != '(' {
                            errors.push(i..i + 1);
                        }
                    } else {
                        errors.push(i..i + 1);
                    }
                }
                _ => {}
            }
        }

        // Unclosed delimiters
        for (i, _) in stack.drain(..) {
            errors.push(i..i + 1);
        }

        // \begin / \end validation
        let allowed_envs = [
            "document",
            "itemize",
            "enumerate",
            "description",
            "figure",
            "figure*",
            "table",
            "table*",
            "equation",
            "align",
            "align*",
            "center",
            "flushleft",
            "flushright",
            "tabular",
            "theorem",
            "lemma",
            "proof",
        ];

        let mut env_stack: Vec<(String, usize, usize)> = Vec::new();
        let mut pos = 0;
        while pos < text.len() {
            let next_begin = text[pos..].find("\\begin{");
            let next_end = text[pos..].find("\\end{");
            let (kind, idx) = match (next_begin, next_end) {
                (Some(b), Some(e)) => {
                    if b <= e {
                        ("begin", b)
                    } else {
                        ("end", e)
                    }
                }
                (Some(b), None) => ("begin", b),
                (None, Some(e)) => ("end", e),
                (None, None) => break,
            };

            let start = pos + idx;
            let name_start = start
                + if kind == "begin" {
                    "\\begin{".len()
                } else {
                    "\\end{".len()
                };

            if let Some(close_rel) = text[name_start..].find('}') {
                let name_end = name_start + close_rel;
                let name = &text[name_start..name_end];

                if !allowed_envs.contains(&name) {
                    errors.push(name_start..name_end);
                }

                if kind == "begin" {
                    env_stack.push((name.to_string(), name_start, name_end));
                } else {
                    match env_stack.pop() {
                        Some((open_name, open_start, open_end)) => {
                            if open_name != name {
                                errors.push(name_start..name_end);
                                errors.push(open_start..open_end);
                            }
                        }
                        None => errors.push(name_start..name_end),
                    }
                }

                pos = name_end + 1;
            } else {
                break;
            }
        }

        for (_, open_start, open_end) in env_stack {
            errors.push(open_start..open_end);
        }

        errors
    }

    fn check_spelling(&self, text: &str) -> Vec<std::ops::Range<usize>> {
        let mut errors = Vec::new();
        if self.dictionary.is_empty() {
             return errors;
        }

        for (start, word) in text.unicode_word_indices() {
             // Filter out non-alphabetic words
             if !word.chars().any(|c| c.is_alphabetic()) {
                 continue;
             }

             let end = start + word.len();
             let lower = word.to_lowercase();

             // Ignore LaTeX commands
             if start > 0 && text[..start].ends_with('\\') { continue; }

             // Ignore words in specific commands like \cite{...}, \usepackage{...}
             let preceding = &text[0..start];
             if let Some(brace_idx) = preceding.rfind('{') {
                 let before_brace = preceding[..brace_idx].trim_end();
                 if before_brace.ends_with("\\cite") ||
                    before_brace.ends_with("\\ref") ||
                    before_brace.ends_with("\\label") ||
                    before_brace.ends_with("\\usepackage") ||
                    before_brace.ends_with("\\documentclass") ||
                    before_brace.ends_with("\\input") ||
                    before_brace.ends_with("\\include") ||
                    before_brace.ends_with("\\bibliographystyle") ||
                    before_brace.ends_with("\\bibliography") {
                     // Verify we haven't closed the brace yet
                     let after_brace = &preceding[brace_idx+1..];
                     if !after_brace.contains('}') {
                         continue;
                     }
                 }
             }

             if !self.dictionary.contains(&lower)
                && !self.user_dictionary.contains(&lower)
                && !self.ignored_words.contains(&lower)
             {
                  errors.push(start..end);
             }
        }

        errors
    }

    fn insert_command(&mut self, ctx: &egui::Context, command: &str) {
        self.insert_snippet(ctx, command);
    }

    fn insert_snippet(&mut self, ctx: &egui::Context, snippet: &str) {
        // Simple snippet parser: replaces $1, $2, etc with empty string and places cursor at $1
        let mut final_text = snippet.to_string();
        let mut selection_range = None;

        // Find $1
        if let Some(idx) = final_text.find("$1") {
            final_text.replace_range(idx..idx+2, "");
            selection_range = Some(idx);
        }

        if let Some(mut state) = egui::TextEdit::load_state(ctx, egui::Id::new("main_editor")) {
            if let Some(range) = state.cursor.char_range() {
                let cursor = range.primary.index;
                self.editor_content.insert_str(cursor, &final_text);

                // Calculate new cursor position
                let new_cursor_idx = if let Some(rel_idx) = selection_range {
                    cursor + rel_idx
                } else {
                    cursor + final_text.len()
                };

                state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(new_cursor_idx))));
                state.store(ctx, egui::Id::new("main_editor"));
            } else {
                self.editor_content.push_str(&final_text);
            }
        } else {
            self.editor_content.push_str(&final_text);
        }
        self.is_dirty = true;
        ctx.request_repaint();
    }

    fn apply_completion(&mut self, ctx: &egui::Context, text: &mut String, completion: &str) {
        if let Some(mut state) = egui::TextEdit::load_state(ctx, egui::Id::new("main_editor")) {
            if let Some(range) = state.cursor.char_range() {
                let cursor_idx = range.primary.index;
                let mut word_start = cursor_idx;
                let chars: Vec<char> = text.chars().collect();

                while word_start > 0 {
                    let prev_char = chars[word_start - 1];
                    let is_word_char = prev_char.is_alphanumeric() || prev_char == '\\'
                        || prev_char == '[' || prev_char == '{' || prev_char == '_' || prev_char == '*';
                    if !is_word_char { break; }
                    word_start -= 1;
                }

                // Detect indentation
                let line_start = text[..word_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let base_indent = text[line_start..word_start].chars().take_while(|c| c.is_whitespace()).collect::<String>();

                // Normalize tabs to our indent unit and re-indent following lines
                let normalized = completion.replace("\t", INDENT_UNIT);
                let indented_completion = normalized.lines().enumerate().map(|(i, line)| {
                    if i > 0 && !line.is_empty() {
                        format!("{}{}", base_indent, line)
                    } else {
                        line.to_string()
                    }
                }).collect::<Vec<_>>().join("\n");

                // Avoid double closing braces/brackets when one already exists at the cursor
                let mut final_completion = indented_completion.clone();
                if let Some(next) = text[cursor_idx..].chars().next() {
                    if (final_completion.ends_with('}') && next == '}') || (final_completion.ends_with(']') && next == ']') {
                        final_completion.pop();
                    }
                }

                text.drain(word_start..cursor_idx);
                text.insert_str(word_start, &final_completion);

                let mut new_cursor_idx = word_start + final_completion.len();
                if let Some(pos) = final_completion.find("{}") {
                    new_cursor_idx = word_start + pos + 1;
                } else if let Some(pos) = final_completion.find("[]") {
                    new_cursor_idx = word_start + pos + 1;
                } else if let Some(pos) = final_completion.find("}{") {
                    new_cursor_idx = word_start + pos + 1;
                }

                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(new_cursor_idx))));
                state.store(ctx, egui::Id::new("main_editor"));
            }
        }
        self.is_dirty = true;
        self.checks_dirty = true;
        self.last_edit_time = 0.0; // Force immediate check
        ctx.request_repaint();
    }

    fn sync_forward_search(&mut self, line_num: usize) {
        let mut found = false;
        if let Some(pdf_path) = &self.pdf_path {
            let stem = pdf_path.file_stem().unwrap_or_default();
            let parent = pdf_path.parent().unwrap_or(std::path::Path::new("."));
            let synctex_path = parent.join(format!("{}.synctex.gz", stem.to_string_lossy()));

            if synctex_path.exists() {
                 if let Ok(file) = std::fs::File::open(&synctex_path) {
                    let mut decoder = GzDecoder::new(file);
                    let mut content = String::new();
                    if decoder.read_to_string(&mut content).is_ok() {
                        let file_name = std::path::Path::new(&self.file_path).file_name().unwrap_or_default().to_string_lossy();

                        // 1. Find File ID
                        let mut file_id = None;
                        for line in content.lines() {
                            if line.starts_with("Input:") {
                                let parts: Vec<&str> = line.splitn(3, ':').collect();
                                if parts.len() == 3 {
                                    if parts[2].contains(&*file_name) {
                                        file_id = parts[1].parse::<usize>().ok();
                                        break;
                                    }
                                }
                            }
                        }

                        if let Some(fid) = file_id {
                            let mut current_page = 1;
                            let prefix = format!("{},{}:", fid, line_num);

                            for line in content.lines() {
                                if line.starts_with('{') {
                                    if let Ok(p) = line[1..].parse::<usize>() { current_page = p; }
                                } else if line.len() > 1 && line[1..].starts_with(&prefix) {
                                     let parts: Vec<&str> = line.split(':').collect();
                                     if parts.len() >= 2 {
                                         let coords: Vec<&str> = parts[1].split(',').collect();
                                         if coords.len() >= 2 {
                                             if let Ok(v_sp) = coords[1].parse::<f32>() {
                                                 let v_pt = v_sp / 65536.0;
                                                 let page_idx = current_page.saturating_sub(1);

                                                 let mut rel_y = 0.5;
                                                 if let Some((_, h)) = self.page_sizes.get(&page_idx) {
                                                     if *h > 0.0 { rel_y = v_pt / *h; }
                                                 }
                                                 self.current_page = page_idx;
                                                 self.pending_scroll_target = Some((page_idx, rel_y));
                                                 found = true;
                                                 break;
                                             }
                                         }
                                     }
                                }
                            }
                        }
                    }
                 }
            }
        }

        if !found {
            // Naive fallback
            let total_lines = self.editor_content.lines().count().max(1);
            let rel = line_num as f32 / total_lines as f32;
            let target_page = ((self.page_count as f32 * rel) as usize).min(self.page_count.saturating_sub(1));
            self.current_page = target_page;
            self.pending_scroll_target = Some((target_page, 0.5));
        }
    }

    fn syntax_highlighting(&self, theme: &ThemeColors, text: &str) -> egui::text::LayoutJob {
        let syntax = self
            .syntax_set
            .find_syntax_by_extension("tex")
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        // Select syntect theme based on our app theme brightness
        let bg_brightness = (theme.bg.r() as u32 + theme.bg.g() as u32 + theme.bg.b() as u32) / 3;

        let theme_name = if bg_brightness < 128 {
            "base16-ocean.dark"
        } else {
            "InspiredGitHub"
        };

        let syntect_theme = &self.theme_set.themes[theme_name];
        let mut highlighter = HighlightLines::new(syntax, syntect_theme);

        // Use cached errors to prevent flashing while typing
        let errors = &self.cached_syntax_errors;
        let spell_errors = &self.cached_spell_errors;

        // Run search check
        let mut search_matches_bytes = Vec::new();
        if self.show_search && !self.search_query.is_empty() {
             let query = &self.search_query;
             let raw_matches = if self.search_case_sensitive {
                 text.match_indices(query).map(|(i, s)| (i, i + s.len())).collect::<Vec<_>>()
             } else {
                 text.to_lowercase().match_indices(&query.to_lowercase()).map(|(i, s)| (i, i + s.len())).collect::<Vec<_>>()
             };

             for (s, e) in raw_matches {
                 let mut valid = true;
                 if self.search_whole_word {
                     let is_start_ok = s == 0 || !text[..s].chars().last().unwrap_or(' ').is_alphanumeric();
                     let is_end_ok = e == text.len() || !text[e..].chars().next().unwrap_or(' ').is_alphanumeric();
                     if !is_start_ok || !is_end_ok { valid = false; }
                 }
                 if valid { search_matches_bytes.push((s, e)); }
             }
        }

        let mut job = egui::text::LayoutJob::default();
        let mut current_byte_idx = 0;
        let mut line_num = 1;

        for line in LinesWithEndings::from(text) {
            let ranges: Vec<(Style, &str)> = highlighter
                .highlight_line(line, &self.syntax_set)
                .unwrap_or_default();

            let has_compiler_error = self.diagnostics.iter().any(|d| d.line == line_num);

            for (style, range_text) in ranges {
                let range_len = range_text.len();
                let range_start = current_byte_idx;
                let range_end = range_start + range_len;

                let fg = style.foreground;
                let text_color = Color32::from_rgb(fg.r, fg.g, fg.b);
                let font_id = TextStyle::Monospace.resolve(&egui::Style::default());

                // Split tokens to precisely highlight errors and search matches
                let mut split_points = vec![0, range_len];

                for e in errors {
                    if e.start > range_start && e.start < range_end {
                        split_points.push(e.start - range_start);
                    }
                    if e.end > range_start && e.end < range_end {
                        split_points.push(e.end - range_start);
                    }
                }

                for e in spell_errors {
                    if e.start > range_start && e.start < range_end {
                        split_points.push(e.start - range_start);
                    }
                    if e.end > range_start && e.end < range_end {
                        split_points.push(e.end - range_start);
                    }
                }

                for (s, e) in &search_matches_bytes {
                    if *s > range_start && *s < range_end {
                        split_points.push(*s - range_start);
                    }
                    if *e > range_start && *e < range_end {
                        split_points.push(*e - range_start);
                    }
                }

                split_points.sort_unstable();
                split_points.dedup();

                for i in 0..split_points.len() - 1 {
                    let local_start = split_points[i];
                    let local_end = split_points[i + 1];
                    let abs_start = range_start + local_start;
                    let abs_end = range_start + local_end;

                    let sub_text = &range_text[local_start..local_end];

                    let has_error = errors.iter().any(|e| e.start <= abs_start && e.end >= abs_end);
                    let is_spell_error = spell_errors.iter().any(|e| e.start <= abs_start && e.end >= abs_end);
                    let is_search_match =
                        search_matches_bytes.iter().any(|(s, e)| *s <= abs_start && *e >= abs_end);

                    let stroke = if has_error {
                        Stroke::new(2.0, theme.error)
                    } else if has_compiler_error {
                        Stroke::new(1.5, theme.error)
                    } else if is_spell_error {
                        Stroke::new(1.0, theme.warning)
                    } else {
                        Stroke::NONE
                    };

                    let background = if is_search_match {
                        theme.warning.linear_multiply(0.3)
                    } else {
                        Color32::TRANSPARENT
                    };

                    job.append(
                        sub_text,
                        0.0,
                        egui::TextFormat {
                            font_id: font_id.clone(),
                            color: text_color,
                            underline: stroke,
                            background,
                            ..Default::default()
                        },
                    );
                }

                current_byte_idx += range_len;
            }
            line_num += 1;
        }

        job
    }
}

impl eframe::App for TypesafeApp {
    #[allow(deprecated)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle pending compilation
        if self.pending_autocompile {
            self.pending_autocompile = false;
            self.compile(ctx);
        }

        // Load PDF if needed
        if self.current_file_type == CurrentFileType::Pdf && self.pdf_path.is_some() && self.pdfium.is_some() {
            if self.page_count == 0 {
                if let Some(pdf_path) = &self.pdf_path.clone() {
                    self.load_pdf_preview(ctx, pdf_path);
                }
            }
        }

        let now = ctx.input(|i| i.time);
        if self.settings.autosave_timer && self.is_dirty {
            if now - self.last_save_time > 30.0 {
                 if !self.file_path.is_empty() && !self.file_path.ends_with("untitled.tex") {
                     self.save_file(ctx, false);
                     self.compilation_log = "Autosaved.\n".to_string();
                 }
                 self.last_save_time = now;
            }
        }



        // Apply theme
        let mut theme = ThemeColors::from_preset(self.settings.theme);
        theme.accent = self.settings.accent.color();
        Self::apply_theme(ctx, theme);

        // Responsive Layout Logic
        let window_width = ctx.screen_rect().width();
        if (window_width - self.last_window_width).abs() > 2.0 {
             // If shrinking
             if window_width < self.last_window_width {
                 let files_allowance = if self.show_file_panel { 250.0 } else { 32.0 };
                 let preview_allowance = if self.show_preview_panel { 350.0 } else { 32.0 };
                 let min_editor_width = 720.0;

                 let remaining = window_width - files_allowance - preview_allowance;

                 // Force PDF to re-fit width when window shrinks to prevent cutoff
                 if self.show_preview_panel {
                     self.fit_mode = PdfFitMode::FitWidth;
                     self.pdf_textures.clear();
                 }

                 if remaining < min_editor_width {
                     if self.show_file_panel {
                         self.show_file_panel = false;
                         // If closing files isn't enough, also close preview
                         let new_remaining = window_width - 32.0 - preview_allowance;
                         if new_remaining < min_editor_width && self.show_preview_panel {
                             self.show_preview_panel = false;
                         }
                     } else if self.show_preview_panel {
                         self.show_preview_panel = false;
                     }
                 }
             }
             self.last_window_width = window_width;
        }

        // Poll synonym results
        while let Ok((word, synonyms)) = self.synonym_rx.try_recv() {
            self.synonym_cache.insert(word, synonyms);
        }

        // Poll compilation messages
        while let Ok(msg) = self.compile_rx.try_recv() {
            match msg {
                CompilationMsg::Start => {
                    self.is_compiling = true;
                    self.compilation_log = "Starting compilation...\n".to_string();
                    self.diagnostics.clear();
                }
                CompilationMsg::Log(line) => {
                    self.compilation_log.push_str(&line);
                    self.compilation_log.push('\n');
                }
                CompilationMsg::Diagnostics(diags) => {
                    self.diagnostics = diags;
                }
                CompilationMsg::Success(pdf_path, bcf_hash, bib_hash) => {
                    self.is_compiling = false;
                    self.preview_status =
                        format!("✓ Compilation success\nOutput: {}", pdf_path.display());
                    self.last_bcf_hash = bcf_hash;
                    self.last_bib_hash = bib_hash;
                    self.compilation_log.push_str("\nDone!");
                    self.load_pdf_preview(ctx, &pdf_path);
                }
                CompilationMsg::Error(err) => {
                    self.is_compiling = false;
                    self.compilation_log.push_str(&format!("\nERROR: {}", err));
                    self.show_log = true;
                }
            }
        }



        // Handle keyboard shortcuts
        if ctx.input(|i| {
            i.key_pressed(egui::Key::S) && (i.modifiers.ctrl || i.modifiers.command)
        }) {
            self.save_file(ctx, true);
        }

        // Block Commenting (Ctrl+/)
        if ctx.input(|i| i.key_pressed(egui::Key::Slash) && (i.modifiers.ctrl || i.modifiers.command)) {
            if let Some(state) = egui::TextEdit::load_state(ctx, egui::Id::new("main_editor")) {
                if let Some(range) = state.cursor.char_range() {
                    let start = range.primary.index.min(range.secondary.index);
                    let end = range.primary.index.max(range.secondary.index);

                    let mut current_pos = 0;
                    let mut start_line_idx = 0;
                    let mut end_line_idx = 0;

                    let mut lines: Vec<String> = self.editor_content.lines().map(|s| s.to_string()).collect();
                    // Handle trailing newline case which lines() swallows
                    if self.editor_content.ends_with('\n') {
                        lines.push(String::new());
                    }

                    for (i, line) in lines.iter().enumerate() {
                        let line_len = line.chars().count() + 1;

                        if current_pos <= start {
                            start_line_idx = i;
                        }
                        if current_pos < end {
                            end_line_idx = i;
                        }
                        current_pos += line_len;
                    }

                    if start_line_idx < lines.len() {
                        let target_end = end_line_idx.min(lines.len() - 1);
                        let target_lines = &lines[start_line_idx..=target_end];
                        let all_commented = target_lines.iter().all(|l| l.trim_start().starts_with('%'));

                        for i in start_line_idx..=target_end {
                            if all_commented {
                                if let Some(idx) = lines[i].find('%') {
                                    lines[i].remove(idx);
                                }
                            } else {
                                lines[i].insert(0, '%');
                            }
                        }

                        self.editor_content = lines.join("\n");
                        self.is_dirty = true;
                    }
                }
            }
        }
        if ctx.input(|i| {
            i.key_pressed(egui::Key::B) && (i.modifiers.ctrl || i.modifiers.command)
        }) {
            if self.current_file_type == CurrentFileType::Tex && !self.is_compiling {
                self.compile(ctx);
            }
        }

        // Search Toggle
        if ctx.input(|i| i.key_pressed(egui::Key::F) && (i.modifiers.ctrl || i.modifiers.command)) {
            if self.current_file_type == CurrentFileType::Pdf {
                self.show_pdf_search = !self.show_pdf_search;
            } else {
                self.show_search = !self.show_search;
                if self.show_search {
                    // Focus handled by UI render
                } else {
                    self.search_matches.clear();
                }
            }
        }

        // Command Palette Toggle
        if ctx.input(|i| i.key_pressed(egui::Key::P) && (i.modifiers.ctrl && i.modifiers.shift)) {
            self.show_command_palette = !self.show_command_palette;
            self.cmd_query.clear();
        }

        // PDF Pop-out (Ctrl+Alt+P)
        if ctx.input(|i| i.key_pressed(egui::Key::P) && i.modifiers.ctrl && i.modifiers.alt) {
            if self.current_file_type == CurrentFileType::Pdf {
                self.show_pdf_popup = !self.show_pdf_popup;
            }
        }

        // ====== TOP PANEL ======
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                // File Menu
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        let mut proceed = true;
                        if self.is_dirty {
                            if self.settings.autosave_on_change {
                                self.save_file(ctx, false);
                            } else {
                                let confirmed = rfd::MessageDialog::new()
                                    .set_title("Unsaved Changes")
                                    .set_description("Do you want to save changes to the current file?")
                                    .set_buttons(rfd::MessageButtons::YesNoCancel)
                                    .show();
                                match confirmed {
                                    rfd::MessageDialogResult::Yes => self.save_file(ctx, true),
                                    rfd::MessageDialogResult::No => {},
                                    rfd::MessageDialogResult::Cancel => proceed = false,
                                    _ => {},
                                }
                            }
                        }
                        if proceed {
                            self.editor_content = "\\documentclass{article}\n\\begin{document}\n\n\\end{document}".to_string();
                            self.file_path = self.current_dir.join("untitled.tex").to_string_lossy().to_string();
                            self.settings.last_file = None;
                            self.settings.save();
                            ui.close_menu();
                        }
                    }
                    ui.separator();
                    if ui.button("Open File...").clicked() {
                        let mut proceed = true;
                        if self.is_dirty {
                             if self.settings.autosave_on_change {
                                 self.save_file(ctx, false);
                             } else {
                                 let confirmed = rfd::MessageDialog::new()
                                     .set_title("Unsaved Changes")
                                     .set_description("Do you want to save changes to the current file?")
                                     .set_buttons(rfd::MessageButtons::YesNoCancel)
                                     .show();
                                 match confirmed {
                                     rfd::MessageDialogResult::Yes => self.save_file(ctx, true),
                                     rfd::MessageDialogResult::No => {},
                                     rfd::MessageDialogResult::Cancel => proceed = false,
                                     _ => {},
                                 }
                             }
                        }
                        if proceed {
                            if let Some(path) = rfd::FileDialog::new().add_filter("LaTeX", &["tex"]).pick_file() {
                                self.file_path = path.to_string_lossy().to_string();
                                let path_str = self.file_path.clone();
                                self.load_file(ctx, &path_str);
                                self.settings.last_file = Some(self.file_path.clone());
                                self.settings.save();
                                if let Some(parent) = path.parent() {
                                    self.current_dir = parent.to_path_buf();
                                }
                                ui.close_menu();
                            }
                        }
                    }
                    if ui.button("Open Folder...").clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            self.current_dir = folder;
                            ui.close_menu();
                        }
                    }
                    ui.separator();
                    if ui.add(egui::Button::new("Save").shortcut_text("Ctrl+S")).clicked() {
                        self.save_file(ctx, true);
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        self.save_file_as(ctx);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Export PDF...").clicked() {
                        self.export_pdf();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Settings").clicked() {
                        self.settings.show_settings_window = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Reload LaTeX Data").clicked() {
                        let mut success = false;
                        let mut count = 0;
                        let mut loaded_source = String::new();

                        // Try exe dir first
                        if let Ok(exe_path) = std::env::current_exe() {
                            if let Some(exe_dir) = exe_path.parent() {
                                let candidate = exe_dir.join("latex_data.json");
                                if candidate.exists() {
                                    if let Ok(content) = std::fs::read_to_string(&candidate) {
                                        if let Ok(data) = serde_json::from_str::<LatexData>(&content) {
                                            self.latex_commands = data.commands;
                                            self.latex_environments = data.environments;
                                            count = self.latex_commands.len() + self.latex_environments.len();
                                            loaded_source = candidate.to_string_lossy().to_string();
                                            success = true;
                                        }
                                    }
                                }
                            }
                        }

                        // Try CWD if not found
                        if !success {
                            if let Ok(content) = std::fs::read_to_string("latex_data.json") {
                                if let Ok(data) = serde_json::from_str::<LatexData>(&content) {
                                    self.latex_commands = data.commands;
                                    self.latex_environments = data.environments;
                                    count = self.latex_commands.len() + self.latex_environments.len();
                                    loaded_source = "CWD/latex_data.json".to_string();
                                    success = true;
                                }
                            }
                        }

                        let msg = if success {
                            format!("Successfully loaded {} items from {}", count, loaded_source)
                        } else {
                            "Failed to load latex_data.json. Ensure it exists next to the exe or in current working directory.".to_string()
                        };

                        rfd::MessageDialog::new()
                            .set_title("Reload Data")
                            .set_description(&msg)
                            .show();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // Edit Menu
                ui.menu_button("Edit", |ui| {
                     if ui.add(egui::Button::new("Undo").shortcut_text("Ctrl+Z")).clicked() {
                         // Handled by TextEdit
                         ui.close_menu();
                     }
                     if ui.add(egui::Button::new("Redo").shortcut_text("Ctrl+Y")).clicked() {
                         // Handled by TextEdit
                         ui.close_menu();
                     }
                     ui.separator();
                     if ui.add(egui::Button::new("Cut").shortcut_text("Ctrl+X")).clicked() {
                        // Handled by TextEdit
                        ui.close_menu();
                     }
                     if ui.add(egui::Button::new("Copy").shortcut_text("Ctrl+C")).clicked() {
                        // Handled by TextEdit
                        ui.close_menu();
                     }
                     if ui.add(egui::Button::new("Paste").shortcut_text("Ctrl+V")).clicked() {
                        // Handled by TextEdit
                        ui.close_menu();
                     }
                     ui.separator();
                     if ui.add(egui::Button::new("Find").shortcut_text("Ctrl+F")).clicked() {
                         self.show_search = !self.show_search;
                         ui.close_menu();
                     }
                });

                // View Menu
                ui.menu_button("View", |ui| {
                    if ui.checkbox(&mut self.show_file_panel, "File Explorer").clicked() {
                        ui.close_menu();
                    }
                    if ui.checkbox(&mut self.show_preview_panel, "PDF Preview").clicked() {
                        ui.close_menu();
                    }
                    if ui.checkbox(&mut self.show_log, "Compilation Log").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.add(egui::Button::new("Command Palette").shortcut_text("Ctrl+Shift+P")).clicked() {
                        self.show_command_palette = !self.show_command_palette;
                         ui.close_menu();
                    }
                    if self.pdf_path.is_some() {
                        ui.separator();
                        if ui.add(egui::Button::new("Pop-out PDF Viewer").shortcut_text("Ctrl+Alt+P")).clicked() {
                            self.show_pdf_popup = true;
                            self.pdf_textures.clear();
                            ui.close_menu();
                        }
                    }
                });

                // Tools Menu
                ui.menu_button("Tools", |ui| {
                     ui.menu_button("LaTeX", |ui| {
                        ui.menu_button("Structure", |ui| {
                            if ui.button("Part").clicked() { self.insert_command(ctx, "\\part{}"); ui.close_menu(); }
                            if ui.button("Chapter").clicked() { self.insert_command(ctx, "\\chapter{}"); ui.close_menu(); }
                            if ui.button("Section").clicked() { self.insert_command(ctx, "\\section{}"); ui.close_menu(); }
                            if ui.button("Subsection").clicked() { self.insert_command(ctx, "\\subsection{}"); ui.close_menu(); }
                            if ui.button("Subsubsection").clicked() { self.insert_command(ctx, "\\subsubsection{}"); ui.close_menu(); }
                            if ui.button("Paragraph").clicked() { self.insert_command(ctx, "\\paragraph{}"); ui.close_menu(); }
                        });

                        ui.menu_button("Formatting", |ui| {
                            if ui.button("Bold").clicked() { self.insert_command(ctx, "\\textbf{}"); ui.close_menu(); }
                            if ui.button("Italic").clicked() { self.insert_command(ctx, "\\textit{}"); ui.close_menu(); }
                            if ui.button("Underline").clicked() { self.insert_command(ctx, "\\underline{}"); ui.close_menu(); }
                            if ui.button("Emphasis").clicked() { self.insert_command(ctx, "\\emph{}"); ui.close_menu(); }
                        });

                        ui.menu_button("Math", |ui| {
                            if ui.button("Inline Math ($)").clicked() { self.insert_command(ctx, "$$"); ui.close_menu(); }
                            if ui.button("Display Math (\\[)").clicked() { self.insert_command(ctx, "\\[\n\t\n\\]"); ui.close_menu(); }
                            if ui.button("Equation").clicked() { self.insert_command(ctx, "\\begin{equation}\n\t\n\\end{equation}"); ui.close_menu(); }
                            if ui.button("Fraction").clicked() { self.insert_command(ctx, "\\frac{}{}"); ui.close_menu(); }
                        });

                        ui.menu_button("Environments", |ui| {
                            if ui.button("Itemize").clicked() { self.insert_command(ctx, "\\begin{itemize}\n\t\\item \n\\end{itemize}"); ui.close_menu(); }
                            if ui.button("Enumerate").clicked() { self.insert_command(ctx, "\\begin{enumerate}\n\t\\item \n\\end{enumerate}"); ui.close_menu(); }
                            if ui.button("Figure").clicked() { self.insert_command(ctx, "\\begin{figure}[h]\n\t\\centering\n\t\\caption{}\n\\end{figure}"); ui.close_menu(); }
                            if ui.button("Table").clicked() { self.insert_command(ctx, "\\begin{table}[h]\n\t\\centering\n\t\\begin{tabular}{c c}\n\t\tA & B \\\\\n\t\\end{tabular}\n\t\\caption{}\n\\end{table}"); ui.close_menu(); }
                        });
                     });
                     ui.separator();
                     if ui.add(egui::Button::new("Build PDF").shortcut_text("Ctrl+B")).clicked() {
                         self.compile(ctx);
                         ui.close_menu();
                     }
                });

                 // Help Menu
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.settings.active_tab = SettingsTab::About;
                        self.settings.show_settings_window = true;
                        ui.close_menu();
                    }
                    if ui.button("License").clicked() {
                        self.settings.active_tab = SettingsTab::License;
                        self.settings.show_settings_window = true;
                        ui.close_menu();
                    }
                });



                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.heading(
                        egui::RichText::new("Typesafe")
                            .color(theme.text_primary)
                            .font(FontId::proportional(22.0)),
                    );

                    if self.is_dirty {
                        ui.label(
                            egui::RichText::new("● Unsaved")
                                .color(theme.warning),
                        );
                    }
                    if self.is_compiling {
                        ui.label(
                            egui::RichText::new("◐ Compiling...")
                                .color(theme.accent),
                        );
                    }
                });
            });
            ui.add_space(4.0);
        });

        let mut show_settings = self.settings.show_settings_window;
        if show_settings {
            egui::Window::new("Settings")
                .open(&mut show_settings)
                .pivot(egui::Align2::CENTER_CENTER)
                .default_pos(ctx.screen_rect().center())
                .fixed_size([500.0, 700.0])
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.settings.active_tab, SettingsTab::Appearance, "Appearance");
                        ui.selectable_value(&mut self.settings.active_tab, SettingsTab::Editor, "Editor");
                        ui.selectable_value(&mut self.settings.active_tab, SettingsTab::Permissions, "Permissions");
                        ui.selectable_value(&mut self.settings.active_tab, SettingsTab::APIs, "APIs");
                        ui.selectable_value(&mut self.settings.active_tab, SettingsTab::About, "About");
                        ui.selectable_value(&mut self.settings.active_tab, SettingsTab::License, "License");
                    });
                    ui.separator();

                    match self.settings.active_tab {
                        SettingsTab::Appearance => {
                            ui.label(
                                egui::RichText::new("🎨 Theme")
                                    .strong()
                                    .color(theme.text_primary),
                            );
                            egui::ComboBox::from_id_source("theme_selector")
                                .selected_text(self.settings.theme.name())
                                .width(200.0)
                                .show_ui(ui, |ui| {
                                    for &preset in ThemePreset::all() {
                                        if ui.selectable_value(&mut self.settings.theme, preset, preset.name()).changed() {
                                            self.settings.save();
                                        }
                                    }
                                });

                            ui.add_space(8.0);

                            ui.label(
                                egui::RichText::new("✨ Accent Color")
                                    .strong()
                                    .color(theme.text_primary),
                            );
                            egui::ComboBox::from_id_source("accent_selector")
                                .selected_text(self.settings.accent.name())
                                .width(200.0)
                                .show_ui(ui, |ui| {
                                    for &accent in AccentColor::all() {
                                        if ui.selectable_value(&mut self.settings.accent, accent, accent.name()).changed() {
                                            self.settings.save();
                                        }
                                    }
                                });
                        },
                        SettingsTab::Editor => {
                            ui.heading("Autosave");
                            ui.add_space(4.0);
                            if ui.checkbox(&mut self.settings.autosave_timer, "Autosave periodically (30s)").changed() {
                                self.settings.save();
                            }
                            if ui.checkbox(&mut self.settings.autosave_on_compile, "Autosave on Compile").changed() {
                                self.settings.save();
                            }
                            if ui.checkbox(&mut self.settings.autosave_on_change, "Autosave on File Switch").changed() {
                                self.settings.save();
                            }
                        },
                        SettingsTab::Permissions => {
                            ui.label("System Integration");
                            ui.add_space(8.0);

                            #[cfg(windows)]
                            if ui.button("Set as Default for .tex files").clicked() {
                                if let Ok(exe_path) = std::env::current_exe() {
                                    let exe = exe_path.to_string_lossy();
                                    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
                                    let path = std::path::Path::new("Software\\Classes\\.tex");
                                    if let Ok((key, _)) = hkcu.create_subkey(&path) {
                                        let _ = key.set_value("", &"Typesafe.Document");
                                    }

                                    let path = std::path::Path::new("Software\\Classes\\Typesafe.Document");
                                    if let Ok((key, _)) = hkcu.create_subkey(&path) {
                                        let _ = key.set_value("", &"LaTeX Document");
                                    }

                                    let path = std::path::Path::new("Software\\Classes\\Typesafe.Document\\DefaultIcon");
                                    if let Ok((key, _)) = hkcu.create_subkey(&path) {
                                        let _ = key.set_value("", &format!("{},0", exe));
                                    }

                                    let path = std::path::Path::new("Software\\Classes\\Typesafe.Document\\shell\\open\\command");
                                    if let Ok((key, _)) = hkcu.create_subkey(&path) {
                                        let _ = key.set_value("", &format!("\"{}\" \"%1\"", exe));
                                    }
                                }
                            }
                        },
                        SettingsTab::APIs => {
                            ui.heading("Compilation");
                            ui.add_space(4.0);
                            if ui.checkbox(&mut self.settings.auto_compile, "Auto-compile on Save").changed() {
                                self.settings.save();
                            }
                        },
                        SettingsTab::About => {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                egui_commonmark::CommonMarkViewer::new("about_md_viewer")
                                    .show(ui, &mut self.markdown_cache, self.readme_content);
                            });
                        },
                        SettingsTab::License => {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.label(egui::RichText::new(self.license_content).monospace());
                            });
                        }
                    }
                });
        }
        self.settings.show_settings_window = show_settings;

        // ====== RENAME DIALOG ======
        if self.rename_dialog_open {
            let mut open = true;
            egui::Window::new("Rename File")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(format!("Renaming: {}", self.rename_target_path.file_name().unwrap_or_default().to_string_lossy()));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("New Name:");
                        let response = ui.text_edit_singleline(&mut self.rename_new_name);
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            // Pressing Enter will effectively be handled by the user clicking Rename,
                            // or we could trigger it here.
                        }
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.rename_dialog_open = false;
                        }
                        if ui.button("Rename").clicked() {
                             let new_path = self.rename_target_path.with_file_name(&self.rename_new_name);
                             if let Err(e) = std::fs::rename(&self.rename_target_path, &new_path) {
                                 rfd::MessageDialog::new()
                                     .set_title("Rename Failed")
                                     .set_description(&format!("Error: {}", e))
                                     .show();
                             } else {
                                 // If we renamed the currently open file, update file_path
                                 let old_path_str = self.rename_target_path.to_string_lossy().to_string();
                                 let new_path_str = new_path.to_string_lossy().to_string();

                                 if self.file_path == old_path_str {
                                     self.file_path = new_path_str.clone();
                                 }
                                 // If we renamed the root file, update root_file
                                 if let Some(root) = &self.root_file {
                                     if root == &old_path_str {
                                         self.root_file = Some(new_path_str);
                                     }
                                 }
                                 self.rename_dialog_open = false;
                             }
                        }
                    });
                });
            if !open {
                self.rename_dialog_open = false;
            }
        }

        // ====== POP-OUT PDF VIEWER WINDOW ======
        if self.show_pdf_popup && self.pdf_path.is_some() {
            let viewport_id = egui::ViewportId::from_hash_of("pdf_viewer_viewport");
            self.pdf_popup_viewport_id = viewport_id;

            let mut show_popup = true;
            ctx.show_viewport_immediate(
                viewport_id,
                egui::ViewportBuilder::default()
                    .with_title("PDF Viewer - Typesafe")
                    .with_inner_size([900.0, 700.0]),
                |ctx, _class| {
                    ctx.request_repaint();
                    if ctx.input(|i| i.viewport().close_requested()) {
                        show_popup = false;
                    }

                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        if ui.button("◀").on_hover_text("Previous Page").clicked() && self.current_page > 0 {
                            self.current_page -= 1;
                        }
                        ui.label(
                            egui::RichText::new(format!(
                                "{}/{}",
                                self.current_page + 1,
                                self.page_count.max(1)
                            ))
                            .color(theme.text_secondary)
                            .strong(),
                        );
                        if ui.button("▶").on_hover_text("Next Page").clicked() && self.current_page + 1 < self.page_count {
                            self.current_page += 1;
                        }

                        ui.separator();

                        let mut zoom = self.popout_zoom;
                        if ui.button("➖").clicked() {
                            zoom = (zoom - 0.1).clamp(0.1, 5.0);
                            self.popout_fit_mode = PdfFitMode::Normal;
                        }
                        if ui.add(egui::Slider::new(&mut zoom, 0.1..=5.0).step_by(0.05).show_value(false)).changed() {
                            self.popout_fit_mode = PdfFitMode::Normal;
                        }
                        if ui.button("➕").clicked() {
                            zoom = (zoom + 0.1).clamp(0.1, 5.0);
                            self.popout_fit_mode = PdfFitMode::Normal;
                        }

                        let mut zoom_percent = (zoom * 100.0).round() as u32;
                        if ui.add(egui::DragValue::new(&mut zoom_percent).suffix("%").clamp_range(10..=500).speed(1.0)).changed() {
                            zoom = zoom_percent as f32 / 100.0;
                            self.popout_fit_mode = PdfFitMode::Normal;
                        }

                        if (zoom - self.popout_zoom).abs() > 0.001 {
                            self.popout_zoom = zoom;
                            self.pdf_textures.clear();
                        }

                        if ui.button("⟳").on_hover_text("Reset Zoom").clicked() {
                            self.popout_zoom = 1.0;
                            self.popout_fit_mode = PdfFitMode::Normal;
                            self.pdf_textures.clear();
                        }

                        ui.separator();

                        if ui.button("↔").on_hover_text("Fit Width").clicked() {
                            self.popout_fit_mode = PdfFitMode::FitWidth;
                            self.pdf_textures.clear();
                        }
                        if ui.button("⬚").on_hover_text("Fit Page").clicked() {
                            self.popout_fit_mode = PdfFitMode::FitPage;
                            self.pdf_textures.clear();
                        }
                        if ui.toggle_value(&mut self.popout_multi_page_view, "::").on_hover_text("Grid View").clicked() {
                             self.pdf_textures.clear();
                        }

                        ui.separator();

                        if ui.button("🔍").on_hover_text("Search (Ctrl+F)").clicked() {
                            self.show_pdf_search = !self.show_pdf_search;
                        }
                    });

                    if self.show_pdf_search {
                        ui.horizontal(|ui| {
                            ui.label("Search:");
                            ui.text_edit_singleline(&mut self.pdf_search_query);
                            if ui.button("✖").clicked() {
                                self.show_pdf_search = false;
                                self.pdf_search_query.clear();
                            }
                        });
                    }

                    ui.separator();

                    // Auto-scale zoom for popout
                    if !self.page_sizes.is_empty() && self.popout_fit_mode != PdfFitMode::Normal {
                        if let Some((w, h)) = self.page_sizes.get(&self.current_page).or_else(|| self.page_sizes.values().next()) {
                            let available_w = (ui.available_width() - 32.0).max(100.0);
                            let available_h = (ui.available_height() - 32.0).max(100.0);

                            if self.popout_fit_mode == PdfFitMode::FitWidth {
                                self.popout_zoom = available_w / w;
                            } else if self.popout_fit_mode == PdfFitMode::FitPage {
                                let zoom_h = available_h / h;
                                let zoom_w = available_w / w;
                                self.popout_zoom = zoom_h.min(zoom_w);
                            }
                        }
                    }

                    let scroll_zoom = ctx.input(|i| if i.modifiers.ctrl { i.raw_scroll_delta.y } else { 0.0 });
                    let pinch_zoom = ctx.input(|i| i.zoom_delta());

                    if (scroll_zoom.abs() > 0.0 || pinch_zoom != 1.0) && ui.rect_contains_pointer(ui.max_rect()) {
                        let mut new_zoom = self.popout_zoom;
                        if scroll_zoom.abs() > 0.0 {
                            new_zoom += scroll_zoom * 0.0015;
                        }
                        if pinch_zoom != 1.0 {
                            new_zoom *= pinch_zoom;
                        }
                        self.popout_zoom = new_zoom.clamp(0.3, 4.0);
                        self.pdf_textures.clear();
                    }

                    egui::ScrollArea::both()
                        .id_source("pdf_popup_scroll")
                        .show(ui, |ui| {
                            if self.page_count > 0 {
                                if self.popout_multi_page_view {
                                     ui.horizontal_wrapped(|ui| {
                                         ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);

                                         for page_idx in 0..self.page_count {
                                             self.render_page(ctx, page_idx);
                                             if let Some(tex) = self.pdf_textures.get(&page_idx) {
                                                 let (pw, ph) = self.page_sizes.get(&page_idx).copied().unwrap_or((595.0, 842.0));
                                                 let aspect = pw / ph;
                                                 let display_width = pw * self.popout_zoom;
                                                 let display_height = display_width / aspect;

                                                 ui.add(egui::Image::new((
                                                     tex.id(),
                                                     Vec2::new(display_width, display_height),
                                                 )).sense(egui::Sense::click()));
                                             }
                                         }
                                     });
                                } else {
                                    ui.vertical_centered(|ui| {
                                        for page_idx in 0..self.page_count {
                                            self.render_page(ctx, page_idx);

                                            if let Some(tex) = self.pdf_textures.get(&page_idx) {
                                                let (pw, ph) = self.page_sizes.get(&page_idx).copied().unwrap_or((595.0, 842.0));
                                                let aspect = pw / ph;
                                                let display_width = pw * self.popout_zoom;
                                                let display_height = display_width / aspect;

                                                ui.add(egui::Image::new((
                                                    tex.id(),
                                                    Vec2::new(display_width, display_height),
                                                )).sense(egui::Sense::click()));
                                            }
                                        }
                                    });
                                }
                            } else {
                                ui.label("No PDF loaded");
                            }
                        });
                    });
                },
            );

            if !show_popup {
                self.show_pdf_popup = false;
                self.pdf_textures.clear();
            }
        }

        // ====== LEFT PANEL (FILES) ======
        if self.show_file_panel {
            egui::SidePanel::left("files_panel")
                .resizable(true)
                .default_width(200.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new("📂").frame(false)).on_hover_text("Collapse").clicked() {
                            self.show_file_panel = false;
                        }
                        ui.label(
                            egui::RichText::new("FILES")
                                .color(theme.text_secondary)
                                .strong(),
                        );
                    });
                    ui.separator();

                    let height = ui.available_height();

                    // Top half: Files
                    ui.push_id("files_section", |ui| {
                        egui::ScrollArea::vertical().max_height(height * 0.5).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if ui.button("🏠").on_hover_text("Home").clicked() {
                                    let home = std::env::var("USERPROFILE")
                                        .or_else(|_| std::env::var("HOME"))
                                        .map(std::path::PathBuf::from)
                                        .unwrap_or_else(|_| std::path::PathBuf::from("."));
                                    self.current_dir = home;
                                }
                                if ui.button("⬆").on_hover_text("Up Directory").clicked() {
                                    if let Some(parent) = self.current_dir.parent() {
                                        self.current_dir = parent.to_path_buf();
                                    }
                                }
                                if ui.button("🔄").on_hover_text("Refresh").clicked() {
                                    ctx.request_repaint();
                                }
                                if ui.button("➕").on_hover_text("New File").clicked() {
                                    let mut proceed = true;
                                    if self.is_dirty {
                                        if self.settings.autosave_on_change {
                                            self.save_file(ctx, false);
                                        } else {
                                            let confirmed = rfd::MessageDialog::new()
                                                .set_title("Unsaved Changes")
                                                .set_description("Do you want to save changes to the current file?")
                                                .set_buttons(rfd::MessageButtons::YesNoCancel)
                                                .show();
                                            match confirmed {
                                                rfd::MessageDialogResult::Yes => self.save_file(ctx, true),
                                                rfd::MessageDialogResult::No => {},
                                                rfd::MessageDialogResult::Cancel => proceed = false,
                                                _ => {},
                                            }
                                        }
                                    }
                                    if proceed {
                                        self.editor_content = "\\documentclass{article}\n\\begin{document}\n\n\\end{document}".to_string();
                                        self.file_path = self.current_dir.join("untitled.tex").to_string_lossy().to_string();
                                        self.settings.last_file = None;
                                        self.settings.save();
                                    }
                                }
                                ui.label(
                                    egui::RichText::new(self.current_dir.file_name().unwrap_or_default().to_string_lossy())
                                        .small()
                                        .color(theme.text_secondary)
                                );
                            });
                            ui.separator();

                            if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
                                let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                                entries.sort_by_key(|e| {
                                    let is_file = e.metadata().map(|m| m.is_file()).unwrap_or(true);
                                    (is_file, e.file_name())
                                });

                                for entry in entries {
                                    let path = entry.path();
                                    let name = entry.file_name().to_string_lossy().to_string();
                                    let is_dir = path.is_dir();

                                    let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                                    let allowed_exts = ["tex", "bib", "cls", "sty", "pdf", "png", "jpg", "jpeg", "md", "txt"];
                                    let is_allowed = allowed_exts.contains(&ext.as_str());

                                    if !is_dir && !is_allowed { continue; }

                                    let icon = if is_dir { "📂" }
                                        else if ext == "pdf" { "📑" }
                                        else if ["png", "jpg", "jpeg"].contains(&ext.as_str()) { "🖼" }
                                        else { "📄" };

                                    let is_selected = path.to_string_lossy() == self.file_path;

                                    let is_root = self.root_file.as_ref().map_or(false, |r| r == &path.to_string_lossy().to_string());
                                    let label_text = if is_root {
                                        format!("{} {} [ROOT]", icon, name)
                                    } else {
                                        format!("{} {}", icon, name)
                                    };

                                    let text = if is_selected {
                                        egui::RichText::new(label_text).color(theme.accent).strong()
                                    } else {
                                        egui::RichText::new(label_text)
                                    };

                                    let btn = ui.add(egui::Button::new(text).frame(false)).on_hover_cursor(egui::CursorIcon::PointingHand);

                                    btn.context_menu(|ui| {
                                        if !is_dir {
                                            if ui.button("Set as Root File").clicked() {
                                                self.root_file = Some(path.to_string_lossy().to_string());
                                                ui.close_menu();
                                            }
                                            if ui.button("✏ Rename").clicked() {
                                                self.rename_target_path = path.clone();
                                                self.rename_new_name = name.clone();
                                                self.rename_dialog_open = true;
                                                ui.close_menu();
                                            }
                                            ui.separator();
                                            if ui.button("🗑 Delete File").clicked() {
                                                if rfd::MessageDialog::new()
                                                    .set_title("Delete File")
                                                    .set_description(&format!("Are you sure you want to delete {}?", name))
                                                    .set_buttons(rfd::MessageButtons::YesNo)
                                                    .show() == rfd::MessageDialogResult::Yes
                                                {
                                                    let _ = std::fs::remove_file(&path);
                                                    ui.close_menu();
                                                    ctx.request_repaint();
                                                }
                                            }
                                        }
                                    });

                                    if btn.clicked() {
                                        if is_dir {
                                            self.current_dir = path;
                                        } else if is_allowed {
                                            if ["tex", "bib", "cls", "sty", "md", "txt", "pdf"].contains(&ext.as_str()) {
                                                // Autosave previous file
                                                if self.settings.autosave_on_change && self.is_dirty && !self.file_path.is_empty() && self.file_path != "untitled.tex" {
                                                    if let Err(e) = std::fs::write(&self.file_path, &self.editor_content) {
                                                        self.compilation_log = format!("Error autosaving: {}\n", e);
                                                    }
                                                }

                                                self.file_path = path.to_string_lossy().to_string();
                                                let path_str = self.file_path.clone();
                                                self.load_file(ctx, &path_str);
                                                self.settings.last_file = Some(self.file_path.clone());
                                                self.settings.save();
                                                if self.current_file_type == CurrentFileType::Tex {
                                                    self.update_outline();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    });

                    ui.separator();

                    // Bottom half: Outline
                    ui.push_id("outline_section", |ui| {
                         ui.label(
                             egui::RichText::new("OUTLINE")
                                 .color(theme.text_secondary)
                                 .strong(),
                         );
                         ui.separator();
                         egui::ScrollArea::vertical().show(ui, |ui| {
                            fn render_tree(
                                ui: &mut egui::Ui,
                                nodes: &mut Vec<StructureNode>,
                                ctx: &egui::Context,
                                jump_action: &mut Option<(String, usize)>
                            ) {
                                for node in nodes {
                                    ui.horizontal(|ui| {
                                        ui.add_space(node.level as f32 * 16.0);
                                        let collapse_icon = if node.expanded { "⏷" } else { "⏵" };

                                        if !node.children.is_empty() {
                                            if ui.add(egui::Button::new(collapse_icon).frame(false).small()).clicked() {
                                                node.expanded = !node.expanded;
                                            }
                                        } else {
                                            ui.add_space(14.0);
                                        }

                                        let type_icon = match node.kind {
                                            NodeKind::Section => "§",
                                            NodeKind::Figure => "🖼",
                                            NodeKind::Table => "▦",
                                            NodeKind::Theorem => "T",
                                            NodeKind::Citation => "❞",
                                            NodeKind::Unknown => "•",
                                        };
                                        ui.label(egui::RichText::new(type_icon).color(egui::Color32::GRAY));

                                        // Use a SelectableLabel or Label with truncation instead of a Button
                                        // This allows it to shrink/truncate properly within the horizontal layout
                                        let label_response = ui.add(
                                            egui::Label::new(egui::RichText::new(&node.label))
                                                .truncate(true)
                                                .sense(egui::Sense::click())
                                        );

                                        if label_response.clicked() {
                                            // Jump to line + 1 to account for 0-indexing in outline vs 1-indexing in editor
                                            *jump_action = Some((node.file_path.clone(), node.line.saturating_sub(1)));
                                        }
                                    });

                                    if node.expanded && !node.children.is_empty() {
                                        render_tree(ui, &mut node.children, ctx, jump_action);
                                    }
                                }
                            }

                            let mut jump = None;
                            render_tree(ui, &mut self.outline_nodes, ctx, &mut jump);

                            if let Some((file, line)) = jump {
                                if file != self.file_path && !file.is_empty() {
                                    let path_to_load = if std::path::Path::new(&file).is_absolute() {
                                        std::path::PathBuf::from(&file)
                                    } else {
                                        self.current_dir.join(&file)
                                    };

                                    if path_to_load.exists() {
                                         self.file_path = path_to_load.to_string_lossy().to_string();
                                         let p = self.file_path.clone();
                                         self.load_file(ctx, &p);
                                         self.settings.last_file = Some(self.file_path.clone());
                                         self.settings.save();
                                    }
                                }

                                if let Some(mut state) = egui::TextEdit::load_state(ctx, egui::Id::new("main_editor")) {
                                    let char_idx = self.editor_content.lines().take(line).map(|l| l.len() + 1).sum::<usize>();
                                    state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(char_idx))));
                                    state.store(ctx, egui::Id::new("main_editor"));
                                    ctx.memory_mut(|m| m.request_focus(egui::Id::new("main_editor")));
                                    self.pending_cursor_scroll = Some(char_idx);
                                }
                            }

                            if self.outline_nodes.is_empty() {
                                ui.label(egui::RichText::new("No sections.").small().italics());
                            }
                        });
                    });
                });
        } else {
            egui::SidePanel::left("files_panel_slim")
                .resizable(false)
                .exact_width(32.0)
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.vertical_centered(|ui| {
                        if ui.button("📂").on_hover_text("Expand Files").clicked() {
                            self.show_file_panel = true;
                        }
                    });
                });
        }

        // ====== RIGHT PANEL (PREVIEW) ======
        // Hide preview panel for PDF files (they use central panel) and Markdown files
        if self.show_preview_panel && self.current_file_type == CurrentFileType::Tex {
        let files_gap = if self.show_file_panel { 250.0 } else { 40.0 };
        let min_editor_width = 720.0;
        let max_preview_w = (ctx.screen_rect().width() - files_gap - min_editor_width).max(300.0);

        egui::SidePanel::right("preview_panel")
            .resizable(true)
            .default_width(600.0)
            .min_width(300.0)
            .max_width(max_preview_w)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                     if ui.add(egui::Button::new("📄").frame(false)).on_hover_text("Collapse").clicked() {
                         self.show_preview_panel = false;
                     }
                     ui.label(
                        egui::RichText::new("PREVIEW")
                            .color(theme.text_secondary)
                            .strong(),
                    );
                     ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                         if ui.button("📤 Pop-out").on_hover_text("Open in separate window (Ctrl+Alt+P)").clicked() {
                             self.show_pdf_popup = true;
                         }
                     });
                });
                ui.separator();



                // Preview Controls
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

                    // Page Navigation
                    if ui.button("◀").on_hover_text("Previous Page").clicked() && self.current_page > 0 {
                        self.current_page -= 1;
                    }
                    ui.label(
                        egui::RichText::new(format!(
                            "{}/{}",
                            self.current_page + 1,
                            self.page_count.max(1)
                        ))
                        .color(theme.text_secondary)
                        .strong(),
                    );
                    if ui.button("▶").on_hover_text("Next Page").clicked() && self.current_page + 1 < self.page_count {
                        self.current_page += 1;
                    }

                    ui.separator();

                    // Zoom Controls
                    let mut zoom = self.zoom;
                    if ui.button("➖").clicked() {
                        zoom = (zoom - 0.1).clamp(0.1, 5.0);
                        self.fit_mode = PdfFitMode::Normal;
                    }
                    if ui.add(egui::Slider::new(&mut zoom, 0.1..=5.0).step_by(0.05).show_value(false)).changed() {
                        self.fit_mode = PdfFitMode::Normal;
                    }
                    if ui.button("➕").clicked() {
                        zoom = (zoom + 0.1).clamp(0.1, 5.0);
                        self.fit_mode = PdfFitMode::Normal;
                    }

                    let mut zoom_percent = (zoom * 100.0).round() as u32;
                    if ui.add(egui::DragValue::new(&mut zoom_percent).suffix("%").clamp_range(10..=500).speed(1.0)).changed() {
                        zoom = zoom_percent as f32 / 100.0;
                        self.fit_mode = PdfFitMode::Normal;
                    }

                    if (zoom - self.zoom).abs() > 0.001 {
                        self.zoom = zoom;
                        self.pdf_textures.clear();
                    }

                    if ui.button("⟳").on_hover_text("Reset Zoom").clicked() {
                        self.zoom = 1.0;
                        self.fit_mode = PdfFitMode::Normal;
                        self.pdf_textures.clear();
                    }

                    ui.separator();

                    // View Mode Icons
                    if ui.button("↔").on_hover_text("Fit Width").clicked() {
                        self.fit_mode = PdfFitMode::FitWidth;
                        self.pdf_textures.clear();
                    }
                    if ui.button("⬚").on_hover_text("Fit Page").clicked() {
                        self.fit_mode = PdfFitMode::FitPage;
                        self.pdf_textures.clear();
                    }
                    if ui.toggle_value(&mut self.pdf_multi_page_view, "::").on_hover_text("Grid View").clicked() {
                         self.pdf_textures.clear();
                    }

                    ui.separator();

                    ui.toggle_value(&mut self.magnifier_enabled, "🔍").on_hover_text("Magnifier");
                });

                // Ctrl+Scroll zoom
                let scroll_zoom = ctx.input(|i| if i.modifiers.ctrl { i.raw_scroll_delta.y } else { 0.0 });
                if scroll_zoom.abs() > 0.0 {
                    self.zoom = (self.zoom + scroll_zoom * 0.0015).clamp(0.3, 4.0);
                    self.pdf_textures.clear();
                }

                // Top-down layout with manual height calculation for preview vs log
                let total_height = ui.available_height();
                let log_header_height = 30.0;
                let log_height = if self.show_log { 150.0 } else { 0.0 };

                // If log is shown, preview gets what's left minus header. If not, preview gets everything minus header.
                // We reserve space for the toggle bar (log_header_height) and a generous buffer to prevent cutoff.
                let preview_height = (total_height - log_height - log_header_height - 24.0).max(50.0);
                let preview_container_width = ui.available_width();

                // Auto-scale zoom if needed
                if !self.page_sizes.is_empty() && self.fit_mode != PdfFitMode::Normal {
                    if let Some((w, h)) = self.page_sizes.get(&self.current_page).or_else(|| self.page_sizes.values().next()) {
                        let mut target_zoom = self.zoom;
                        let available_w = (preview_container_width - 32.0).max(100.0);

                        if self.fit_mode == PdfFitMode::FitWidth {
                             target_zoom = available_w / w;
                        } else if self.fit_mode == PdfFitMode::FitPage {
                             let target_h = preview_height;
                             let zoom_h = target_h / h;
                             let zoom_w = available_w / w;
                             target_zoom = zoom_h.min(zoom_w);
                        }

                        if (target_zoom - self.zoom).abs() > 0.005 {
                             self.zoom = target_zoom;
                             self.pdf_textures.clear();
                             ctx.request_repaint();
                        }
                    }
                }

                // 1. PDF Preview
                egui::ScrollArea::both()
                    .id_source("pdf_preview_scroll")
                    .max_height(preview_height)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            if self.page_count > 0 {
                                // Lazily render all pages
                                for page_idx in 0..self.page_count {
                                    self.render_page(ctx, page_idx);

                                    if let Some(tex) = self.pdf_textures.get(&page_idx) {
                                        if let Some([w, h]) = self.preview_size {
                                        let aspect = w as f32 / h as f32;
                                        // Use the container width captured outside the scroll area to avoid progressive scaling loop
                                        let available_w = (preview_container_width - 20.0).max(100.0);
                                        // With auto-zoom logic above, self.zoom should be correct for Fit modes
                                        let display_width = if let Some((pw, _)) = self.page_sizes.get(&page_idx) {
                                            pw * self.zoom
                                        } else {
                                            available_w * self.zoom
                                        };
                                        let display_height = display_width / aspect;

                                        let img_resp = ui.add(egui::Image::new((
                                            tex.id(),
                                            Vec2::new(display_width, display_height),
                                        )).sense(egui::Sense::click()));

                                        if let Some((p, rel_y)) = self.pending_scroll_target {
                                            if p == page_idx {
                                                let y_pos = img_resp.rect.min.y + img_resp.rect.height() * rel_y;
                                                let target_rect = egui::Rect::from_min_size(
                                                    egui::pos2(img_resp.rect.min.x, y_pos),
                                                    egui::vec2(img_resp.rect.width(), 1.0)
                                                );
                                                ui.scroll_to_rect(target_rect, Some(egui::Align::Center));
                                                self.pending_scroll_target = None;
                                            }
                                        }

                                        if self.page_count > 0 {
                                            if img_resp.double_clicked() {
                                                let mut jumped = false;

                                                // Try SyncTeX inverse search
                                                if let Some(pos) = img_resp.interact_pointer_pos() {
                                                    if let Some((_, page_h)) = self.page_sizes.get(&page_idx) {
                                                        let rel_y = (pos.y - img_resp.rect.min.y) / img_resp.rect.height();
                                                        let y_pt = rel_y * page_h;

                                                        if let Some(pdf_path) = &self.pdf_path {
                                                            let stem = pdf_path.file_stem().unwrap_or_default();
                                                            let parent = pdf_path.parent().unwrap_or(std::path::Path::new("."));
                                                            let synctex_path = parent.join(format!("{}.synctex.gz", stem.to_string_lossy()));

                                                            if synctex_path.exists() {
                                                                if let Ok(file) = std::fs::File::open(&synctex_path) {
                                                                    let mut decoder = GzDecoder::new(file);
                                                                    let mut content = String::new();
                                                                    if decoder.read_to_string(&mut content).is_ok() {
                                                                        // Manual Inverse Search
                                                                        let target_page = page_idx + 1;
                                                                        let mut current_page = 0;
                                                                        let mut best_dist = f32::MAX;
                                                                        let mut best_line = 0;
                                                                        let target_y_sp = y_pt * 65536.0;

                                                                        for line in content.lines() {
                                                                            if line.starts_with('{') {
                                                                                if let Ok(p) = line[1..].parse::<usize>() { current_page = p; }
                                                                            } else if current_page == target_page {
                                                                                let first = line.chars().next().unwrap_or(' ');
                                                                                if "xkgvh".contains(first) {
                                                                                    if let Some(colon) = line.find(':') {
                                                                                        let parts: Vec<&str> = line[1..colon].split(',').collect();
                                                                                        if parts.len() >= 2 {
                                                                                            if let Ok(rec_line) = parts[1].parse::<usize>() {
                                                                                                let coords: Vec<&str> = line[colon+1..].split(',').collect();
                                                                                                if coords.len() >= 2 {
                                                                                                    if let Ok(v_sp) = coords[1].parse::<f32>() {
                                                                                                        let dist = (v_sp - target_y_sp).abs();
                                                                                                        if dist < best_dist && dist < 65536.0 * 50.0 { // Increased tolerance
                                                                                                            best_dist = dist;
                                                                                                            best_line = rec_line;
                                                                                                        }
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }

                                                                            if best_line > 0 {
                                                                                let line = best_line - 1;
                                                                                // Calculate accurate char index (handling UTF-8 and mixed newlines)
                                                                                let char_idx = if line == 0 { 0 } else {
                                                                                    self.editor_content.chars()
                                                                                        .enumerate()
                                                                                        .filter(|&(_, c)| c == '\n')
                                                                                        .nth(line - 1)
                                                                                        .map(|(i, _)| i + 1)
                                                                                        .unwrap_or(0)
                                                                                };

                                                                                if let Some(mut state) = egui::TextEdit::load_state(ctx, egui::Id::new("main_editor")) {
                                                                                        // Calculate line length for selection
                                                                                        let line_len = self.editor_content.chars().skip(char_idx).take_while(|&c| c != '\n').count();

                                                                                        state.cursor.set_char_range(Some(egui::text::CCursorRange::two(
                                                                                            egui::text::CCursor::new(char_idx),
                                                                                            egui::text::CCursor::new(char_idx + line_len)
                                                                                        )));
                                                                                        state.store(ctx, egui::Id::new("main_editor"));

                                                                                        // Force focus to editor so it scrolls and shows cursor
                                                                                        ctx.memory_mut(|m| m.request_focus(egui::Id::new("main_editor")));
                                                                                        self.pending_cursor_scroll = Some(char_idx);
                                                                                        jumped = true;
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                // Fallback to naive estimation
                                                if !jumped {
                                                    let total_lines = self.editor_content.lines().count().max(1);
                                                    let rel_in_page = if let Some(pos) = img_resp.interact_pointer_pos() {
                                                        ((pos.y - img_resp.rect.min.y) / img_resp.rect.height().max(1.0)).clamp(0.0, 1.0)
                                                    } else {
                                                        0.5
                                                    };
                                                    let rel = (page_idx as f32 + rel_in_page) / (self.page_count as f32);
                                                    let target_line = ((rel * total_lines as f32).round() as usize).min(total_lines.saturating_sub(1));
                                                    if let Some(mut state) = egui::TextEdit::load_state(ctx, egui::Id::new("main_editor")) {
                                                        let char_idx = self.editor_content.lines().take(target_line).map(|l| l.len() + 1).sum::<usize>();
                                                        let line_len = self.editor_content.lines().nth(target_line).map(|l| l.len()).unwrap_or(0);
                                                        state.cursor.set_char_range(Some(egui::text::CCursorRange::two(egui::text::CCursor::new(char_idx), egui::text::CCursor::new(char_idx + line_len))));
                                                        state.store(ctx, egui::Id::new("main_editor"));
                                                    }
                                                }
                                            }
                                        }

                                        // Hover magnifier (full-page lens)
                                        if self.magnifier_enabled && img_resp.hovered() {
                                            if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                                                let lens = Vec2::new(self.magnifier_size, self.magnifier_size / aspect.max(0.1));
                                                let rect = img_resp.rect;
                                                let rel = (pos - rect.min) / rect.size();
                                                let half_u = (lens.x / (display_width * self.magnifier_zoom)).min(0.5);
                                                let half_v = (lens.y / (display_height * self.magnifier_zoom)).min(0.5);
                                                let center_u = rel.x.clamp(0.0, 1.0);
                                                let center_v = rel.y.clamp(0.0, 1.0);
                                                let uv_min = egui::pos2((center_u - half_u).clamp(0.0, 1.0), (center_v - half_v).clamp(0.0, 1.0));
                                                let uv_max = egui::pos2((center_u + half_u).clamp(0.0, 1.0), (center_v + half_v).clamp(0.0, 1.0));
                                                egui::Area::new(egui::Id::new(format!("magnifier_{}", page_idx)))
                                                    .order(egui::Order::Tooltip)
                                                    .fixed_pos(pos + Vec2::new(12.0, 12.0))
                                                    .show(ctx, |ui| {
                                                        ui.set_min_size(lens);
                                                        ui.set_max_size(lens);
                                                        ui.add(egui::Image::new((tex.id(), lens * self.magnifier_zoom)).uv(egui::Rect::from_min_max(uv_min, uv_max)));
                                                    });
                                            }
                                        }

                                        ui.add_space(10.0);
                                    }
                                }
                            }
                        } else {
                            ui.label(
                                egui::RichText::new(&self.preview_status)
                                    .color(theme.text_secondary),
                            );
                        }
                    });
                });

                ui.separator();

                // 2. Footer Bar (Log Toggle)
                ui.horizontal(|ui| {
                    let icon = if self.show_log { "▼" } else { "▶" };
                    if ui.button(format!("{} Log", icon)).clicked() {
                        self.show_log = !self.show_log;
                    }
                    if !self.compilation_log.is_empty() {
                            ui.label(egui::RichText::new(if self.is_compiling { "Compiling..." } else { "Status:" }).small());
                    }
                });
                ui.add_space(5.0);

                // 3. Compilation Log (if shown)
                if self.show_log {
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .id_source("log_scroll")
                        .max_height(ui.available_height())
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            if !self.diagnostics.is_empty() {
                                    ui.label(egui::RichText::new("Diagnostics (Click to Jump):").strong().color(egui::Color32::from_rgb(255, 100, 100)));
                                    for diag in &self.diagnostics {
                                        if ui.link(format!("Line {}: {}", diag.line, diag.message)).clicked() {
                                            let char_idx = self.editor_content.lines().take(diag.line.saturating_sub(1)).map(|l| l.len() + 1).sum::<usize>();
                                            if let Some(mut state) = egui::TextEdit::load_state(ctx, egui::Id::new("main_editor")) {
                                                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(char_idx))));
                                                state.store(ctx, egui::Id::new("main_editor"));
                                            }
                                        }
                                    }
                                    ui.separator();
                            }
                            ui.add(
                                egui::TextEdit::multiline(&mut self.compilation_log)
                                    .font(TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .code_editor(),
                            );
                        });
                }
            });
        } else {
             egui::SidePanel::right("preview_panel_slim")
                .resizable(false)
                .exact_width(32.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        if ui.button("📄").on_hover_text("Expand Preview").clicked() {
                            self.show_preview_panel = true;
                        }
                    });
                });
        }

        // Run debounced checks
        let now = ctx.input(|i| i.time);
        if self.checks_dirty && now - self.last_edit_time > 0.5 {
            self.cached_syntax_errors = self.check_syntax(&self.editor_content);
            self.cached_spell_errors = self.check_spelling(&self.editor_content);
            self.checks_dirty = false;
        } else if self.checks_dirty {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        // ====== CENTRAL PANEL (EDITOR OR PDF VIEWER) ======
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show different content based on file type
            match self.current_file_type {
                CurrentFileType::Pdf => {
                    // Show PDF preview in full editor area
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new("📤").frame(false)).on_hover_text("Pop-out (Ctrl+Shift+P)").clicked() {
                            self.show_pdf_popup = true;
                            self.pdf_textures.clear();
                        }
                        ui.label(
                            egui::RichText::new("PDF VIEWER")
                                .color(theme.text_secondary)
                                .strong(),
                        );
                    });
                    ui.separator();

                    if self.show_pdf_popup {
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                egui::RichText::new("PDF is open in separate window")
                                    .color(theme.text_secondary)
                            );
                            if ui.button("Return to Main Window").clicked() {
                                self.show_pdf_popup = false;
                                self.pdf_textures.clear();
                            }
                        });
                        return;
                    }

                    // PDF Controls
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        if ui.button("◀").on_hover_text("Previous Page").clicked() && self.current_page > 0 {
                            self.current_page -= 1;
                        }
                        ui.label(
                            egui::RichText::new(format!(
                                "{}/{}",
                                self.current_page + 1,
                                self.page_count.max(1)
                            ))
                            .color(theme.text_secondary)
                            .strong(),
                        );
                        if ui.button("▶").on_hover_text("Next Page").clicked() && self.current_page + 1 < self.page_count {
                            self.current_page += 1;
                        }

                        ui.separator();

                        let mut zoom = self.zoom;
                        if ui.button("➖").clicked() {
                            zoom = (zoom - 0.1).clamp(0.1, 5.0);
                            self.fit_mode = PdfFitMode::Normal;
                        }
                        if ui.add(egui::Slider::new(&mut zoom, 0.1..=5.0).step_by(0.05).show_value(false)).changed() {
                            self.fit_mode = PdfFitMode::Normal;
                        }
                        if ui.button("➕").clicked() {
                            zoom = (zoom + 0.1).clamp(0.1, 5.0);
                            self.fit_mode = PdfFitMode::Normal;
                        }

                        let mut zoom_percent = (zoom * 100.0).round() as u32;
                        if ui.add(egui::DragValue::new(&mut zoom_percent).suffix("%").clamp_range(10..=500).speed(1.0)).changed() {
                            zoom = zoom_percent as f32 / 100.0;
                            self.fit_mode = PdfFitMode::Normal;
                        }

                        if (zoom - self.zoom).abs() > 0.001 {
                            self.zoom = zoom;
                            self.pdf_textures.clear();
                        }

                        if ui.button("⟳").on_hover_text("Reset Zoom").clicked() {
                            self.zoom = 1.0;
                            self.fit_mode = PdfFitMode::Normal;
                            self.pdf_textures.clear();
                        }

                        ui.separator();

                        if ui.button("↔").on_hover_text("Fit Width").clicked() {
                            self.fit_mode = PdfFitMode::FitWidth;
                            self.pdf_textures.clear();
                        }
                        if ui.button("⬚").on_hover_text("Fit Page").clicked() {
                            self.fit_mode = PdfFitMode::FitPage;
                            self.pdf_textures.clear();
                        }
                        if ui.toggle_value(&mut self.pdf_multi_page_view, "::").on_hover_text("Grid View").clicked() {
                             self.pdf_textures.clear();
                        }

                        ui.separator();

                        if ui.button("🔍").on_hover_text("Search (Ctrl+F)").clicked() {
                            self.show_pdf_search = !self.show_pdf_search;
                        }
                    });

                    if self.show_pdf_search {
                        ui.horizontal(|ui| {
                            ui.label("Search:");
                            ui.text_edit_singleline(&mut self.pdf_search_query);
                            if ui.button("✖").clicked() {
                                self.show_pdf_search = false;
                                self.pdf_search_query.clear();
                            }
                        });
                        ui.separator();
                    }

                    // Display PDF
                    let scroll_zoom = ctx.input(|i| if i.modifiers.ctrl { i.raw_scroll_delta.y } else { 0.0 });
                    let pinch_zoom = ctx.input(|i| i.zoom_delta());

                    if (scroll_zoom.abs() > 0.0 || pinch_zoom != 1.0) && ui.rect_contains_pointer(ui.max_rect()) {
                        let mut new_zoom = self.zoom;
                        if scroll_zoom.abs() > 0.0 {
                            new_zoom += scroll_zoom * 0.0015;
                        }
                        if pinch_zoom != 1.0 {
                            new_zoom *= pinch_zoom;
                        }
                        self.zoom = new_zoom.clamp(0.3, 4.0);
                        self.pdf_textures.clear();
                    }

                    egui::ScrollArea::both()
                        .id_source("pdf_preview_scroll")
                        .show(ui, |ui| {
                             if self.page_count > 0 {
                                 if self.pdf_multi_page_view {
                                     ui.horizontal_wrapped(|ui| {
                                         ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);

                                         for page_idx in 0..self.page_count {
                                             self.render_page(ctx, page_idx);
                                             if let Some(tex) = self.pdf_textures.get(&page_idx) {
                                                 let (pw, ph) = self.page_sizes.get(&page_idx).copied().unwrap_or((595.0, 842.0));
                                                 let aspect = pw / ph;
                                                 let display_width = pw * self.zoom;
                                                 let display_height = display_width / aspect;

                                                 ui.add(egui::Image::new((
                                                     tex.id(),
                                                     Vec2::new(display_width, display_height),
                                                 )).sense(egui::Sense::click()));
                                             }
                                         }
                                     });
                                 } else {
                                     ui.vertical_centered(|ui| {
                                         for page_idx in 0..self.page_count {
                                             self.render_page(ctx, page_idx);
                                             if let Some(tex) = self.pdf_textures.get(&page_idx) {
                                                 let (pw, ph) = self.page_sizes.get(&page_idx).copied().unwrap_or((595.0, 842.0));
                                                 let aspect = pw / ph;
                                                 let display_width = pw * self.zoom;
                                                 let display_height = display_width / aspect;

                                                 ui.add(egui::Image::new((
                                                     tex.id(),
                                                     Vec2::new(display_width, display_height),
                                                 )).sense(egui::Sense::click()));
                                             }
                                         }
                                     });
                                 }
                             } else {
                                 ui.label("No PDF loaded");
                             }
                        });
                    return;
                }
                CurrentFileType::Markdown => {
                    ui.horizontal(|ui| {
                        ui.add(egui::Button::new(" ").frame(false).sense(egui::Sense::hover()));
                        ui.label(
                            egui::RichText::new("MARKDOWN PREVIEW")
                                .color(theme.text_secondary)
                                .strong(),
                        );
                    });
                    ui.separator();

                    egui::ScrollArea::both().show(ui, |ui| {
                        egui_commonmark::CommonMarkViewer::new("markdown_viewer")
                            .show(ui, &mut self.markdown_cache, &self.editor_content);
                    });

                    ui.separator();
                    ui.label(egui::RichText::new("⚠ PDF preview not available for Markdown files").italics().color(theme.warning));
                    return;
                }
                _ => {} // Continue with normal editor for .tex files
            }
            ui.horizontal(|ui| {
                ui.add(egui::Button::new(" ").frame(false).sense(egui::Sense::hover()));
                ui.label(
                    egui::RichText::new("EDITOR")
                        .color(theme.text_secondary)
                        .strong(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("▶ Build").clicked() {
                        self.compile(ctx);
                    }
                    ui.add_space(8.0);
                    if ui.button("💾 Save").clicked() {
                         if !self.file_path.is_empty() && self.file_path != "untitled.tex" {
                            if let Err(e) = std::fs::write(&self.file_path, &self.editor_content) {
                                self.compilation_log = format!("Error saving: {}\n", e);
                            } else {
                                self.is_dirty = false;
                            }
                        }
                    }
                });
            });
            ui.separator();

            // Quick insert toolbar
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                if ui.button(egui::RichText::new("𝐁").strong()).on_hover_text("Bold").clicked() { self.insert_snippet(ctx, "\\textbf{$1}"); }
                if ui.button(egui::RichText::new("𝐼").italics()).on_hover_text("Italic").clicked() { self.insert_snippet(ctx, "\\textit{$1}"); }
                if ui.button(egui::RichText::new("U").underline()).on_hover_text("Underline").clicked() { self.insert_snippet(ctx, "\\underline{$1}"); }
                ui.separator();
                if ui.button("H1").on_hover_text("Section").clicked() { self.insert_snippet(ctx, "\\section{$1}"); }
                if ui.button("H2").on_hover_text("Subsection").clicked() { self.insert_snippet(ctx, "\\subsection{$1}"); }
                if ui.button("H3").on_hover_text("Subsubsection").clicked() { self.insert_snippet(ctx, "\\subsubsection{$1}"); }
                ui.separator();
                if ui.button("α").on_hover_text("Alpha").clicked() { self.insert_snippet(ctx, "\\alpha"); }
                if ui.button("β").on_hover_text("Beta").clicked() { self.insert_snippet(ctx, "\\beta"); }
                if ui.button("π").on_hover_text("Pi").clicked() { self.insert_snippet(ctx, "\\pi"); }
                if ui.button("∑").on_hover_text("Sum").clicked() { self.insert_snippet(ctx, "\\sum_{$1}^{$2}"); }
                if ui.button("∫").on_hover_text("Integral").clicked() { self.insert_snippet(ctx, "\\int_{$1}^{$2}"); }
                if ui.button("∞").on_hover_text("Infinity").clicked() { self.insert_snippet(ctx, "\\infty"); }
                ui.separator();
                if ui.button("•").on_hover_text("Itemize").clicked() { self.insert_snippet(ctx, "\\begin{itemize}\n    \\item $1\n\\end{itemize}"); }
                if ui.button("1.").on_hover_text("Enumerate").clicked() { self.insert_snippet(ctx, "\\begin{enumerate}\n    \\item $1\n\\end{enumerate}"); }
                ui.separator();
                if ui.button("Title").on_hover_text("Title").clicked() { self.insert_snippet(ctx, "\\title{$1}"); }
                if ui.button("Auth").on_hover_text("Author").clicked() { self.insert_snippet(ctx, "\\author{$1}"); }
            });
            ui.separator();

            // Search & Replace Bar
            if self.show_search {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;

            // Search Input
            let search_resp = ui.add(egui::TextEdit::singleline(&mut self.search_query).desired_width(120.0).hint_text("Find..."));
            if search_resp.changed() {
                self.search_match_index = 0;
                self.checks_dirty = true;
                // Perform search
                self.search_matches.clear();
                        if !self.search_query.is_empty() {
                            let text = &self.editor_content;
                            let query = &self.search_query;
                            let matches = if self.search_case_sensitive {
                                text.match_indices(query).map(|(i, _)| i).collect::<Vec<_>>()
                            } else {
                                text.to_lowercase().match_indices(&query.to_lowercase()).map(|(i, _)| i).collect::<Vec<_>>()
                            };

                            for start_byte in matches {
                                // Filter whole word if needed
                                let mut valid = true;
                                if self.search_whole_word {
                                    let is_start_ok = start_byte == 0 || !text[..start_byte].chars().last().unwrap_or(' ').is_alphanumeric();
                                    let end_byte = start_byte + query.len();
                                    let is_end_ok = end_byte == text.len() || !text[end_byte..].chars().next().unwrap_or(' ').is_alphanumeric();
                                    if !is_start_ok || !is_end_ok {
                                        valid = false;
                                    }
                                }

                                if valid {
                                    // Convert byte index to char index for TextEdit
                                    let char_idx = text[..start_byte].chars().count();
                                    let len_chars = text[start_byte..start_byte + query.len()].chars().count();
                                    self.search_matches.push((char_idx, char_idx + len_chars));
                                }
                            }
                        }
                    }

                    // Replace Input
                    ui.add(egui::TextEdit::singleline(&mut self.replace_query).desired_width(120.0).hint_text("Replace..."));

                    // Options
                    if ui.toggle_value(&mut self.search_case_sensitive, "Aa").on_hover_text("Case Sensitive").clicked() {
                         // Force re-search by clearing
                         self.search_matches.clear();
                    }
                    if ui.toggle_value(&mut self.search_whole_word, "W").on_hover_text("Whole Word").clicked() {
                        self.search_matches.clear();
                    }

                    ui.separator();

                    // Match Navigation
                    let match_count = self.search_matches.len();
                    ui.label(if match_count > 0 {
                        format!("{}/{}", self.search_match_index + 1, match_count)
                    } else {
                        "0/0".to_string()
                    });

                    if ui.button("◀").clicked() && match_count > 0 {
                        if self.search_match_index == 0 {
                            self.search_match_index = match_count - 1;
                        } else {
                            self.search_match_index -= 1;
                        }
                        // Scroll to match
                        if let Some(&(start, end)) = self.search_matches.get(self.search_match_index) {
                            if let Some(mut state) = egui::TextEdit::load_state(ctx, egui::Id::new("main_editor")) {
                                state.cursor.set_char_range(Some(egui::text::CCursorRange::two(egui::text::CCursor::new(start), egui::text::CCursor::new(end))));
                                state.store(ctx, egui::Id::new("main_editor"));
                            }
                        }
                    }
                    if ui.button("▶").clicked() && match_count > 0 {
                        self.search_match_index = (self.search_match_index + 1) % match_count;
                         if let Some(&(start, end)) = self.search_matches.get(self.search_match_index) {
                            if let Some(mut state) = egui::TextEdit::load_state(ctx, egui::Id::new("main_editor")) {
                                state.cursor.set_char_range(Some(egui::text::CCursorRange::two(egui::text::CCursor::new(start), egui::text::CCursor::new(end))));
                                state.store(ctx, egui::Id::new("main_editor"));
                            }
                        }
                    }

                    ui.separator();

                    // Replace Actions
                    if ui.button("Replace").clicked() && match_count > 0 {
                        if let Some(&(start_char, end_char)) = self.search_matches.get(self.search_match_index) {
                             // Convert chars to bytes
                             let start_byte = self.editor_content.chars().take(start_char).map(|c| c.len_utf8()).sum::<usize>();
                             let end_byte = self.editor_content.chars().take(end_char).map(|c| c.len_utf8()).sum::<usize>();

                             if start_byte < self.editor_content.len() && end_byte <= self.editor_content.len() {
                                 self.editor_content.replace_range(start_byte..end_byte, &self.replace_query);
                                 self.is_dirty = true;
                                 self.search_matches.clear(); // Clear to force refresh
                             }
                        }
                    }

                    if ui.button("All").on_hover_text("Replace All").clicked() && match_count > 0 {
                         let from = &self.search_query;
                         let to = &self.replace_query;

                         if !self.search_whole_word && self.search_case_sensitive {
                             self.editor_content = self.editor_content.replace(from, to);
                         } else if !self.search_whole_word && !self.search_case_sensitive {
                             let lower_content = self.editor_content.to_lowercase();
                             let lower_from = from.to_lowercase();
                             let mut result = String::new();
                             let mut last_end = 0;
                             for (start, part) in lower_content.match_indices(&lower_from) {
                                 result.push_str(&self.editor_content[last_end..start]);
                                 result.push_str(to);
                                 last_end = start + part.len();
                             }
                             result.push_str(&self.editor_content[last_end..]);
                             self.editor_content = result;
                         } else {
                             // Whole word replace all strategy
                             let mut byte_matches = Vec::new();
                             let mut char_indices = self.editor_content.char_indices().map(|(b, _)| b).collect::<Vec<_>>();
                             char_indices.push(self.editor_content.len());

                             for &(start_char, end_char) in &self.search_matches {
                                 if let (Some(&start_byte), Some(&end_byte)) = (char_indices.get(start_char), char_indices.get(end_char)) {
                                     byte_matches.push((start_byte, end_byte));
                                 }
                             }

                             byte_matches.reverse();
                             for (start, end) in byte_matches {
                                 self.editor_content.replace_range(start..end, to);
                             }
                         }

                         self.is_dirty = true;
                         self.search_matches.clear();
                    }

                    if ui.button("✖").clicked() {
                        self.show_search = false;
                        self.search_matches.clear();
                    }
                });
                ui.separator();
            }

            let mut text = self.editor_content.clone();
            let theme_clone = theme;
            let editor_id = egui::Id::new("main_editor");

            // Smart Indentation (Pre-process)
            if ctx.memory(|m| m.has_focus(editor_id)) && ctx.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.is_none()) {
                ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));

                if let Some(mut state) = egui::TextEdit::load_state(ctx, editor_id) {
                    if let Some(range) = state.cursor.char_range() {
                        let idx = range.primary.index;

                        // Identify indentation of current line
                        let text_before = &self.editor_content[..idx];
                        let line_start = text_before.rfind('\n').map(|i| i + 1).unwrap_or(0);
                        let current_line_prefix = &self.editor_content[line_start..idx];
                        let indent: String = current_line_prefix.chars().take_while(|c| c.is_whitespace()).collect();

                        let mut next_indent = indent.clone();

                        // Check if we should increase indent
                        let trimmed = current_line_prefix.trim_end();
                        if trimmed.ends_with('{') || (trimmed.ends_with('}') && trimmed.contains("\\begin{")) {
                            if !trimmed.contains("\\end{") {
                                next_indent.push_str(INDENT_UNIT);
                            }
                        }

                        let to_insert = format!("\n{}", next_indent);
                        self.editor_content.insert_str(idx, &to_insert);
                        text = self.editor_content.clone(); // Update local text for editor

                        let new_cursor = idx + to_insert.len();
                        state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(new_cursor))));
                        state.store(ctx, editor_id);
                        self.is_dirty = true;
                    }
                }
            }

            // Handle SyncTeX (Ctrl+J)
            if ctx.input(|i| i.key_pressed(egui::Key::J) && i.modifiers.ctrl) {
                if let Some(state) = egui::TextEdit::load_state(ctx, egui::Id::new("main_editor")) {
                    if let Some(range) = state.cursor.char_range() {
                        let idx = range.primary.index;
                        let line_num = self.editor_content[..idx].chars().filter(|&c| c == '\n').count() + 1;

                        self.sync_forward_search(line_num);
                    }
                }
            }

            // Handle Navigation
            if self.show_completions && !self.completion_suggestions.is_empty() {
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
                    if self.completion_selected_index > 0 {
                        self.completion_selected_index -= 1;
                    }
                }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
                    if self.completion_selected_index < self.completion_suggestions.len() - 1 {
                        self.completion_selected_index += 1;
                    }
                }
                if ctx.input(|i| i.key_pressed(egui::Key::Tab) || i.key_pressed(egui::Key::Enter) || (i.key_pressed(egui::Key::Space) && i.modifiers.ctrl)) {
                    ctx.input_mut(|i| {
                        i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                        i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter);
                        i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
                        i.consume_key(egui::Modifiers::CTRL, egui::Key::Space);
                    });
                    if let Some((_, completion)) = self.completion_suggestions.get(self.completion_selected_index).cloned() {
                        self.apply_completion(ctx, &mut text, &completion);
                        self.show_completions = false;
                        self.completion_suggestions.clear();
                        self.editor_content = text.clone();
                    }
                }
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.show_completions = false;
                }
            } else {
                // Snippet Tab Navigation
                if ctx.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.is_none()) {
                    if let Some(mut state) = egui::TextEdit::load_state(ctx, egui::Id::new("main_editor")) {
                        if let Some(range) = state.cursor.char_range() {
                            let cursor_idx = range.primary.index;
                            // Search for next $digit placeholder
                            if let Some(rel_idx) = text[cursor_idx..].find('$') {
                                let target_idx = cursor_idx + rel_idx;
                                if target_idx + 1 < text.len() {
                                    let next_char = text.chars().nth(target_idx + 1).unwrap_or(' ');
                                    if next_char.is_digit(10) {
                                        // Found a placeholder (e.g., $2)
                                        ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab));

                                        // Remove the marker
                                        text.replace_range(target_idx..target_idx+2, "");

                                        // Move cursor to the placeholder position
                                        state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(target_idx))));
                                        state.store(ctx, egui::Id::new("main_editor"));

                                        // Update content immediately
                                        self.editor_content = text.clone();
                                        self.is_dirty = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let gutter_width = 30.0;
            let scroll_target = self.pending_cursor_scroll;
            if self.pending_cursor_scroll.is_some() {
                self.pending_cursor_scroll = None;
            }

            egui::ScrollArea::vertical()
                .id_source("editor_scroll")
                .show(ui, |ui| {
                    let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                        let mut layout_job = self.syntax_highlighting(&theme_clone, string);
                        layout_job.wrap.max_width = wrap_width;
                        ui.fonts(|f| f.layout_job(layout_job))
                    };

                    let output = egui::Frame::none()
                        .inner_margin(egui::Margin { left: gutter_width, ..Default::default() })
                        .show(ui, |ui| {
                            let out = egui::TextEdit::multiline(&mut text)
                                .id(editor_id)
                                .font(TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .code_editor()
                                .lock_focus(true)
                                .layouter(&mut layouter)
                                .show(ui);

                            if let Some(idx) = scroll_target {
                                let ccursor = egui::text::CCursor::new(idx);
                                let cursor = out.galley.from_ccursor(ccursor);
                                let rect = out.galley.pos_from_cursor(&cursor);
                                let target = egui::Rect::from_min_size(out.response.rect.min + rect.min.to_vec2(), egui::Vec2::ZERO);
                                ui.scroll_to_rect(target, Some(egui::Align::Center));
                            }
                            out
                        });

                    let response = output.inner.response;

                    // Context Menu Logic
                    if response.secondary_clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let galley = output.inner.galley.clone();
                            {
                                let rel_pos = pos - response.rect.min;
                                let cursor = galley.cursor_from_pos(rel_pos);
                                let idx = cursor.ccursor.index;

                                let text_chars: Vec<char> = self.editor_content.chars().collect();
                                if idx < text_chars.len() {
                                    // Find word boundaries
                                    let mut start = idx;
                                    while start > 0 && text_chars[start - 1].is_alphanumeric() {
                                        start -= 1;
                                    }
                                    let mut end = idx;
                                    while end < text_chars.len() && text_chars[end].is_alphanumeric() {
                                        end += 1;
                                    }

                                    if start < end {
                                        let word: String = text_chars[start..end].iter().collect();
                                        // Convert char indices to byte indices for range replacement
                                        let start_byte = self.editor_content.chars().take(start).map(|c| c.len_utf8()).sum();
                                        let end_byte = self.editor_content.chars().take(end).map(|c| c.len_utf8()).sum();

                                        self.context_menu_word = Some(word.clone());
                                        self.context_menu_replace_range = Some(start_byte..end_byte);

                                        // Populate spelling suggestions
                                        let lower = word.to_lowercase();
                                        let mut suggestions = Vec::new();
                                        let is_misspelled = !self.dictionary.is_empty()
                                            && !self.dictionary.contains(&lower)
                                            && !self.user_dictionary.contains(&lower)
                                            && !self.ignored_words.contains(&lower);

                                        if is_misspelled {
                                            let target_len = lower.len();
                                            for valid_word in &self.dictionary {
                                                if valid_word.len().abs_diff(target_len) <= 2 {
                                                    let dist = strsim::levenshtein(&lower, valid_word);
                                                    if dist <= 2 {
                                                        suggestions.push((dist, valid_word.clone()));
                                                    }
                                                }
                                            }
                                            suggestions.sort_by_key(|k| k.0);
                                            suggestions.truncate(3);
                                        }
                                        self.context_menu_suggestions = suggestions.into_iter().map(|x| x.1).collect();

                                    } else {
                                        self.context_menu_word = None;
                                        self.context_menu_replace_range = None;
                                    }
                                }
                            }
                        }
                    }

                    // Trigger synonym fetch if needed
                    if let Some(word) = &self.context_menu_word {
                        let lower = word.to_lowercase();
                        if !self.synonym_cache.contains_key(&lower) && !self.pending_synonyms.contains(&lower) {
                             self.pending_synonyms.insert(lower.clone());
                             let tx = self.synonym_tx.clone();
                             let w = lower.clone();
                             std::thread::spawn(move || {
                                  let url = format!("https://api.datamuse.com/words?ml={}&max=5", w);
                                  if let Ok(resp) = reqwest::blocking::get(&url) {
                                      if let Ok(json) = resp.json::<Vec<serde_json::Value>>() {
                                           let syns: Vec<String> = json.iter()
                                               .filter_map(|v| v["word"].as_str().map(|s| s.to_string()))
                                               .collect();
                                           let _ = tx.send((w, syns));
                                      }
                                  }
                              });
                          self.pending_cursor_scroll = None;
                      }
                    }

                    let selected_word = self.context_menu_word.clone();
                    let selected_range = self.context_menu_replace_range.clone();
                    let selected_suggestions = self.context_menu_suggestions.clone();
                    let mut replacement = None;

                    response.context_menu(|ui| {
                        if let Some(word) = &selected_word {
                            ui.label(egui::RichText::new(format!("Selected: \"{}\"", word)).strong());
                            ui.separator();

                            if selected_suggestions.is_empty() {
                                // Only check if it's actually considered misspelled by our logic
                                let lower = word.to_lowercase();
                                let is_misspelled = !self.dictionary.is_empty()
                                    && !self.dictionary.contains(&lower)
                                    && !self.user_dictionary.contains(&lower)
                                    && !self.ignored_words.contains(&lower);

                                if is_misspelled {
                                    ui.label(egui::RichText::new("No spelling suggestions").italics());
                                }
                            } else {
                                for suggestion in &selected_suggestions {
                                    if ui.button(format!("Fix: {}", suggestion)).clicked() {
                                        replacement = Some(suggestion.clone());
                                        ui.close_menu();
                                    }
                                }
                            }

                            ui.separator();
                            if ui.button("Go to PDF").clicked() {
                                if let Some(range) = &selected_range {
                                    let start_byte = range.start;
                                    let line_num = self.editor_content[..start_byte].chars().filter(|&c| c == '\n').count() + 1;

                                    self.sync_forward_search(line_num);
                                }
                                ui.close_menu();
                            }

                            // Dictionary Actions
                            let lower = word.to_lowercase();
                            let is_known = self.dictionary.contains(&lower) || self.user_dictionary.contains(&lower) || self.ignored_words.contains(&lower);

                            if !is_known {
                                ui.separator();
                                if ui.button("➕ Add to Dictionary").clicked() {
                                    self.user_dictionary.insert(lower.clone());
                                    // Append to file
                                    use std::io::Write;
                                    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("user_dictionary.txt") {
                                        let _ = writeln!(file, "{}", lower);
                                    }
                                    ui.close_menu();
                                }
                                if ui.button("🚫 Ignore Word").clicked() {
                                    self.ignored_words.insert(lower.clone());
                                    ui.close_menu();
                                }
                            }

                            ui.separator();
                            let lower = word.to_lowercase();
                            ui.menu_button("Synonyms", |ui| {
                                 if let Some(synonyms) = self.synonym_cache.get(&lower) {
                                     if synonyms.is_empty() {
                                         ui.label("No synonyms found.");
                                     } else {
                                         for syn in synonyms {
                                             if ui.button(syn).clicked() {
                                                 replacement = Some(syn.to_string());
                                                 ui.close_menu();
                                             }
                                         }
                                     }
                                 } else {
                                     ui.spinner();
                                     ui.label("Fetching...");
                                 }
                            });
                        } else {
                             ui.label("No word selected");
                        }
                    });

                    if let Some(text) = replacement {
                        if let Some(range) = selected_range {
                             if range.start < self.editor_content.len() && range.end <= self.editor_content.len() {
                                 self.editor_content.replace_range(range, &text);
                                 self.is_dirty = true;
                             }
                        }
                    }

                    // Paint line numbers
                            {
                                let galley = output.inner.galley.clone();
                                let painter = ui.painter();
                                let min_pos = response.rect.min - egui::vec2(gutter_width, 0.0);

                                // Paint gutter background
                                let gutter_rect = egui::Rect::from_min_max(
                                    min_pos,
                                    egui::pos2(min_pos.x + gutter_width, response.rect.max.y)
                                );
                                painter.rect_filled(gutter_rect, 0.0, theme_clone.bg_tertiary);

                                let mut logical_line = 1;
                                let mut start_new_line = true;

                                for row in &galley.rows {
                                     if start_new_line {
                                         let pos = min_pos + egui::vec2(0.0, row.rect.min.y - galley.rect.min.y);
                                         painter.text(
                                             pos + egui::vec2(gutter_width - 4.0, 0.0),
                                             egui::Align2::RIGHT_TOP,
                                             logical_line.to_string(),
                                             egui::FontId::monospace(10.0),
                                             theme_clone.text_secondary,
                                         );
                                         logical_line += 1;
                                     }
                                     start_new_line = row.ends_with_newline;
                                }
                            }

                            if response.changed() {
                                self.editor_content = text.clone();
                                self.is_dirty = true;
                                self.last_edit_time = ctx.input(|i| i.time);
                                self.checks_dirty = true;
                                self.cached_spell_errors.clear();

                        // Autocomplete Trigger
                        if let Some(state) = egui::TextEdit::load_state(ctx, editor_id) {
                            if let Some(range) = state.cursor.char_range() {
                                let idx = range.primary.index;
                                let text_slice = &self.editor_content[..idx];

                                self.show_completions = false;
                                self.completion_suggestions.clear();

                                // Regex-based trigger for robustness (allows spaces like \ref { )
                                let trigger_regex = regex::Regex::new(r"\\(ref|cite)\s*\{$").unwrap();
                                let env_regex = regex::Regex::new(r"\\begin\{([a-zA-Z]*)$").unwrap();

                                if !text_slice.is_empty() {
                                    let last_few = if text_slice.len() > 30 { &text_slice[text_slice.len()-30..] } else { text_slice };
                                    self.log_debug(&format!("Editor change: cursor={}, last_chars='{}'", idx, last_few));
                                }

                                if let Some(cap) = trigger_regex.captures(text_slice) {
                                    let command = cap.get(1).unwrap().as_str();
                                    self.show_completions = true;
                                    if command == "ref" {
                                        self.completion_suggestions = self.labels.iter().map(|l| (l.clone(), format!("\\ref{{{}}}", l))).collect();
                                    } else {
                                        self.completion_suggestions = self.bib_items.iter().map(|b| (b.clone(), format!("\\cite{{{}}}", b))).collect();
                                    }
                                } else if let Some(cap) = env_regex.captures(text_slice) {
                                    let query = cap.get(1).unwrap().as_str();
                                    self.completion_suggestions = self.latex_environments.iter()
                                         .filter(|env| env.trigger.starts_with(query))
                                         .map(|env| (env.trigger.clone(), env.completion.clone()))
                                         .take(50)
                                         .collect();
                                    self.log_debug(&format!("Env trigger match: query='{}', suggestions={}", query, self.completion_suggestions.len()));
                                    if !self.completion_suggestions.is_empty() {
                                        self.show_completions = true;
                                    }
                                } else if let Some(bs_idx) = text_slice.rfind('\\') {
                                    let after_bs = &text_slice[bs_idx+1..];
                                    // Check if it's a command being typed (no spaces, braces, etc)
                                    if !after_bs.contains(|c: char| c.is_whitespace() || c == '{' || c == '[' || c == '}' || c == '(' || c == ')') && (after_bs.is_empty() || after_bs.chars().all(|c| c.is_alphabetic())) {
                                         let query = after_bs;
                                         self.completion_suggestions = self.latex_commands.iter()
                                             .filter(|cmd| {
                                                 cmd.trigger.starts_with(&format!("\\{}", query)) ||
                                                 (cmd.trigger.starts_with(query) && !query.is_empty())
                                             })
                                             .map(|cmd| (cmd.trigger.clone(), cmd.completion.clone()))
                                             .take(50)
                                             .collect();

                                         if !query.is_empty() {
                                             self.log_debug(&format!("Command trigger match: query='{}', suggestions={}", query, self.completion_suggestions.len()));
                                         }

                                         if !self.completion_suggestions.is_empty() && !query.is_empty() {
                                             self.show_completions = true;
                                         }
                                    }
                                }

                                if self.show_completions {
                                    // Calculate popup position
                                    if let Some(range) = state.cursor.char_range() {
                                        let galley = output.inner.galley.clone();
                                        let cursor = galley.from_ccursor(range.primary);
                                        let cursor_rect = galley.pos_from_cursor(&cursor);
                                        self.completion_popup_pos = response.rect.min + cursor_rect.max.to_vec2() + egui::vec2(0.0, 5.0);
                                    }
                                }
                            }
                        }
                    }





                    // Disable old logic block (preserves file structure if block was larger)
                    if false && response.has_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.is_none()) {
                        ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));

                        if let Some(mut state) = egui::TextEdit::load_state(ctx, editor_id) {
                            if let Some(range) = state.cursor.char_range() {
                                let cursor_idx = range.primary.index;
                                let prefix = &text[..cursor_idx];
                                let last_line_start = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
                                let last_line = &prefix[last_line_start..];

                                // Calculate base indent
                                let mut indent = last_line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                                let trimmed = last_line.trim_end();
                                let content = trimmed.trim_start();

                                // Look ahead to decide if we should dedent
                                let after = &text[cursor_idx..];
                                let after_trimmed = after.trim_start();
                                let next_char = after_trimmed.chars().next();
                                let dedent_next = next_char == Some('}') || next_char == Some(']') || after_trimmed.starts_with("\\end{");

                                if dedent_next && indent.len() >= INDENT_UNIT.len() {
                                    indent.truncate(indent.len() - INDENT_UNIT.len());
                                }

                                let mut to_insert = format!("\n{}", indent);

                                // Extra indent if current line opens a block
                                let is_env_start = content.starts_with("\\begin") && !content.starts_with("\\end");
                                let is_structure = content.starts_with("\\section") || content.starts_with("\\subsection") || content.starts_with("\\chapter");
                                let opens_scope = content.ends_with('{') || content.ends_with('[');

                                let opens_block = is_env_start || is_structure || opens_scope;

                                if opens_block && !dedent_next {
                                    to_insert.push_str(INDENT_UNIT);
                                }

                                text.insert_str(cursor_idx, &to_insert);
                                let new_cursor_idx = cursor_idx + to_insert.len();
                                state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(new_cursor_idx))));
                                state.store(ctx, editor_id);
                            }
                        }
                    }

                    // Auto-close pairs
                    if response.has_focus() {
                        let mut pair_to_insert = None;
                        // Check for typed characters
                        ctx.input(|i| {
                            for event in &i.events {
                                if let egui::Event::Text(s) = event {
                                    match s.as_str() {
                                        "{" => pair_to_insert = Some("}"),
                                        "[" => pair_to_insert = Some("]"),
                                        "(" => pair_to_insert = Some(")"),
                                        "$" => pair_to_insert = Some("$"),
                                        _ => {}
                                    }
                                }
                            }
                        });

                        if let Some(closer) = pair_to_insert {
                             if let Some(state) = egui::TextEdit::load_state(ctx, editor_id) {
                                if let Some(range) = state.cursor.char_range() {
                                    // The TextEdit has already inserted the opener.
                                    // The cursor is now AFTER the opener.
                                    let cursor_idx = range.primary.index;
                                    text.insert_str(cursor_idx, closer);
                                }
                            }
                        }
                    }

                    // Handle Tab key
                    if response.has_focus()
                        && ctx.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.is_none())
                    {
                        ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab));
                        text.insert_str(text.len(), "    ");
                    }

                    // Handle Manual Completion Request (Ctrl+Space)
                    if response.has_focus() && ctx.input(|i| i.key_pressed(egui::Key::Space) && i.modifiers.ctrl) {
                        ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Space));

                        if let Some(state) = egui::TextEdit::load_state(ctx, editor_id) {
                            if let Some(range) = state.cursor.char_range() {
                                let idx = range.primary.index;
                                let text_slice = &text[..idx];

                                // Force completion trigger by scanning backwards for a backslash
                                if let Some(bs_idx) = text_slice.rfind('\\') {
                                    let after_bs = &text_slice[bs_idx+1..];
                                    if after_bs.chars().all(|c| c.is_alphabetic()) {
                                        let query = after_bs;
                                        self.completion_suggestions = self.latex_commands.iter()
                                            .filter(|cmd| {
                                                cmd.trigger.starts_with(&format!("\\{}", query)) ||
                                                (cmd.trigger.starts_with(query) && !query.is_empty())
                                            })
                                            .map(|cmd| (cmd.trigger.clone(), cmd.completion.clone()))
                                            .take(50)
                                            .collect();

                                        if !self.completion_suggestions.is_empty() {
                                            self.show_completions = true;
                                            self.completion_selected_index = 0;

                                            let galley = output.inner.galley.clone();
                                            let cursor = galley.from_ccursor(range.primary);
                                            let cursor_rect = galley.pos_from_cursor(&cursor);
                                            self.completion_popup_pos = response.rect.min + cursor_rect.max.to_vec2() + egui::vec2(0.0, 5.0);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Sync text changes back to editor_content
                    if response.changed() {
                        self.editor_content = text;
                        self.is_dirty = true;
                        self.update_outline();
                        ctx.request_repaint();
                    } else {
                        self.editor_content = text;
                    }
                });
        });

            // Show autocomplete dropdown popup
            if self.show_completions && !self.completion_suggestions.is_empty() {
                let popup_id = egui::Id::new("autocomplete_popup");

                let _area_resp = egui::Area::new(popup_id)
                    .fixed_pos(self.completion_popup_pos)
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        let frame_resp = Frame::popup(&ctx.style())
                            .show(ui, |ui| {
                                // Measure text width precisely
                                let font_id = egui::TextStyle::Monospace.resolve(ui.style());
                                let mut max_text_width: f32 = 0.0;
                                ui.fonts(|f| {
                                    for (trigger, _) in &self.completion_suggestions {
                                        // Layout the text to get exact width
                                        let job = egui::text::LayoutJob::simple_singleline(
                                            trigger.clone(),
                                            font_id.clone(),
                                            egui::Color32::TEMPORARY_COLOR
                                        );
                                        let galley = f.layout_job(job);
                                        max_text_width = max_text_width.max(galley.rect.width());
                                    }
                                });

                                // Add padding for button frames and scrollbar
                                let desired_width = (max_text_width + 45.0).max(180.0);

                                ui.set_min_width(desired_width);
                                ui.set_max_width(desired_width); // Enforce strict width
                                ui.set_max_height(300.0);

                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new("Ctrl+Space to select")
                                            .small()
                                            .italics()
                                            .color(ui.visuals().weak_text_color()),
                                    );
                                });
                                ui.separator();

                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    for (i, (trigger, completion)) in self.completion_suggestions.clone().iter().enumerate() {
                                        let is_selected = i == self.completion_selected_index;

                                        // Display trigger
                                        let mut text = egui::RichText::new(trigger).monospace();
                                        if is_selected {
                                            text = text.color(theme.accent).strong();
                                        }

                                        let btn = ui.button(text);
                                        if is_selected {
                                            btn.scroll_to_me(Some(egui::Align::Center));
                                        }

                                        if btn.clicked() {
                                            let mut text = self.editor_content.clone();
                                            self.apply_completion(ctx, &mut text, &completion);
                                            self.editor_content = text;
                                            self.show_completions = false;
                                            self.completion_suggestions.clear();
                                            self.completion_selected_index = 0;
                                        }
                                    }
                                });
                            });
                        self.completion_popup_rect = Some(frame_resp.response.rect);
                    });

                // Handle keyboard navigation and selection
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    if self.completion_selected_index > 0 {
                        self.completion_selected_index -= 1;
                    }
                }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    if self.completion_selected_index < self.completion_suggestions.len() - 1 {
                        self.completion_selected_index += 1;
                    }
                }
                if ctx.input(|i| (i.key_pressed(egui::Key::Space) && i.modifiers.ctrl) || i.key_pressed(egui::Key::Tab)) {
                    if self.completion_selected_index < self.completion_suggestions.len() {
                        let (_, completion) = self.completion_suggestions[self.completion_selected_index].clone();
                        let mut text = self.editor_content.clone();
                        self.apply_completion(ctx, &mut text, &completion);
                        self.editor_content = text;
                        self.show_completions = false;
                        self.completion_suggestions.clear();
                        self.completion_selected_index = 0;
                    }
                }

                // Close popup on Escape or click outside
                let close_on_click = if let Some(rect) = self.completion_popup_rect {
                    ctx.input(|i| i.pointer.any_click() && !rect.contains(i.pointer.hover_pos().unwrap_or_default()))
                } else {
                    false
                };

                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) || close_on_click {
                    self.show_completions = false;
                    self.completion_suggestions.clear();
                    self.completion_popup_rect = None;
                }
            }

        // Command Palette Overlay
        if self.show_command_palette {
            let modal_id = egui::Id::new("command_palette_modal");
            egui::Area::new(modal_id)
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 100.0))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let mut frame = Frame::popup(&ctx.style());
                    frame.fill = theme.bg_secondary;
                    frame.stroke = Stroke::new(1.0, theme.accent);
                    frame.show(ui, |ui| {
                        ui.set_width(400.0);
                        ui.vertical(|ui| {
                            let text_res = ui.text_edit_singleline(&mut self.cmd_query);
                            text_res.request_focus();

                            // Navigation
                            if text_res.has_focus() {
                                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                                    if self.cmd_selected_index > 0 { self.cmd_selected_index -= 1; }
                                }
                                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                                    self.cmd_selected_index += 1;
                                }
                            }

                            ui.separator();

                            let commands = vec![
                                ("Compile Project", "Build the current project"),
                                ("Save File", "Save current changes"),
                                ("Open File", "Open a file..."),
                                ("Open Folder", "Open a folder..."),
                                ("Toggle Sidebar", "Show/Hide file panel"),
                            ];

                            let filtered: Vec<_> = commands.into_iter()
                                .filter(|(name, _)| name.to_lowercase().contains(&self.cmd_query.to_lowercase()))
                                .collect();

                            if self.cmd_selected_index >= filtered.len() && !filtered.is_empty() {
                                self.cmd_selected_index = filtered.len() - 1;
                            }

                            // Enter to select
                            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

                            for (i, (name, desc)) in filtered.iter().enumerate() {
                                let is_selected = i == self.cmd_selected_index;
                                let label = if is_selected {
                                    egui::RichText::new(*name).color(theme.accent).strong()
                                } else {
                                    egui::RichText::new(*name).strong()
                                };

                                let btn = ui.button(label);
                                if is_selected {
                                    btn.scroll_to_me(Some(egui::Align::Center));
                                }

                                if btn.clicked() || (is_selected && enter_pressed) {
                                    match *name {
                                        "Compile Project" => if !self.is_compiling { self.compile(ctx); },
                                        "Save File" => self.save_file(ctx, true),
                                        "Open File" => {
                                            if let Some(path) = rfd::FileDialog::new().add_filter("LaTeX", &["tex"]).pick_file() {
                                                self.file_path = path.to_string_lossy().to_string();
                                                let p = self.file_path.clone();
                                                self.load_file(ctx, &p);
                                                self.update_outline();
                                            }
                                        },
                                        "Open Folder" => {
                                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                                self.current_dir = folder;
                                            }
                                        },
                                        "Toggle Sidebar" => self.show_file_panel = !self.show_file_panel,
                                        _ => {}
                                    }
                                    self.show_command_palette = false;
                                }
                                ui.label(egui::RichText::new(*desc).small().color(theme.text_secondary));
                                ui.separator();
                            }
                        });
                    });
                });

            // Close on Escape
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_command_palette = false;
            }
        }
    }
}

// Helper Functions

fn ensure_fontconfig() {
    let fonts_conf_path = if std::path::Path::new("fonts.conf").exists() {
        "fonts.conf"
    } else if std::path::Path::new("deps/fonts.conf").exists() {
        "deps/fonts.conf"
    } else {
        // Fallback: try to create in deps if it exists, otherwise root
        if std::path::Path::new("deps").exists() {
            "deps/fonts.conf"
        } else {
            "fonts.conf"
        }
    };

    if cfg!(windows) {
        if !std::path::Path::new(fonts_conf_path).exists() {
            let fonts_conf_content = r#"<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "fonts.dtd">
<fontconfig>
  <dir>C:/Windows/Fonts</dir>
  <cachedir>~/.fontconfig</cachedir>
</fontconfig>
"#;
            let _ = std::fs::write(fonts_conf_path, fonts_conf_content);
        }

        let abs_path = if std::path::Path::new(fonts_conf_path).is_absolute() {
            std::path::PathBuf::from(fonts_conf_path)
        } else {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")).join(fonts_conf_path)
        };
        std::env::set_var("FONTCONFIG_FILE", &abs_path);
        let config_dir = abs_path.parent().unwrap_or(std::path::Path::new("."));
        std::env::set_var("FONTCONFIG_PATH", config_dir);
    }
}



fn render_pdf_page_to_texture(
    ctx: &egui::Context,
    pdfium: &Pdfium,
    pdf_path: &std::path::PathBuf,
    page_index: usize,
    zoom: f32,
    preview_size: &mut Option<[usize; 2]>,
    page_sizes: &mut std::collections::HashMap<usize, (f32, f32)>,
) -> Result<egui::TextureHandle, Box<dyn std::error::Error>> {
    let doc = pdfium.load_pdf_from_file(pdf_path, None)?;

    let page_index_u16: u16 = page_index.try_into()?;
    let page = doc.pages().get(page_index_u16)?;

    let width_pt = page.width().value;
    let height_pt = page.height().value;
    page_sizes.insert(page_index, (width_pt, height_pt));

    let scale = ctx.pixels_per_point().max(1.0);
    // Increase resolution: render at 3x scale relative to standard 72 DPI pt size
    let width_pixels = width_pt * zoom * scale * 3.0;
    let target_width = width_pixels.clamp(100.0, 8192.0).round() as i32;

    let render_config = PdfRenderConfig::new()
        .set_target_width(target_width)
        .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

    let bitmap = page.render_with_config(&render_config)?;
    let dyn_img = bitmap.as_image().into_rgba8();

    let size = [dyn_img.width() as usize, dyn_img.height() as usize];
    let pixels = dyn_img.into_raw();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);

    *preview_size = Some(size);

    Ok(ctx.load_texture(
        "pdf_preview",
        color_image,
        egui::TextureOptions {
            magnification: egui::TextureFilter::Linear,
            minification: egui::TextureFilter::Linear,
            wrap_mode: egui::TextureWrapMode::ClampToEdge,
        },
    ))
}

fn load_icon() -> Option<egui::IconData> {
    let png_path = std::path::Path::new("icon.png");
    let deps_path = std::path::Path::new("deps/icon.png");
    let root_deps_path = std::path::Path::new("../../deps/icon.png");
    let root_path = std::path::Path::new("../../icon.png");

    let icon_path = if png_path.exists() {
        std::path::Path::new("icon.png")
    } else if deps_path.exists() {
        deps_path
    } else if root_deps_path.exists() {
        root_deps_path
    } else if root_path.exists() {
        root_path
    } else {
        return None;
    };

    if let Ok(image) = image::open(icon_path) {
        let image = image.to_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        Some(egui::IconData {
            rgba,
            width,
            height,
        })
    } else {
        None
    }
}

fn main() -> Result<(), eframe::Error> {
    ensure_fontconfig();

    let mut viewport = egui::ViewportBuilder::default()
        .with_min_inner_size([800.0, 600.0])
        .with_maximized(true);

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Typesafe",
        options,
        Box::new(|_cc| Box::<TypesafeApp>::default()),
    )
}
