use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationType {
    Zoom,
    Slide,
    Fade,
    None,
}

impl AnimationType {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "zoom" => Some(AnimationType::Zoom),
            "slide" => Some(AnimationType::Slide),
            "fade" => Some(AnimationType::Fade),
            "none" => Some(AnimationType::None),
            _ => Option::None,
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            AnimationType::Zoom => "zoom",
            AnimationType::Slide => "slide",
            AnimationType::Fade => "fade",
            AnimationType::None => "none",
        }
    }
}

impl std::fmt::Display for AnimationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowRule {
    // Matchers
    pub appid: Option<String>,
    pub title: Option<String>,

    // State/Behavior
    pub isfloating: Option<bool>,
    pub isfullscreen: Option<bool>,
    pub isglobal: Option<bool>,
    pub isoverlay: Option<bool>,
    pub isopensilent: Option<bool>,
    pub istagsilent: Option<bool>,
    pub force_maximize: Option<bool>,
    pub ignore_maximize: Option<bool>,
    pub ignore_minimize: Option<bool>,
    pub force_tile_state: Option<bool>,
    pub noopenmaximized: Option<bool>,
    pub single_scratchpad: Option<bool>,
    pub allow_shortcuts_inhibit: Option<bool>,
    pub isfakefullscreen: Option<bool>,
    pub indleinhibit_when_focus: Option<bool>,

    // Geometry
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub offsetx: Option<i32>,
    pub offsety: Option<i32>,
    pub monitor: Option<String>,
    pub tags: Option<u8>,
    pub no_force_center: Option<bool>,
    pub isnosizehint: Option<bool>,

    // Visuals
    pub noblur: Option<bool>,
    pub isnoborder: Option<bool>,
    pub isnoshadow: Option<bool>,
    pub isnoradius: Option<bool>,
    pub isnoanimation: Option<bool>,
    pub focused_opacity: Option<f32>,
    pub unfocused_opacity: Option<f32>,
    pub allow_csd: Option<bool>,

    // Animation
    pub animation_type_open: Option<AnimationType>,
    pub animation_type_close: Option<AnimationType>,
    pub nofadein: Option<bool>,
    pub nofadeout: Option<bool>,

    // Layout
    pub scroller_proportion: Option<f32>,
    pub scroller_proportion_single: Option<f32>,

    // Terminal
    pub isterm: Option<bool>,
    pub noswallow: Option<bool>,

    // Special
    pub globalkeybinding: Option<String>,
    pub isunglobal: Option<bool>,
    pub isnamedscratchpad: Option<bool>,
    pub force_tearing: Option<bool>,
}

impl Default for WindowRule {
    fn default() -> Self {
        WindowRule {
            appid: Option::None,
            title: Option::None,
            isfloating: Option::None,
            isfullscreen: Option::None,
            isglobal: Option::None,
            isoverlay: Option::None,
            isopensilent: Option::None,
            istagsilent: Option::None,
            force_maximize: Option::None,
            ignore_maximize: Option::None,
            ignore_minimize: Option::None,
            force_tile_state: Option::None,
            noopenmaximized: Option::None,
            single_scratchpad: Option::None,
            allow_shortcuts_inhibit: Option::None,
            isfakefullscreen: Option::None,
            indleinhibit_when_focus: Option::None,
            width: Option::None,
            height: Option::None,
            offsetx: Option::None,
            offsety: Option::None,
            monitor: Option::None,
            tags: Option::None,
            no_force_center: Option::None,
            isnosizehint: Option::None,
            noblur: Option::None,
            isnoborder: Option::None,
            isnoshadow: Option::None,
            isnoradius: Option::None,
            isnoanimation: Option::None,
            focused_opacity: Option::None,
            unfocused_opacity: Option::None,
            allow_csd: Option::None,
            animation_type_open: Option::None,
            animation_type_close: Option::None,
            nofadein: Option::None,
            nofadeout: Option::None,
            scroller_proportion: Option::None,
            scroller_proportion_single: Option::None,
            isterm: Option::None,
            noswallow: Option::None,
            globalkeybinding: Option::None,
            isunglobal: Option::None,
            isnamedscratchpad: Option::None,
            force_tearing: Option::None,
        }
    }
}

