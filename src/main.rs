#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crossbeam_channel::{unbounded, Receiver, Sender};
use image;
use egui::{
    Align, Color32, FontId, Frame, Layout, Rounding, Stroke, TextStyle, Vec2, Visuals,
};
use egui::epaint::Shadow;
use egui::text::{CCursor, CCursorRange};
use eframe::egui;
use pdfium_render::prelude::*;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Style};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

const INDENT_UNIT: &str = "    ";

#[cfg(windows)]
use std::os::windows::process::CommandExt;

// Themes

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, Debug)]
struct Diagnostic {
    message: String,
    line: usize,
    file: String,
}

enum CompilationMsg {
    Start,
    Log(String),
    Diagnostics(Vec<Diagnostic>),
    Success(PathBuf),
    Error(String),
}

// Settings and App State

#[derive(Clone, PartialEq)]
enum SettingsTab {
    Appearance,
    Permissions,
    APIs,
}

#[derive(Clone)]
struct Settings {
    pub theme: ThemePreset,
    pub accent: AccentColor,
    pub show_settings_window: bool,
    pub active_tab: SettingsTab,
    pub auto_compile: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemePreset::GruvboxDark,
            accent: AccentColor::Orange,
            show_settings_window: false,
            active_tab: SettingsTab::Appearance,
            auto_compile: true,
        }
    }
}

#[derive(Clone)]
struct OutlineItem {
    label: String,
    line: usize,
    level: usize,
}

struct TypesafeApp {
    // Editor
    editor_content: String,
    file_path: String,
    current_dir: std::path::PathBuf,
    root_file: Option<String>,
    show_file_panel: bool,
    outline_items: Vec<OutlineItem>,
    labels: Vec<String>,
    bib_items: Vec<String>,
    context_menu_word: Option<String>,
    context_menu_suggestions: Vec<String>,
    context_menu_replace_range: Option<std::ops::Range<usize>>,
    dictionary: std::collections::HashSet<String>,
    synonym_cache: std::collections::HashMap<String, Vec<String>>,
    pending_synonyms: std::collections::HashSet<String>,
    synonym_rx: Receiver<(String, Vec<String>)>,
    synonym_tx: Sender<(String, Vec<String>)>,
    is_dirty: bool,

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
    #[allow(dead_code)]
    completion_suggestions: Vec<(String, String)>,
    #[allow(dead_code)]
    show_completions: bool,
    #[allow(dead_code)]
    completion_popup_pos: egui::Pos2,
    #[allow(dead_code)]
    completion_popup_rect: Option<egui::Rect>,
    #[allow(dead_code)]
    completion_selected_index: usize,

    // Search and Replace
    show_search: bool,
    search_query: String,
    replace_query: String,
    search_case_sensitive: bool,
    search_whole_word: bool,
    search_matches: Vec<(usize, usize)>,
    search_match_index: usize,
}

impl Default for TypesafeApp {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        let (syn_tx, syn_rx) = unbounded();

        // Load syntax highlighting data
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        let current_dir = if std::path::Path::new("examples").exists() {
            std::path::PathBuf::from("examples")
        } else {
            std::path::PathBuf::from(".")
        };