impl WindowRule {
    pub fn from_params(params: &HashMap<String, String>) -> Self {
        let parse_bool = |key: &str| -> Option<bool> {
            match params.get(key).map(|s| s.as_str()) {
                Some("1") => Some(true),
                Some("0") => Some(false),
                _ => Option::None,
            }
        };
        WindowRule {
            appid: params.get("appid").map(|s| s.clone()),
            title: params.get("title").map(|s| s.clone()),
            isfloating: parse_bool("isfloating"),
            isfullscreen: parse_bool("isfullscreen"),
            isglobal: parse_bool("isglobal"),
            isoverlay: parse_bool("isoverlay"),
            isopensilent: parse_bool("isopensilent"),
            istagsilent: parse_bool("istagsilent"),
            force_maximize: parse_bool("force_maximize"),
            ignore_maximize: parse_bool("ignore_maximize"),
            ignore_minimize: parse_bool("ignore_minimize"),
            force_tile_state: parse_bool("force_tile_state"),
            noopenmaximized: parse_bool("noopenmaximized"),
            single_scratchpad: parse_bool("single_scratchpad"),
            allow_shortcuts_inhibit: parse_bool("allow_shortcuts_inhibit"),
            isfakefullscreen: parse_bool("isfakefullscreen"),
            indleinhibit_when_focus: parse_bool("indleinhibit_when_focus"),
            width: params.get("width").and_then(|s| s.parse().ok()),
            height: params.get("height").and_then(|s| s.parse().ok()),
            offsetx: params.get("offsetx").and_then(|s| s.parse().ok()),
            offsety: params.get("offsety").and_then(|s| s.parse().ok()),
            monitor: params.get("monitor").map(|s| s.clone()),
            tags: params.get("tags").and_then(|s| s.parse().ok()),
            no_force_center: parse_bool("no_force_center"),
            isnosizehint: parse_bool("isnosizehint"),
            noblur: parse_bool("noblur"),
            isnoborder: parse_bool("isnoborder"),
            isnoshadow: parse_bool("isnoshadow"),
            isnoradius: parse_bool("isnoradius"),
            isnoanimation: parse_bool("isnoanimation"),
            focused_opacity: params.get("focused_opacity").and_then(|s| s.parse().ok()),
            unfocused_opacity: params.get("unfocused_opacity").and_then(|s| s.parse().ok()),
            allow_csd: parse_bool("allow_csd"),
            animation_type_open: params
                .get("animation_type_open")
                .and_then(|s| AnimationType::from_str(s)),
            animation_type_close: params
                .get("animation_type_close")
                .and_then(|s| AnimationType::from_str(s)),
            nofadein: parse_bool("nofadein"),
            nofadeout: parse_bool("nofadeout"),
            scroller_proportion: params
                .get("scroller_proportion")
                .and_then(|s| s.parse().ok()),
            scroller_proportion_single: params
                .get("scroller_proportion_single")
                .and_then(|s| s.parse().ok()),
            isterm: parse_bool("isterm"),
            noswallow: parse_bool("noswallow"),
            globalkeybinding: params.get("globalkeybinding").map(|s| s.clone()),
            isunglobal: parse_bool("isunglobal"),
            isnamedscratchpad: parse_bool("isnamedscratchpad"),
            force_tearing: parse_bool("force_tearing"),
        }
    }

    pub fn to_conf_line(&self) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();

        if let Some(ref v) = self.appid {
            parts.push(format!("appid:{}", v));
        }
        if let Some(ref v) = self.title {
            parts.push(format!("title:{}", v));
        }
        if let Some(v) = self.isfloating {
            parts.push(format!("isfloating:{}", v as u8));
        }
        if let Some(v) = self.isfullscreen {
            parts.push(format!("isfullscreen:{}", v as u8));
        }
        if let Some(v) = self.isglobal {
            parts.push(format!("isglobal:{}", v as u8));
        }
        if let Some(v) = self.isoverlay {
            parts.push(format!("isoverlay:{}", v as u8));
        }
        if let Some(v) = self.isopensilent {
            parts.push(format!("isopensilent:{}", v as u8));
        }
        if let Some(v) = self.istagsilent {
            parts.push(format!("istagsilent:{}", v as u8));
        }
        if let Some(v) = self.force_maximize {
            parts.push(format!("force_maximize:{}", v as u8));
        }
        if let Some(v) = self.ignore_maximize {
            parts.push(format!("ignore_maximize:{}", v as u8));
        }
        if let Some(v) = self.ignore_minimize {
            parts.push(format!("ignore_minimize:{}", v as u8));
        }
        if let Some(v) = self.force_tile_state {
            parts.push(format!("force_tile_state:{}", v as u8));
        }
        if let Some(v) = self.noopenmaximized {
            parts.push(format!("noopenmaximized:{}", v as u8));
        }
        if let Some(v) = self.single_scratchpad {
            parts.push(format!("single_scratchpad:{}", v as u8));
        }
        if let Some(v) = self.allow_shortcuts_inhibit {
            parts.push(format!("allow_shortcuts_inhibit:{}", v as u8));
        }
        if let Some(v) = self.isfakefullscreen {
            parts.push(format!("isfakefullscreen:{}", v as u8));
        }
        if let Some(v) = self.indleinhibit_when_focus {
            parts.push(format!("indleinhibit_when_focus:{}", v as u8));
        }
        if let Some(v) = self.width {
            parts.push(format!("width:{}", v));
        }
        if let Some(v) = self.height {
            parts.push(format!("height:{}", v));
        }
        if let Some(v) = self.offsetx {
            parts.push(format!("offsetx:{}", v));
        }
        if let Some(v) = self.offsety {
            parts.push(format!("offsety:{}", v));
        }
        if let Some(ref v) = self.monitor {
            parts.push(format!("monitor:{}", v));
        }
        if let Some(v) = self.tags {
            parts.push(format!("tags:{}", v));
        }
        if let Some(v) = self.no_force_center {
            parts.push(format!("no_force_center:{}", v as u8));
        }
        if let Some(v) = self.isnosizehint {
            parts.push(format!("isnosizehint:{}", v as u8));
        }
        if let Some(v) = self.noblur {
            parts.push(format!("noblur:{}", v as u8));
        }
        if let Some(v) = self.isnoborder {
            parts.push(format!("isnoborder:{}", v as u8));
        }
        if let Some(v) = self.isnoshadow {
            parts.push(format!("isnoshadow:{}", v as u8));
        }
        if let Some(v) = self.isnoradius {
            parts.push(format!("isnoradius:{}", v as u8));
        }
        if let Some(v) = self.isnoanimation {
            parts.push(format!("isnoanimation:{}", v as u8));
        }
        if let Some(v) = self.focused_opacity {
            parts.push(format!("focused_opacity:{}", v));
        }
        if let Some(v) = self.unfocused_opacity {
            parts.push(format!("unfocused_opacity:{}", v));
        }
        if let Some(v) = self.allow_csd {
            parts.push(format!("allow_csd:{}", v as u8));
        }
        if let Some(ref v) = self.animation_type_open {
            parts.push(format!("animation_type_open:{}", v.to_str()));
        }
        if let Some(ref v) = self.animation_type_close {
            parts.push(format!("animation_type_close:{}", v.to_str()));
        }
        if let Some(v) = self.nofadein {
            parts.push(format!("nofadein:{}", v as u8));
        }
        if let Some(v) = self.nofadeout {
            parts.push(format!("nofadeout:{}", v as u8));
        }
        if let Some(v) = self.scroller_proportion {
            parts.push(format!("scroller_proportion:{}", v));
        }
        if let Some(v) = self.scroller_proportion_single {
            parts.push(format!("scroller_proportion_single:{}", v));
        }
        if let Some(v) = self.isterm {
            parts.push(format!("isterm:{}", v as u8));
        }
        if let Some(v) = self.noswallow {
            parts.push(format!("noswallow:{}", v as u8));
        }
        if let Some(ref v) = self.globalkeybinding {
            parts.push(format!("globalkeybinding:{}", v));
        }
        if let Some(v) = self.isunglobal {
            parts.push(format!("isunglobal:{}", v as u8));
        }
        if let Some(v) = self.isnamedscratchpad {
            parts.push(format!("isnamedscratchpad:{}", v as u8));
        }
        if let Some(v) = self.force_tearing {
            parts.push(format!("force_tearing:{}", v as u8));
        }