        // Try to find a default tex file
        let mut default_file = "test.tex".to_string();
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "tex") {
                    default_file = path.to_string_lossy().to_string();
                    break;
                }
            }
        }

        let default_content = std::fs::read_to_string(&default_file).unwrap_or_else(|_| {
            "\\documentclass{article}\n\\begin{document}\nHello Typesafe!\n\\end{document}".to_string()
        });

        // Initialize Dictionary
        let mut dictionary = std::collections::HashSet::new();
        let dict_path = std::path::Path::new("dictionary.txt");
        if dict_path.exists() {
            if let Ok(content) = std::fs::read_to_string(dict_path) {
                for line in content.lines() {
                    dictionary.insert(line.trim().to_lowercase());
                }
            }
        } else {
             // Attempt download in a separate thread
             std::thread::spawn(|| {
                 if let Ok(resp) = reqwest::blocking::get("https://raw.githubusercontent.com/dwyl/english-words/master/words_alpha.txt") {
                     if let Ok(text) = resp.text() {
                         let _ = std::fs::write("dictionary.txt", &text);
                     }
                 }
             });
        }

        Self {
            editor_content: default_content,
            file_path: default_file,
            current_dir,
            root_file: None,
            show_file_panel: true,
            outline_items: Vec::new(),
            labels: Vec::new(),
            bib_items: Vec::new(),
            context_menu_word: None,
            context_menu_suggestions: Vec::new(),
            context_menu_replace_range: None,
            dictionary,
            synonym_cache: std::collections::HashMap::new(),
            pending_synonyms: std::collections::HashSet::new(),
            synonym_rx: syn_rx,
            synonym_tx: syn_tx,
            is_dirty: false,
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
                .or_else(|_| Pdfium::bind_to_system_library())
                .map(Pdfium::new)
                .ok(),
            page_count: 0,
            current_page: 0,
            zoom: 1.0,
            fit_mode: PdfFitMode::Normal,
            magnifier_enabled: false,
            magnifier_zoom: 2.0,
            magnifier_size: 180.0,
            compilation_log: String::new(),
            show_log: false,
            is_compiling: false,
            compile_rx: rx,
            compile_tx: tx,
            pending_autocompile: true,
            diagnostics: Vec::new(),
            settings: Settings::default(),
            completion_suggestions: Vec::new(),
            show_completions: false,
            completion_popup_pos: egui::Pos2::ZERO,
            completion_popup_rect: None,
            completion_selected_index: 0,

            show_search: false,
            search_query: String::new(),
            replace_query: String::new(),
            search_case_sensitive: false,
            search_whole_word: false,
            search_matches: Vec::new(),
            search_match_index: 0,
        }
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
        visuals.window_rounding = Rounding::same(10.0);
        visuals.window_shadow = Shadow {
            offset: Vec2::new(0.0, 4.0),
            blur: 12.0,
            spread: 0.0,
            color: Color32::from_black_alpha(60),
        };

        // Text styles

        // Widget backgrounds
        visuals.widgets.inactive.bg_fill = theme.bg_tertiary;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, theme.border);

        visuals.widgets.hovered.bg_fill = theme.bg_tertiary.linear_multiply(1.2);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, theme.accent);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, theme.accent);

        visuals.widgets.active.bg_fill = theme.accent.linear_multiply(0.3);
        visuals.widgets.active.bg_stroke = Stroke::new(1.5, theme.accent);
        visuals.widgets.active.fg_stroke = Stroke::new(2.0, theme.accent);

        visuals.widgets.open.bg_fill = theme.accent.linear_multiply(0.2);
        visuals.widgets.open.bg_stroke = Stroke::new(1.5, theme.accent);

        // Selection
        visuals.selection.bg_fill = theme.accent.linear_multiply(0.4);
        visuals.selection.stroke = Stroke::new(1.0, theme.accent);

        // Button styling
        visuals.widgets.inactive.weak_bg_fill = theme.bg_tertiary;

        // Separator
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, theme.border);

        // Interactive elements
        visuals.widgets.noninteractive.bg_fill = theme.bg;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, theme.border);

        ctx.set_visuals(visuals);
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
        ) {
            Ok(texture) => {
                self.pdf_textures.insert(page_idx, texture);
            }
            Err(e) => {
                self.preview_status = format!("Render error on page {}: {}", page_idx, e);
            }
        }
    }

    fn load_file(&mut self, path: &str) {
        match std::fs::read_to_string(path) {
            Ok(contents) => {
                self.editor_content = contents;
                self.is_dirty = false;
                self.compile();
            }
            Err(e) => {
                self.editor_content = format!("Error loading file: {}", e);
            }
        }
    }

    fn save_file(&mut self) {
        match std::fs::write(&self.file_path, &self.editor_content) {
            Ok(_) => {
                self.is_dirty = false;
                self.compilation_log = "File saved successfully\n".to_string();
                self.update_outline();
                if self.settings.auto_compile {
                    self.compile();
                }
            }
            Err(e) => {
                self.compilation_log = format!("Error saving file: {}\n", e);
            }
        }
    }

    fn save_file_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new().add_filter("LaTeX", &["tex"]).save_file() {
            self.file_path = path.to_string_lossy().to_string();
            self.save_file();
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

    fn compile(&mut self) {
        let tx = self.compile_tx.clone();
        let content = self.editor_content.clone();
        let file_path = self.file_path.clone();
        let root_file = self.root_file.clone();

        std::thread::spawn(move || {
            let _ = tx.send(CompilationMsg::Start);

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
            if let Err(e) = std::fs::write(&save_path, &content) {
                let _ = tx.send(CompilationMsg::Error(format!("Write error: {}", e)));
                return;
            }

            let tectonic = locate_tectonic();
            let mut cmd = Command::new(&tectonic);
            // Determine output dir
            let parent_dir = std::path::Path::new(&target_path).parent().unwrap_or(std::path::Path::new("."));

            cmd.arg("-X")
                .arg("compile")
                .arg(&target_path)
                .arg("-o")
                .arg(parent_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                cmd.creation_flags(CREATE_NO_WINDOW);
            }

            let output = cmd.output();

            if let Ok(out) = &output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let mut diagnostics = Vec::new();
                // Regex to find "l.123 context" pattern commonly output by TeX engines
                let line_regex = regex::Regex::new(r"l\.(\d+)\s+(.*)").unwrap();

                for line in stdout.lines().chain(stderr.lines()) {
                    if let Some(caps) = line_regex.captures(line) {
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
                }
            }

            match output {
                Ok(output) if output.status.success() => {
                    let stem = std::path::Path::new(&target_path).file_stem().unwrap_or_default();
                    let pdf_name = format!("{}.pdf", stem.to_string_lossy());
                    let pdf_path = parent_dir.join(&pdf_name);

                    if pdf_path.exists() {
                        let _ = tx.send(CompilationMsg::Success(pdf_path));
                    } else {
                        let _ = tx.send(CompilationMsg::Error(
                            "PDF file not found after compilation".to_string(),
                        ));
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
                }
                Err(e) => {
                    let _ = tx.send(CompilationMsg::Error(format!("Failed to run tectonic: {}", e)));
                }
            }
        });

        self.is_compiling = true;
    }

    fn update_outline(&mut self) {
        let mut items = Vec::new();
        let mut labels = Vec::new();

        // Regex for sections
        let re_section = regex::Regex::new(r"\\(part|chapter|section|subsection|subsubsection)\*?\{([^}]+)\}").unwrap();
        // Regex for labels
        let re_label = regex::Regex::new(r"\\label\{([^}]+)\}").unwrap();

        for (line_idx, line) in self.editor_content.lines().enumerate() {
            if let Some(caps) = re_section.captures(line) {
                let type_str = caps.get(1).map_or("", |m| m.as_str());
                let title = caps.get(2).map_or("", |m| m.as_str());

                let level = match type_str {
                    "part" => 0,
                    "chapter" => 0,
                    "section" => 1,
                    "subsection" => 2,
                    "subsubsection" => 3,
                    _ => 1,
                };

                items.push(OutlineItem {
                    label: title.to_string(),
                    line: line_idx,
                    level,
                });
            }

            if let Some(caps) = re_label.captures(line) {
                if let Some(label) = caps.get(1) {
                    labels.push(label.as_str().to_string());
                }
            }
        }
        self.outline_items = items;
        self.labels = labels;

        // Scan bibliography
        let mut bib_items = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "bib") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let re_bib = regex::Regex::new(r"@\w+\{([^,]+),").unwrap();
                        for cap in re_bib.captures_iter(&content) {
                            if let Some(key) = cap.get(1) {
                                bib_items.push(key.as_str().to_string());
                            }
                        }
                    }
                }
            }
        }
        self.bib_items = bib_items;
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

        let mut start_idx = 0;
        for (i, c) in text.char_indices() {
             if !c.is_alphabetic() {
                 if i > start_idx {
                     let word = &text[start_idx..i];
                     let lower = word.to_lowercase();
                     // Filter out LaTeX commands and check dictionary
                     if !word.starts_with('\\') && !self.dictionary.contains(&lower) {
                         let is_command = start_idx > 0 && text[..start_idx].ends_with('\\');
                         if !is_command {
                            errors.push(start_idx..i);
                         }
                     }
                 }
                 start_idx = i + 1;
             }
        }
        // Last word
        if start_idx < text.len() {
             let word = &text[start_idx..];
             let lower = word.to_lowercase();
             if !word.starts_with('\\') && !self.dictionary.contains(&lower) {
                  let is_command = start_idx > 0 && text[..start_idx].ends_with('\\');
                  if !is_command {
                      errors.push(start_idx..text.len());
                  }
             }
        }

        errors
    }

    fn insert_command(&mut self, ctx: &egui::Context, command: &str) {
        if let Some(mut state) = egui::TextEdit::load_state(ctx, egui::Id::new("main_editor")) {
            if let Some(range) = state.cursor.char_range() {
                let cursor = range.primary.index;
                self.editor_content.insert_str(cursor, command);

                // Calculate new cursor position
                let mut new_cursor_idx = cursor + command.len();

                // If command contains braces {}, place cursor inside them
                if let Some(offset) = command.find("{}") {
                    new_cursor_idx = cursor + offset + 1;
                } else if let Some(offset) = command.find("}{") { // for \frac
                    new_cursor_idx = cursor + offset + 1;
                } else if let Some(offset) = command.find("$$") { // for inline math
                    new_cursor_idx = cursor + offset + 1;
                } else if let Some(offset) = command.find("[]") {
                    new_cursor_idx = cursor + offset + 1;
                }

                state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(new_cursor_idx))));
                state.store(ctx, egui::Id::new("main_editor"));
            } else {
                self.editor_content.push_str(command);
            }
        } else {
            self.editor_content.push_str(command);
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
        ctx.request_repaint();
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

        // Run syntax check
        let errors = self.check_syntax(text);

        // Run spell check
        let spell_errors = self.check_spelling(text);

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

        for line in LinesWithEndings::from(text) {
            let ranges: Vec<(Style, &str)> = highlighter
                .highlight_line(line, &self.syntax_set)
                .unwrap_or_default();

            for (style, range_text) in ranges {
                let range_len = range_text.len();
                let range_start = current_byte_idx;
                let range_end = range_start + range_len;

                let fg = style.foreground;
                let text_color = Color32::from_rgb(fg.r, fg.g, fg.b);
                let font_id = TextStyle::Monospace.resolve(&egui::Style::default());

                // Split tokens to precisely highlight errors and search matches
                let mut split_points = vec![0, range_len];

                for e in &errors {
                    if e.start > range_start && e.start < range_end {
                        split_points.push(e.start - range_start);
                    }
                    if e.end > range_start && e.end < range_end {
                        split_points.push(e.end - range_start);
                    }
                }

                for e in &spell_errors {
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
            self.compile();
        }



        // Apply theme
        let mut theme = ThemeColors::from_preset(self.settings.theme);
        theme.accent = self.settings.accent.color();
        Self::apply_theme(ctx, theme);

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
                CompilationMsg::Success(pdf_path) => {
                    self.is_compiling = false;
                    self.preview_status =
                        format!("✓ Compilation success\nOutput: {}", pdf_path.display());
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

        if self.show_log {
            egui::Window::new("Compilation Log & Diagnostics")
                .open(&mut self.show_log)
                .resizable(true)
                .default_height(200.0)
                .show(ctx, |ui| {
                    if !self.diagnostics.is_empty() {
                         ui.label(egui::RichText::new("Diagnostics:").strong().color(egui::Color32::from_rgb(255, 100, 100)));
                         egui::ScrollArea::vertical().max_height(100.0).id_source("diag_scroll").show(ui, |ui| {
                             for diag in &self.diagnostics {
                                 if ui.link(format!("Line {}: {}", diag.line, diag.message)).clicked() {
                                     // Calculate char index for line
                                     let char_idx = self.editor_content.lines().take(diag.line.saturating_sub(1)).map(|l| l.len() + 1).sum::<usize>();

                                     if let Some(mut state) = egui::TextEdit::load_state(ctx, egui::Id::new("main_editor")) {
                                          state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(char_idx))));
                                          state.store(ctx, egui::Id::new("main_editor"));
                                     }
                                 }
                             }
                         });
                         ui.separator();
                    }

                    ui.label("Raw Log:");
                    egui::ScrollArea::vertical().id_source("log_scroll").show(ui, |ui| {
                        ui.add(egui::TextEdit::multiline(&mut self.compilation_log.as_str()).code_editor());
                    });
                });
        }

        // Handle keyboard shortcuts
        if ctx.input(|i| {
            i.key_pressed(egui::Key::S) && (i.modifiers.ctrl || i.modifiers.command)
        }) {
            self.save_file();
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
            if !self.is_compiling {
                self.compile();
            }
        }

        // Search Toggle
        if ctx.input(|i| i.key_pressed(egui::Key::F) && (i.modifiers.ctrl || i.modifiers.command)) {
            self.show_search = !self.show_search;
            if self.show_search {
                // Focus handled by UI render
            } else {
                self.search_matches.clear();
            }
        }

        // Command Palette Toggle
        if ctx.input(|i| i.key_pressed(egui::Key::P) && (i.modifiers.ctrl && i.modifiers.shift)) {
            self.show_command_palette = !self.show_command_palette;
            self.cmd_query.clear();
        }

        // ====== TOP PANEL ======
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                // File Menu
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.editor_content = "\\documentclass{article}\n\\begin{document}\n\n\\end{document}".to_string();
                        self.file_path = "untitled.tex".to_string();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Open File...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().add_filter("LaTeX", &["tex"]).pick_file() {
                            self.file_path = path.to_string_lossy().to_string();
                            let path_str = self.file_path.clone();
                            self.load_file(&path_str);
                            if let Some(parent) = path.parent() {
                                self.current_dir = parent.to_path_buf();
                            }
                            ui.close_menu();
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
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        self.save_file_as();
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
                    if ui.checkbox(&mut self.show_log, "Compilation Log").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.add(egui::Button::new("Command Palette").shortcut_text("Ctrl+Shift+P")).clicked() {
                        self.show_command_palette = !self.show_command_palette;
                         ui.close_menu();
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
                         self.compile();
                         ui.close_menu();
                     }
                });

                 // Help Menu
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        // Placeholder
                        ui.close_menu();
                    }
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.heading(
                        egui::RichText::new("Typesafe")
                            .color(theme.text_primary)
                            .font(FontId::proportional(22.0)),
                    );
                    ui.label(
                        egui::RichText::new(format!(" | {}", self.file_path))
                            .color(theme.text_secondary)
                            .font(FontId::proportional(16.0)),
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

        if self.settings.show_settings_window {
            egui::Window::new("Settings")
                .open(&mut self.settings.show_settings_window)
                .resizable(true)
                .default_width(400.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.settings.active_tab, SettingsTab::Appearance, "Appearance");
                        ui.selectable_value(&mut self.settings.active_tab, SettingsTab::Permissions, "Permissions");
                        ui.selectable_value(&mut self.settings.active_tab, SettingsTab::APIs, "APIs");
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
                                        ui.selectable_value(&mut self.settings.theme, preset, preset.name());
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
                                        ui.selectable_value(&mut self.settings.accent, accent, accent.name());
                                    }
                                });
                        },
                        SettingsTab::Permissions => {
                            ui.label("Permission settings will go here.");
                        },
                        SettingsTab::APIs => {
                            ui.heading("Tectonic Engine");
                            ui.add_space(4.0);
                            ui.checkbox(&mut self.settings.auto_compile, "Auto-compile on Save");
                            ui.add_space(8.0);
                            if ui.button("🗑 Clear Package Cache").clicked() {
                                // Placeholder for clearing cache logic
                                self.compilation_log.push_str("To clear cache manually, delete the tectonic cache directory in your user folder.\n");
                                self.show_log = true;
                            }
                        }
                    }
                });
        }

        // ====== LEFT PANEL (FILES) ======
        if self.show_file_panel {
            egui::SidePanel::left("files_panel")
                .resizable(true)
                .default_width(200.0)
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("📂 FILES")
                            .color(theme.text_secondary)
                            .strong(),
                    );
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
                                        if !is_dir && ui.button("Set as Root File").clicked() {
                                            self.root_file = Some(path.to_string_lossy().to_string());
                                            ui.close_menu();
                                        }
                                    });

                                    if btn.clicked() {
                                        if is_dir {
                                            self.current_dir = path;
                                        } else if is_allowed {
                                            if ["tex", "bib", "cls", "sty", "md", "txt"].contains(&ext.as_str()) {
                                                self.file_path = path.to_string_lossy().to_string();
                                                let path_str = self.file_path.clone();
                                                self.load_file(&path_str);
                                                self.update_outline();
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
                            for item in &self.outline_items {
                                ui.horizontal(|ui| {
                                    ui.add_space(item.level as f32 * 10.0);
                                    if ui.button(egui::RichText::new(&item.label)).clicked() {
                                        if let Some(mut state) = egui::TextEdit::load_state(ctx, egui::Id::new("main_editor")) {
                                            let char_idx = self.editor_content.lines().take(item.line).map(|l| l.len() + 1).sum::<usize>();
                                            state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(char_idx))));
                                            state.store(ctx, egui::Id::new("main_editor"));
                                        }
                                    }
                                });
                            }
                            if self.outline_items.is_empty() {
                                ui.label(egui::RichText::new("No sections.").small().italics());
                            }
                        });
                    });
                });
        }

        // ====== RIGHT PANEL (PREVIEW) ======
        egui::SidePanel::right("preview_panel")
            .resizable(true)
            .default_width(600.0)
            .min_width(300.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("📄 PREVIEW")
                        .color(theme.text_secondary)
                        .strong(),
                );
                ui.separator();



                // Preview Controls
                ui.horizontal(|ui| {
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
                    ui.add(egui::Slider::new(&mut zoom, 0.5..=3.0).step_by(0.05).show_value(false));

                    let mut zoom_percent = (zoom * 100.0).round() as u32;
                    if ui.add(egui::DragValue::new(&mut zoom_percent).suffix("%").clamp_range(10..=500).speed(1.0)).changed() {
                        zoom = zoom_percent as f32 / 100.0;
                    }

                    if (zoom - self.zoom).abs() > 0.001 {
                        self.zoom = zoom;
                        self.pdf_textures.clear();
                    }

                    if ui.button("⟳").on_hover_text("Reset Zoom").clicked() {
                        self.zoom = 1.0;
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

                    ui.separator();

                    ui.toggle_value(&mut self.magnifier_enabled, "🔍").on_hover_text("Magnifier");
                });

                // Ctrl+Scroll zoom
                let scroll_zoom = ctx.input(|i| if i.modifiers.ctrl { i.raw_scroll_delta.y } else { 0.0 });
                if scroll_zoom.abs() > 0.0 {
                    self.zoom = (self.zoom + scroll_zoom * 0.0015).clamp(0.3, 4.0);
                    self.pdf_textures.clear();
                }

                // PDF Preview Area
                let available_size = ui.available_size();
                let scroll_height = available_size.y - if self.show_log { 160.0 } else { 30.0 };

                egui::ScrollArea::both()
                    .auto_shrink([false; 2])
                    .max_height(scroll_height)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            if self.page_count > 0 {
                                // Lazily render all pages
                                for page_idx in 0..self.page_count {
                                 self.render_page(ctx, page_idx);

                                 if let Some(tex) = self.pdf_textures.get(&page_idx) {
                                     if let Some([w, h]) = self.preview_size {
                                        let aspect = w as f32 / h as f32;
                                        let available_w = (ui.available_width() - 20.0).max(100.0);
                                        let mut display_width = available_w * self.zoom;
                                        if self.fit_mode == PdfFitMode::FitWidth {
                                            display_width = available_w;
                                        } else if self.fit_mode == PdfFitMode::FitPage {
                                            let by_height = scroll_height.max(50.0) * aspect;
                                            display_width = by_height.min(available_w);
                                        }
                                        let display_height = display_width / aspect;

                                        let img_resp = ui.add(egui::Image::new((
                                            tex.id(),
                                            Vec2::new(display_width, display_height),
                                        )).sense(egui::Sense::click()));


                                        if self.page_count > 0 {
                                            if img_resp.double_clicked() {
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

                // Compilation Log
                ui.separator();
                ui.horizontal(|ui| {
                    let icon = if self.show_log { "▼" } else { "▶" };
                    if ui.button(format!("{} Log", icon)).clicked() {
                        self.show_log = !self.show_log;
                    }
                    if !self.compilation_log.is_empty() {
                         ui.label(egui::RichText::new(if self.is_compiling { "Compiling..." } else { "Status:" }).small());
                    }
                });

                if self.show_log {
                    egui::ScrollArea::vertical()
                        .id_source("log_scroll")
                        .max_height(160.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.compilation_log)
                                    .font(TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .code_editor(),
                            );
                        });
                }
            });

        // ====== CENTRAL PANEL (EDITOR) ======
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("✏ EDITOR")
                        .color(theme.text_secondary)
                        .strong(),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("⚡ Compile (Ctrl+B)").clicked() && !self.is_compiling {
                        self.compile();
                    }
                    if ui.button("💾 Save (Ctrl+S)").clicked() {
                        self.save_file();
                    }
                });
            });
            ui.separator();

            // Quick insert toolbar
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                if ui.button(egui::RichText::new("𝐁").strong()).on_hover_text("Bold").clicked() { self.insert_command(ctx, "\\textbf{}"); }
                if ui.button(egui::RichText::new("𝐼").italics()).on_hover_text("Italic").clicked() { self.insert_command(ctx, "\\textit{}"); }
                if ui.button(egui::RichText::new("U").underline()).on_hover_text("Underline").clicked() { self.insert_command(ctx, "\\underline{}"); }
                ui.separator();
                if ui.button("H1").on_hover_text("Section").clicked() { self.insert_command(ctx, "\\section{}"); }
                if ui.button("H2").on_hover_text("Subsection").clicked() { self.insert_command(ctx, "\\subsection{}"); }
                if ui.button("H3").on_hover_text("Subsubsection").clicked() { self.insert_command(ctx, "\\subsubsection{}"); }
                ui.separator();
                if ui.button("•").on_hover_text("Itemize").clicked() { self.insert_command(ctx, "\\begin{itemize}\n    \\item \n\\end{itemize}"); }
                if ui.button("1.").on_hover_text("Enumerate").clicked() { self.insert_command(ctx, "\\begin{enumerate}\n    \\item \n\\end{enumerate}"); }
                ui.separator();
                if ui.button("Title").on_hover_text("Title").clicked() { self.insert_command(ctx, "\\title{}"); }
                if ui.button("Auth").on_hover_text("Author").clicked() { self.insert_command(ctx, "\\author{}"); }
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
                if ctx.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Tab)) {
                    ctx.input_mut(|i| {
                        i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                        i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
                    });
                    if let Some((completion, _)) = self.completion_suggestions.get(self.completion_selected_index) {
                        let completion = completion.clone();
                        self.apply_completion(ctx, &mut text, &completion);
                        self.show_completions = false;
                        self.editor_content = text.clone();
                    }
                }
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.show_completions = false;
                }
            }

            let gutter_width = 30.0;
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
                            egui::TextEdit::multiline(&mut text)
                                .id(editor_id)
                                .font(TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .code_editor()
                                .lock_focus(true)
                                .layouter(&mut layouter)
                                .show(ui)
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
                                        if !self.dictionary.is_empty() && !self.dictionary.contains(&lower) {
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
                                  let url = format!("https://api.datamuse.com/words?rel_syn={}", w);
                                  if let Ok(resp) = reqwest::blocking::get(&url) {
                                      if let Ok(json) = resp.json::<Vec<serde_json::Value>>() {
                                           let syns: Vec<String> = json.iter()
                                               .filter_map(|v| v["word"].as_str().map(|s| s.to_string()))
                                               .take(5)
                                               .collect();
                                           let _ = tx.send((w, syns));
                                      }
                                  }
                             });
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
                                ui.label(egui::RichText::new("No spelling suggestions").italics());
                            } else {
                                for suggestion in &selected_suggestions {
                                    if ui.button(format!("Fix: {}", suggestion)).clicked() {
                                        replacement = Some(suggestion.clone());
                                        ui.close_menu();
                                    }
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

                        // Autocomplete Trigger
                        if let Some(state) = egui::TextEdit::load_state(ctx, editor_id) {
                            if let Some(range) = state.cursor.char_range() {
                                let idx = range.primary.index;
                                let text_slice = &self.editor_content[..idx];

                                self.show_completions = false;
                                self.completion_suggestions.clear();

                                if text_slice.ends_with("\\ref{") {
                                    self.show_completions = true;
                                    self.completion_suggestions = self.labels.iter().map(|l| (l.clone(), "Label".to_string())).collect();
                                } else if text_slice.ends_with("\\cite{") {
                                    self.show_completions = true;
                                    self.completion_suggestions = self.bib_items.iter().map(|b| (b.clone(), "Bib".to_string())).collect();
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

                    if self.show_completions && !self.completion_suggestions.is_empty() {
                         let pos = self.completion_popup_pos;
                         egui::Area::new(egui::Id::new("completion_popup"))
                             .fixed_pos(pos)
                             .order(egui::Order::Foreground)
                             .show(ctx, |ui| {
                                 egui::Frame::popup(ui.style()).show(ui, |ui| {
                                     ui.set_max_width(300.0);
                                     ui.set_max_height(200.0);
                                     egui::ScrollArea::vertical().show(ui, |ui| {
                                         let suggestions = self.completion_suggestions.clone();
                                         for (i, (suggestion, kind)) in suggestions.iter().enumerate() {
                                             let selected = i == self.completion_selected_index;
                                             let label = format!("{} ({})", suggestion, kind);
                                             if ui.selectable_label(selected, label).clicked() {
                                                 self.completion_selected_index = i;
                                                 let completion = suggestion.clone();
                                                 self.apply_completion(ctx, &mut text, &completion);
                                                 self.show_completions = false;
                                                 self.editor_content = text.clone();
                                             }
                                         }
                                     });
                                 });
                             });
                    }

                    // Smart Indentation
                    if response.has_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.is_none()) {
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

                    // Define LaTeX command completions
                    let latex_completions: Vec<(&str, &str)> = vec![
                        // Environments
                        ("\\begin{equation}", "\\begin{equation}\n\t\n\\end{equation}"),
                        ("\\begin{align}", "\\begin{align}\n\t\n\\end{align}"),
                        ("\\begin{align*}", "\\begin{align*}\n\t\n\\end{align*}"),
                        ("\\begin{itemize}", "\\begin{itemize}\n\t\\item \n\\end{itemize}"),
                        ("\\begin{enumerate}", "\\begin{enumerate}\n\t\\item \n\\end{enumerate}"),
                        ("\\begin{figure}", "\\begin{figure}[h]\n\t\\centering\n\t\\caption{}\n\\end{figure}"),
                        ("\\begin{table}", "\\begin{table}[h]\n\t\\centering\n\t\\caption{}\n\\end{table}"),
                        ("\\begin{description}", "\\begin{description}\n\t\\item[] \n\\end{description}"),

                        // End Environments
                        ("\\end{equation}", "\\end{equation}"),
                        ("\\end{align}", "\\end{align}"),
                        ("\\end{itemize}", "\\end{itemize}"),
                        ("\\end{enumerate}", "\\end{enumerate}"),
                        ("\\end{figure}", "\\end{figure}"),
                        ("\\end{table}", "\\end{table}"),
                        ("\\end{description}", "\\end{description}"),

                        // Sections
                        ("\\section", "\\section{}"),
                        ("\\subsection", "\\subsection{}"),
                        ("\\subsubsection", "\\subsubsection{}"),
                        ("\\chapter", "\\chapter{}"),
                        ("\\part", "\\part{}"),

                        // Text formatting
                        ("\\textbf{", "\\textbf{}"),
                        ("\\textit{", "\\textit{}"),
                        ("\\texttt{", "\\texttt{}"),
                        ("\\underline{", "\\underline{}"),
                        ("\\emph{", "\\emph{}"),

                        // Math mode
                        ("\\frac{", "\\frac{}{}"),
                        ("\\sqrt{", "\\sqrt{}"),
                        ("\\sum", "\\sum_{n=1}^{N}"),
                        ("\\int", "\\int_{a}^{b}"),
                        ("\\lim", "\\lim_{x \\to 0}"),

                        // References
                        ("\\label{", "\\label{}"),
                        ("\\ref{", "\\ref{}"),
                        ("\\cite{", "\\cite{}"),

                        // Document structure
                        ("\\documentclass{", "\\documentclass{}"),
                        ("\\usepackage{", "\\usepackage{}"),
                        ("\\usepackage[", "\\usepackage[]{}"),

                        // Graphics
                        ("\\includegraphics{", "\\includegraphics{}"),
                        ("\\includegraphics[", "\\includegraphics[width=]{}"),
                        ("[h]", "[h]"),
                        ("[t]", "[t]"),
                        ("[width=]", "[width=]"),
                        ("[scale=]", "[scale=]"),
                        ("\\caption{", "\\caption{}"),

                        // Other
                        ("\\item ", "\\item "),
                        ("\\centering", "\\centering"),
                        ("\\input{", "\\input{}"),
                        ("\\include{", "\\include{}"),
                        ("\\footnote{", "\\footnote{}"),
                        ("\\maketitle", "\\maketitle"),
                        ("\\tableofcontents", "\\tableofcontents"),
                    ];

                    // Handle Tab key
                    if response.has_focus()
                        && ctx.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.is_none())
                    {
                        ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab));
                        text.insert_str(text.len(), "    ");
                    }

                    // Handle Acceptance
                    let mut accepted = false;
                    if self.show_completions && !self.completion_suggestions.is_empty() {
                        let ctrl_space_pressed = ctx.input(|i| i.key_pressed(egui::Key::Space) && i.modifiers.ctrl);

                        if ctrl_space_pressed {
                            ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Space));

                            if let Some((_, completion)) = self.completion_suggestions.get(self.completion_selected_index).cloned() {
                                self.apply_completion(ctx, &mut text, &completion);
                                self.show_completions = false;
                                self.completion_suggestions.clear();
                                accepted = true;
                            }
                        }
                    }

                    // Autocomplete Logic (Open / Update)
                    let ctrl_space = !accepted && response.has_focus()
                        && ctx.input(|i| i.key_pressed(egui::Key::Space) && i.modifiers.ctrl);

                    if ctrl_space {
                        ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Space));
                    }

                    // Trigger if Ctrl+Space OR if text changed while focused
                    if ctrl_space || (response.has_focus() && response.changed() && !accepted) {
                        // Get the actual cursor position from the editor state
                        let cursor_idx = if let Some(state) = egui::TextEdit::load_state(ctx, editor_id) {
                            if let Some(range) = state.cursor.char_range() {
                                range.primary.index
                            } else {
                                text.chars().count()
                            }
                        } else {
                            text.chars().count()
                        };

                        let mut word_start = cursor_idx;
                        let chars: Vec<char> = text.chars().collect();

                        // Scan backwards to find word boundary
                        while word_start > 0 {
                            let prev_char = chars[word_start - 1];
                            let is_word_char = prev_char.is_alphanumeric() || prev_char == '\\' || prev_char == '[' || prev_char == '{' || prev_char == '_' || prev_char == '*';

                            if !is_word_char {
                                break;
                            }
                            word_start -= 1;
                        }

                        // Extract the word being completed
                        let word: String = chars[word_start..cursor_idx].iter().collect();

                        // Find matching completions
                        let mut matches: Vec<(&str, &str)> = latex_completions
                            .iter()
                            .filter(|(trigger, _)| trigger.starts_with(word.as_str()) && trigger.len() >= word.len())
                            .copied()
                            .collect();

                        // If explicitly requested, allow empty word matches
                        // If implicit (typing), require at least one char
                        let should_show = !matches.is_empty() && (ctrl_space || word.len() > 0);

                        if should_show {
                            // Sort by trigger length
                            matches.sort_by_key(|(t, _)| t.len());

                            // Store suggestions for dropdown
                            self.completion_suggestions = matches.iter()
                                .map(|(trigger, completion)| (trigger.to_string(), completion.to_string()))
                                .collect();
                            self.completion_selected_index = 0;

                            // Calculate popup position based on cursor location
                            let font_id = egui::TextStyle::Monospace.resolve(ui.style());
                            let row_height = ui.fonts(|f| f.row_height(&font_id));
                            let char_width = ui.fonts(|f| f.glyph_width(&font_id, ' '));

                            // Find line and column (cursor_idx is char index)
                            let line = chars[..cursor_idx].iter().filter(|&&c| c == '\n').count();
                            let last_newline_idx = chars[..cursor_idx].iter().rposition(|&c| c == '\n').map(|i| i + 1).unwrap_or(0);
                            let col = cursor_idx - last_newline_idx;

                            // Estimate position (relative to text edit top-left)
                            let x_offset = col as f32 * char_width;
                            let y_offset = (line + 1) as f32 * row_height;

                            self.completion_popup_pos = response.rect.min + egui::vec2(x_offset + 8.0, y_offset);
                            self.show_completions = true;
                        } else if !ctrl_space {
                             self.show_completions = false;
                             self.completion_suggestions.clear();
                             self.completion_selected_index = 0;
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
                                        "Compile Project" => if !self.is_compiling { self.compile(); },
                                        "Save File" => self.save_file(),
                                        "Open File" => {
                                            if let Some(path) = rfd::FileDialog::new().add_filter("LaTeX", &["tex"]).pick_file() {
                                                self.file_path = path.to_string_lossy().to_string();
                                                let p = self.file_path.clone();
                                                self.load_file(&p);
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
        std::env::set_var("FONTCONFIG_FILE", fonts_conf_path);
        let config_dir = std::path::Path::new(fonts_conf_path).parent().unwrap_or(std::path::Path::new("."));
        std::env::set_var("FONTCONFIG_PATH", config_dir);
    }
}

fn locate_tectonic() -> String {
    let exe = if cfg!(windows) { "tectonic.exe" } else { "tectonic" };

    if std::path::Path::new(exe).exists() {
        exe.to_string()
    } else {
        let deps_exe = format!("deps/{}", exe);
        if std::path::Path::new(&deps_exe).exists() {
            deps_exe
        } else {
            "tectonic".to_string()
        }
    }
}

fn render_pdf_page_to_texture(
    ctx: &egui::Context,
    pdfium: &Pdfium,
    pdf_path: &std::path::PathBuf,
    page_index: usize,
    zoom: f32,
    preview_size: &mut Option<[usize; 2]>,
) -> Result<egui::TextureHandle, Box<dyn std::error::Error>> {
    let doc = pdfium.load_pdf_from_file(pdf_path, None)?;

    let page_index_u16: u16 = page_index.try_into()?;
    let page = doc.pages().get(page_index_u16)?;

    let target_width = (1400.0 * zoom).round() as i32;
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
    let icon_path = if std::path::Path::new("icon.png").exists() {
        "icon.png"
    } else if std::path::Path::new("deps/icon.png").exists() {
        "deps/icon.png"
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
        .with_min_inner_size([1200.0, 700.0])
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