        if parts.is_empty() {
            return Option::None;
        }
        Some(format!("windowrule={}", parts.join(",")))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TagRule {
    pub id: Option<u8>,
    pub layout_name: Option<String>,
    pub nmaster: Option<u32>,
    pub mfact: Option<f32>,
    pub monitor_name: Option<String>,
}

impl Default for TagRule {
    fn default() -> Self {
        TagRule {
            id: Option::None,
            layout_name: Option::None,
            nmaster: Option::None,
            mfact: Option::None,
            monitor_name: Option::None,
        }
    }
}

impl TagRule {
    pub fn from_params(params: &HashMap<String, String>) -> Self {
        TagRule {
            id: params.get("id").and_then(|s| s.parse().ok()),
            layout_name: params.get("layout_name").map(|s| s.clone()),
            nmaster: params.get("nmaster").and_then(|s| s.parse().ok()),
            mfact: params.get("mfact").and_then(|s| s.parse().ok()),
            monitor_name: params.get("monitor_name").map(|s| s.clone()),
        }
    }

    pub fn to_conf_line(&self) -> Option<String> {
        // A tagrule without id is meaningless
        let _id = self.id?;

        let mut parts: Vec<String> = Vec::new();

        if let Some(v) = self.id {
            parts.push(format!("id:{}", v));
        }
        if let Some(ref v) = self.layout_name {
            parts.push(format!("layout_name:{}", v));
        }
        if let Some(v) = self.nmaster {
            parts.push(format!("nmaster:{}", v));
        }
        if let Some(v) = self.mfact {
            parts.push(format!("mfact:{}", v));
        }
        if let Some(ref v) = self.monitor_name {
            parts.push(format!("monitor_name:{}", v));
        }

        if parts.is_empty() {
            return Option::None;
        }
        Some(format!("tagrule={}", parts.join(",")))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerRule {
    // Matcher
    pub layer_name: Option<String>,

    // Visuals
    pub noblur: Option<bool>,
    pub noshadow: Option<bool>,

    // Animation
    pub noanim: Option<bool>,
    pub animation_type_open: Option<AnimationType>,
    pub animation_type_close: Option<AnimationType>,
}

impl Default for LayerRule {
    fn default() -> Self {
        LayerRule {
            layer_name: Option::None,
            noblur: Option::None,
            noshadow: Option::None,
            noanim: Option::None,
            animation_type_open: Option::None,
            animation_type_close: Option::None,
        }
    }
}

impl LayerRule {
    pub fn from_params(params: &HashMap<String, String>) -> Self {
        let parse_bool = |key: &str| -> Option<bool> {
            match params.get(key).map(|s| s.as_str()) {
                Some("1") => Some(true),
                Some("0") => Some(false),
                _ => Option::None,
            }
        };
        LayerRule {
            layer_name: params.get("layer_name").map(|s| s.clone()),
            noblur: parse_bool("noblur"),
            noshadow: parse_bool("noshadow"),
            noanim: parse_bool("noanim"),
            animation_type_open: params
                .get("animation_type_open")
                .and_then(|s| AnimationType::from_str(s)),
            animation_type_close: params
                .get("animation_type_close")
                .and_then(|s| AnimationType::from_str(s)),
        }
    }

    pub fn to_conf_line(&self) -> Option<String> {
        // A layerrule without layer_name matcher is meaningless
        self.layer_name.as_ref()?;

        let mut parts: Vec<String> = Vec::new();

        if let Some(ref v) = self.layer_name {
            parts.push(format!("layer_name:{}", v));
        }
        if let Some(v) = self.noblur {
            parts.push(format!("noblur:{}", v as u8));
        }
        if let Some(v) = self.noshadow {
            parts.push(format!("noshadow:{}", v as u8));
        }
        if let Some(v) = self.noanim {
            parts.push(format!("noanim:{}", v as u8));
        }
        if let Some(ref v) = self.animation_type_open {
            parts.push(format!("animation_type_open:{}", v.to_str()));
        }
        if let Some(ref v) = self.animation_type_close {
            parts.push(format!("animation_type_close:{}", v.to_str()));
        }

        if parts.is_empty() {
            return Option::None;
        }
        Some(format!("layerrule={}", parts.join(",")))
    }
}

pub fn hash_file(path: &std::path::Path) -> Option<u64> {
    let content = fs::read(path).ok()?;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    Some(hasher.finish())
}

pub fn backup_rules(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;
    }
    fs::copy(src, dst)
        .map_err(|e| format!("Failed to backup rules: {}", e))?;
    Ok(())
}

fn parse_token(token: &str) -> Option<(String, String)> {
    let mut parts = token.trim().splitn(2, ':');
    let key = parts.next()?.trim().to_string();
    let val = parts.next()?.trim().to_string();
    Some((key, val))
}

pub fn parse_rules_file(content: &str) -> (Vec<WindowRule>, Vec<TagRule>, Vec<LayerRule>) {
    let mut window_rules: Vec<WindowRule> = Vec::new();
    let mut tag_rules: Vec<TagRule> = Vec::new();
    let mut layer_rules: Vec<LayerRule> = Vec::new();

    for line in content.lines() {
        let t = line.trim();
        if t.starts_with('#') || t.is_empty() {
            continue;
        }
        let Some((keyword, params_str)) = t.split_once('=') else {
            continue;
        };
        let params: HashMap<String, String> = params_str.split(',').filter_map(parse_token).collect();
        match keyword.trim() {
            "windowrule" => window_rules.push(WindowRule::from_params(&params)),
            "tagrule" => tag_rules.push(TagRule::from_params(&params)),
            "layerrule" => layer_rules.push(LayerRule::from_params(&params)),
            _ => {}
        }
    }

    (window_rules, tag_rules, layer_rules)
}

pub struct RulesConfig {
    pub window_rules: Vec<WindowRule>,
    pub tag_rules: Vec<TagRule>,
    pub layer_rules: Vec<LayerRule>,
}

impl Default for RulesConfig {
    fn default() -> Self {
        RulesConfig {
            window_rules: Vec::new(),
            tag_rules: Vec::new(),
            layer_rules: Vec::new(),
        }
    }
}

impl RulesConfig {
    pub fn load(path: &std::path::Path) -> Self {
        if !path.exists() {
            return Self::default();
        }
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        let (window_rules, tag_rules, layer_rules) = parse_rules_file(&content);
        RulesConfig {
            window_rules,
            tag_rules,
            layer_rules,
        }
    }

    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create rules config directory: {}", e))?;
        }

        // Preserve non-rule lines from the existing file (user comments, other directives).
        // Strip out any lines that are managed rule entries or the mdiplay-generated header.
        let preserved: Vec<String> = if path.exists() {
            let existing = fs::read_to_string(path)
                .map_err(|e| format!("Failed to read existing rules config: {}", e))?;
            existing
                .lines()
                .filter(|line| {
                    let t = line.trim();
                    let is_managed_rule = t.starts_with("windowrule=")
                        || t.starts_with("tagrule=")
                        || t.starts_with("layerrule=");
                    let is_generated_header = t == "# Generated by mdiplay";
                    !is_managed_rule && !is_generated_header
                })
                .map(|l| l.to_string())
                .collect()
        } else {
            Vec::new()
        };

        // Build the rule lines from the current in-memory state.
        let mut rule_lines: Vec<String> = Vec::new();
        for rule in &self.window_rules {
            if let Some(line) = rule.to_conf_line() {
                rule_lines.push(line);
            }
        }
        for rule in &self.tag_rules {
            if let Some(line) = rule.to_conf_line() {
                rule_lines.push(line);
            }
        }
        for rule in &self.layer_rules {
            if let Some(line) = rule.to_conf_line() {
                rule_lines.push(line);
            }
        }

        // Combine: preserved non-rule content + managed rule block.
        let mut output_parts: Vec<String> = preserved;
        // Trim trailing blank lines from preserved content before appending rules.
        while output_parts.last().map(|l: &String| l.trim().is_empty()).unwrap_or(false) {
            output_parts.pop();
        }
        if !rule_lines.is_empty() {
            if !output_parts.is_empty() {
                output_parts.push(String::new());
            }
            output_parts.push("# Generated by mdiplay".to_string());
            output_parts.extend(rule_lines);
        }
        output_parts.push(String::new()); // trailing newline

        let content = output_parts.join("\n");
        fs::write(path, content)
            .map_err(|e| format!("Failed to write rules config: {}", e))?;

        Ok(())
    }
}
