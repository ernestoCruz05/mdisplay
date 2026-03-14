use iced::widget::canvas::{self, Action, Cache, Canvas, Event, Geometry, Path, Program};
use iced::widget::{
    Container, Scrollable, Space, button, checkbox, column, container, pick_list, row, scrollable,
    text, text_input, tooltip,
};
use iced::{
    Color, Element, Length, Point, Rectangle, Renderer, Size, Task, Theme, alignment, mouse,
};
use std::str::FromStr;

use crate::backend::{Output, OutputMode, restore_default_config, save_config};
use crate::rules::RulesConfig;
use crate::wayland::{apply_outputs, fetch_outputs, fetch_toplevels, ToplevelInfo};

#[derive(Debug, Clone)]
pub enum StatusMessage {
    Success(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Monitor,
    Rules,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Window,
    Tag,
    Layer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TriState {
    Unset,
    Yes,
    No,
}

impl TriState {
    fn from_opt(v: Option<bool>) -> Self {
        match v {
            None => TriState::Unset,
            Some(true) => TriState::Yes,
            Some(false) => TriState::No,
        }
    }
    fn to_opt(self) -> Option<bool> {
        match self {
            TriState::Unset => None,
            TriState::Yes => Some(true),
            TriState::No => Some(false),
        }
    }
}

impl std::fmt::Display for TriState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TriState::Unset => "(unset)",
            TriState::Yes => "Yes",
            TriState::No => "No",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnimTypePick {
    Unset,
    Zoom,
    Slide,
    Fade,
    NoAnim,
}

impl AnimTypePick {
    fn from_opt(v: &Option<crate::rules::AnimationType>) -> Self {
        use crate::rules::AnimationType;
        match v {
            None => AnimTypePick::Unset,
            Some(AnimationType::Zoom) => AnimTypePick::Zoom,
            Some(AnimationType::Slide) => AnimTypePick::Slide,
            Some(AnimationType::Fade) => AnimTypePick::Fade,
            Some(AnimationType::None) => AnimTypePick::NoAnim,
        }
    }
    fn to_opt(self) -> Option<crate::rules::AnimationType> {
        use crate::rules::AnimationType;
        match self {
            AnimTypePick::Unset => None,
            AnimTypePick::Zoom => Some(AnimationType::Zoom),
            AnimTypePick::Slide => Some(AnimationType::Slide),
            AnimTypePick::Fade => Some(AnimationType::Fade),
            AnimTypePick::NoAnim => Some(AnimationType::None),
        }
    }
}

impl std::fmt::Display for AnimTypePick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AnimTypePick::Unset => "(unset)",
            AnimTypePick::Zoom => "Zoom",
            AnimTypePick::Slide => "Slide",
            AnimTypePick::Fade => "Fade",
            AnimTypePick::NoAnim => "None",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormTab {
    StateBehavior,
    Geometry,
    Visuals,
    Animation,
    LayoutTerminal,
    Special,
}

#[derive(Debug, Clone)]
pub enum WindowRuleField {
    // Matchers
    AppId(String),
    Title(String),
    // State/Behavior (Option<bool> via TriState)
    IsFloating(Option<bool>),
    IsFullscreen(Option<bool>),
    IsGlobal(Option<bool>),
    IsOverlay(Option<bool>),
    IsOpenSilent(Option<bool>),
    IsTagSilent(Option<bool>),
    ForceMaximize(Option<bool>),
    IgnoreMaximize(Option<bool>),
    IgnoreMinimize(Option<bool>),
    ForceTileState(Option<bool>),
    NoOpenMaximized(Option<bool>),
    SingleScratchpad(Option<bool>),
    AllowShortcutsInhibit(Option<bool>),
    IsFakeFullscreen(Option<bool>),
    IndleInhibitWhenFocus(Option<bool>),
    // Geometry (String inputs, parsed to Option<T>)
    Width(String),
    Height(String),
    OffsetX(String),
    OffsetY(String),
    Monitor(String),
    Tags(String),
    NoForceCenter(Option<bool>),
    IsNoSizeHint(Option<bool>),
    // Visuals
    NoBlur(Option<bool>),
    IsNoBorder(Option<bool>),
    IsNoShadow(Option<bool>),
    IsNoRadius(Option<bool>),
    IsNoAnimation(Option<bool>),
    FocusedOpacity(String),
    UnfocusedOpacity(String),
    AllowCsd(Option<bool>),
    // Animation
    AnimTypeOpen(Option<crate::rules::AnimationType>),
    AnimTypeClose(Option<crate::rules::AnimationType>),
    NoFadeIn(Option<bool>),
    NoFadeOut(Option<bool>),
    // Layout
    ScrollerProportion(String),
    ScrollerProportionSingle(String),
    // Terminal
    IsTerm(Option<bool>),
    NoSwallow(Option<bool>),
    // Special
    GlobalKeyBinding(String),
    IsUnGlobal(Option<bool>),
    IsNamedScratchpad(Option<bool>),
    ForceTearing(Option<bool>),
}

#[derive(Debug, Clone)]
pub enum TagRuleField {
    Id(String),
    LayoutName(String),
    Nmaster(String),
    Mfact(String),
    MonitorName(String),
}

#[derive(Debug, Clone)]
pub enum LayerRuleField {
    LayerName(String),
    NoBlur(Option<bool>),
    NoShadow(Option<bool>),
    NoAnim(Option<bool>),
    AnimTypeOpen(Option<crate::rules::AnimationType>),
    AnimTypeClose(Option<crate::rules::AnimationType>),
}

#[derive(Debug, Clone)]
pub enum Message {
    MonitorClicked(usize),
    MonitorPositioned(usize, i32, i32),
    XChanged(String),
    YChanged(String),
    XInc,
    XDec,
    YInc,
    YDec,
    ScaleChanged(String),
    ScaleInc,
    ScaleDec,
    EnabledToggled(bool),
    ResolutionSelected(usize),
    TransformSelected(String),
    ApplyClicked,
    SaveClicked,
    RestoreDefaultClicked,
    ResolutionSizeSelected(String),
    SwitchView(View),
    SelectTab(Tab),
    DeleteWindowRule(usize),
    DeleteTagRule(usize),
    DeleteLayerRule(usize),
    AddWindowRule,
    EditWindowRule(usize),
    SaveWindowRule,
    CancelWindowRule,
    FormTabSelected(FormTab),
    WindowRuleChanged(WindowRuleField),
    OpenWindowPicker,
    WindowSelected(String),
    CancelWindowPicker,
    AddTagRule,
    EditTagRule(usize),
    SaveTagRule,
    CancelTagRule,
    TagRuleChanged(TagRuleField),
    AddLayerRule,
    EditLayerRule(usize),
    SaveLayerRule,
    CancelLayerRule,
    LayerRuleChanged(LayerRuleField),
    SaveRulesConfig,
    DismissStatus,
}

/// Raw string drafts for float fields so intermediate input (e.g. "0.") is preserved.
#[derive(Debug, Default, Clone)]
struct FloatDrafts {
    focused_opacity: String,
    unfocused_opacity: String,
    scroller_proportion: String,
    scroller_proportion_single: String,
    mfact: String,
}

pub struct MangoDisplay {
    outputs: Vec<Output>,
    selected_output_idx: Option<usize>,
    layout_cache: Cache,
    x_input: String,
    y_input: String,
    scale_input: String,
    pub settings: crate::settings::AppSettings,
    status_message: Option<StatusMessage>,
    rules_config: RulesConfig,
    current_view: View,
    current_tab: Tab,
    editing_window_rule: Option<(Option<usize>, crate::rules::WindowRule)>,
    form_tab: FormTab,
    show_window_picker: bool,
    available_windows: Vec<ToplevelInfo>,
    editing_tag_rule: Option<(Option<usize>, crate::rules::TagRule)>,
    editing_layer_rule: Option<(Option<usize>, crate::rules::LayerRule)>,
    float_drafts: FloatDrafts,
}

impl Default for MangoDisplay {
    fn default() -> Self {
        let mut settings = crate::settings::AppSettings::load();

        let rules_path = expand_tilde(&settings.rules_conf_path);
        let bak_path = expand_tilde(&settings.rules_bak_path);

        // Detect external changes to rules.conf via hash comparison.
        // On first run with a pre-existing file, or whenever the file changed
        // outside mdisplay, backup to rules.bak and re-import.
        let mut import_status: Option<StatusMessage> = None;
        let current_hash = crate::rules::hash_file(&rules_path);
        if rules_path.exists() && current_hash != settings.rules_conf_hash {
            match crate::rules::backup_rules(&rules_path, &bak_path) {
                Ok(()) => {
                    import_status = Some(StatusMessage::Success(format!(
                        "Rules imported from {} (backup → {})",
                        settings.rules_conf_path, settings.rules_bak_path
                    )));
                }
                Err(e) => {
                    import_status = Some(StatusMessage::Error(format!(
                        "Could not backup rules.conf: {}", e
                    )));
                }
            }
            settings.rules_conf_hash = current_hash;
            let _ = settings.save();
        }

        let rules_config = RulesConfig::load(&rules_path);
        let outputs = fetch_outputs().unwrap_or_default();
        let selected_output_idx = if !outputs.is_empty() { Some(0) } else { None };
        let mut app = Self {
            outputs,
            selected_output_idx,
            layout_cache: Cache::default(),
            x_input: String::new(),
            y_input: String::new(),
            scale_input: String::new(),
            settings,
            status_message: import_status,
            rules_config,
            current_view: View::Monitor,
            current_tab: Tab::Window,
            editing_window_rule: None,
            form_tab: FormTab::StateBehavior,
            show_window_picker: false,
            available_windows: Vec::new(),
            editing_tag_rule: None,
            editing_layer_rule: None,
            float_drafts: FloatDrafts::default(),
        };
        app.update_inputs_for_selection();
        app
    }
}

static TRISTATE_OPTS: &[TriState] = &[TriState::Unset, TriState::Yes, TriState::No];
static ANIM_TYPE_OPTS: &[AnimTypePick] = &[
    AnimTypePick::Unset,
    AnimTypePick::Zoom,
    AnimTypePick::Slide,
    AnimTypePick::Fade,
    AnimTypePick::NoAnim,
];

fn expand_tilde(raw: &str) -> std::path::PathBuf {
    if raw.starts_with("~/") {
        dirs::home_dir()
            .map(|h| h.join(&raw[2..]))
            .unwrap_or_else(|| std::path::PathBuf::from(raw))
    } else {
        std::path::PathBuf::from(raw)
    }
}

fn info_tip<'a>(desc: &'a str) -> Option<Element<'a, Message>> {
    if desc.is_empty() {
        return None;
    }
    let tip_content = container(text(desc).size(12))
        .padding([4, 8])
        .style(container::rounded_box);
    let indicator = iced_fonts::bootstrap::info_circle().size(12).color([0.5, 0.5, 0.5]);
    Some(
        tooltip(indicator, tip_content, tooltip::Position::Right)
            .into()
    )
}

fn tri_row<'a>(label: &'a str, desc: &'a str, value: Option<bool>, on_change: impl Fn(Option<bool>) -> Message + 'a) -> Element<'a, Message> {
    let mut r = row![
        container(text(label).size(14)).width(Length::Fixed(170.0)),
        pick_list(TRISTATE_OPTS, Some(TriState::from_opt(value)), move |ts: TriState| on_change(ts.to_opt()))
            .width(Length::Fixed(100.0)),
    ]
    .spacing(8)
    .align_y(alignment::Vertical::Center);
    if let Some(tip) = info_tip(desc) {
        r = r.push(tip);
    }
    r.into()
}

fn str_row<'a>(label: &'a str, desc: &'a str, value: &str, on_input: impl Fn(String) -> Message + 'a) -> Element<'a, Message> {
    let mut r = row![
        container(text(label).size(14)).width(Length::Fixed(170.0)),
        text_input("", value)
            .on_input(on_input)
            .width(Length::Fixed(180.0)),
    ]
    .spacing(8)
    .align_y(alignment::Vertical::Center);
    if let Some(tip) = info_tip(desc) {
        r = r.push(tip);
    }
    r.into()
}

impl MangoDisplay {
    fn update_inputs_for_selection(&mut self) {
        if let Some(idx) = self.selected_output_idx {
            if let Some(out) = self.outputs.get(idx) {
                self.x_input = out.position.0.to_string();
                self.y_input = out.position.1.to_string();
                self.scale_input = format!("{:.2}", out.scale);
            }
        }
    }

    fn normalize_positions(&mut self) {
        let min_x = self.outputs.iter().map(|o| o.position.0).min().unwrap_or(0);
        let min_y = self.outputs.iter().map(|o| o.position.1).min().unwrap_or(0);

        let mut changed = false;
        let offset_x = if min_x < 0 { -min_x } else { 0 };
        let offset_y = if min_y < 0 { -min_y } else { 0 };

        if offset_x > 0 || offset_y > 0 {
            for out in &mut self.outputs {
                out.position.0 += offset_x;
                out.position.1 += offset_y;
            }
            changed = true;
        }

        if changed {
            self.update_inputs_for_selection();
            self.layout_cache.clear();
        }
    }

    fn set_status(&mut self, msg: StatusMessage) -> Task<Message> {
        self.status_message = Some(msg);
        Task::perform(
            futures_timer::Delay::new(std::time::Duration::from_secs(4)),
            |_| Message::DismissStatus,
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let mut task = Task::none();
        match message {
            Message::MonitorClicked(idx) => {
                self.selected_output_idx = Some(idx);
                self.update_inputs_for_selection();
                self.layout_cache.clear();
            }
            Message::MonitorPositioned(idx, x, y) => {
                if let Some(out) = self.outputs.get_mut(idx) {
                    out.position = (x, y);
                }
                if Some(idx) == self.selected_output_idx {
                    self.update_inputs_for_selection();
                }
                self.layout_cache.clear();
            }
            Message::XChanged(val) => {
                self.x_input = val.clone();
                if let (Some(idx), Ok(mut v)) = (self.selected_output_idx, i32::from_str(&val)) {
                    if v < 0 {
                        v = 0;
                    }
                    self.outputs[idx].position.0 = v;
                    self.layout_cache.clear();
                }
            }
            Message::YChanged(val) => {
                self.y_input = val.clone();
                if let (Some(idx), Ok(mut v)) = (self.selected_output_idx, i32::from_str(&val)) {
                    if v < 0 {
                        v = 0;
                    }
                    self.outputs[idx].position.1 = v;
                    self.layout_cache.clear();
                }
            }
            Message::XInc => {
                if let Some(idx) = self.selected_output_idx {
                    self.outputs[idx].position.0 += 1;
                    self.update_inputs_for_selection();
                    self.layout_cache.clear();
                }
            }
            Message::XDec => {
                if let Some(idx) = self.selected_output_idx {
                    if self.outputs[idx].position.0 > 0 {
                        self.outputs[idx].position.0 -= 1;
                        self.update_inputs_for_selection();
                        self.layout_cache.clear();
                    }
                }
            }
            Message::YInc => {
                if let Some(idx) = self.selected_output_idx {
                    self.outputs[idx].position.1 += 1;
                    self.update_inputs_for_selection();
                    self.layout_cache.clear();
                }
            }
            Message::YDec => {
                if let Some(idx) = self.selected_output_idx {
                    if self.outputs[idx].position.1 > 0 {
                        self.outputs[idx].position.1 -= 1;
                        self.update_inputs_for_selection();
                        self.layout_cache.clear();
                    }
                }
            }
            Message::ScaleChanged(val) => {
                self.scale_input = val.clone();
                if let (Some(idx), Ok(v)) = (self.selected_output_idx, f32::from_str(&val)) {
                    if v > 0.1 {
                        self.outputs[idx].scale = v;
                        self.layout_cache.clear();
                    }
                }
            }
            Message::ScaleInc => {
                if let Some(idx) = self.selected_output_idx {
                    self.outputs[idx].scale += 0.05;
                    self.update_inputs_for_selection();
                    self.layout_cache.clear();
                }
            }
            Message::ScaleDec => {
                if let Some(idx) = self.selected_output_idx {
                    self.outputs[idx].scale -= 0.05;
                    self.update_inputs_for_selection();
                    self.layout_cache.clear();
                }
            }
            Message::EnabledToggled(val) => {
                if let Some(idx) = self.selected_output_idx {
                    self.outputs[idx].enabled = val;
                    self.layout_cache.clear();
                }
            }

            Message::ResolutionSizeSelected(res_str) => {
                if let Some(idx) = self.selected_output_idx {
                    let parts: Vec<&str> = res_str.split('x').collect();
                    if parts.len() == 2 {
                        if let (Ok(w), Ok(h)) = (i32::from_str(parts[0]), i32::from_str(parts[1])) {
                            for m in &mut self.outputs[idx].modes {
                                m.current = false;
                            }
                            if let Some(mode) = self.outputs[idx]
                                .modes
                                .iter_mut()
                                .find(|m| m.width == w && m.height == h)
                            {
                                mode.current = true;
                            }
                            self.layout_cache.clear();
                        }
                    }
                }
            }
            Message::ResolutionSelected(res_idx) => {
                if let Some(idx) = self.selected_output_idx {
                    for m in &mut self.outputs[idx].modes {
                        m.current = false;
                    }
                    if res_idx < self.outputs[idx].modes.len() {
                        self.outputs[idx].modes[res_idx].current = true;
                    }
                    self.layout_cache.clear();
                }
            }
            Message::TransformSelected(trans) => {
                if let Some(idx) = self.selected_output_idx {
                    self.outputs[idx].transform = trans;
                    self.layout_cache.clear();
                }
            }
            Message::ApplyClicked => {
                self.normalize_positions();
                match apply_outputs(&self.outputs) {
                    Ok(()) => task = self.set_status(StatusMessage::Success("Applied successfully!".into())),
                    Err(e) => task = self.set_status(StatusMessage::Error(format!("Apply error: {}", e))),
                }
            }
            Message::SaveClicked => {
                self.normalize_positions();
                match save_config(&self.outputs, &self.settings) {
                    Ok(()) => task = self.set_status(StatusMessage::Success(
                        format!("Saved to {}", self.settings.monitors_conf_path)
                    )),
                    Err(e) => task = self.set_status(StatusMessage::Error(format!("Save error: {}", e))),
                }
            }
            Message::RestoreDefaultClicked => match restore_default_config(&self.settings) {
                Ok(()) => task = self.set_status(StatusMessage::Success("Restored to default config!".into())),
                Err(e) => task = self.set_status(StatusMessage::Error(format!("Restore error: {}", e))),
            },
            Message::SwitchView(v) => {
                self.current_view = v;
            }
            Message::SelectTab(t) => {
                self.current_tab = t;
            }
            Message::DeleteWindowRule(idx) => {
                if idx < self.rules_config.window_rules.len() {
                    self.rules_config.window_rules.remove(idx);
                }
            }
            Message::DeleteTagRule(idx) => {
                if idx < self.rules_config.tag_rules.len() {
                    self.rules_config.tag_rules.remove(idx);
                }
            }
            Message::DeleteLayerRule(idx) => {
                if idx < self.rules_config.layer_rules.len() {
                    self.rules_config.layer_rules.remove(idx);
                }
            }
            Message::AddWindowRule => {
                let rule = crate::rules::WindowRule::default();
                self.float_drafts = FloatDrafts {
                    focused_opacity: rule.focused_opacity.map(|v| format!("{:.2}", v)).unwrap_or_default(),
                    unfocused_opacity: rule.unfocused_opacity.map(|v| format!("{:.2}", v)).unwrap_or_default(),
                    scroller_proportion: rule.scroller_proportion.map(|v| format!("{:.2}", v)).unwrap_or_default(),
                    scroller_proportion_single: rule.scroller_proportion_single.map(|v| format!("{:.2}", v)).unwrap_or_default(),
                    mfact: String::new(),
                };
                self.editing_window_rule = Some((None, rule));
                self.form_tab = FormTab::StateBehavior;
            }
            Message::EditWindowRule(i) => {
                if let Some(rule) = self.rules_config.window_rules.get(i) {
                    self.float_drafts = FloatDrafts {
                        focused_opacity: rule.focused_opacity.map(|v| format!("{:.2}", v)).unwrap_or_default(),
                        unfocused_opacity: rule.unfocused_opacity.map(|v| format!("{:.2}", v)).unwrap_or_default(),
                        scroller_proportion: rule.scroller_proportion.map(|v| format!("{:.2}", v)).unwrap_or_default(),
                        scroller_proportion_single: rule.scroller_proportion_single.map(|v| format!("{:.2}", v)).unwrap_or_default(),
                        mfact: String::new(),
                    };
                    self.editing_window_rule = Some((Some(i), rule.clone()));
                    self.form_tab = FormTab::StateBehavior;
                }
            }
            Message::SaveWindowRule => {
                if let Some((idx, draft)) = self.editing_window_rule.take() {
                    let appid_empty = draft.appid.as_ref().map(|s| s.is_empty()).unwrap_or(true);
                    let title_empty = draft.title.as_ref().map(|s| s.is_empty()).unwrap_or(true);
                    if appid_empty && title_empty {
                        self.editing_window_rule = Some((idx, draft));
                        task = self.set_status(StatusMessage::Error(
                            "At least one matcher (App ID or Title) is required.".to_string(),
                        ));
                    } else {
                        match idx {
                            Some(i) => {
                                if let Some(existing) = self.rules_config.window_rules.get_mut(i) {
                                    *existing = draft;
                                }
                            }
                            None => {
                                self.rules_config.window_rules.push(draft);
                            }
                        }
                        self.editing_window_rule = None;
                    }
                }
            }
            Message::CancelWindowRule => {
                self.editing_window_rule = None;
                self.show_window_picker = false;
                self.available_windows = Vec::new();
            }
            Message::FormTabSelected(ft) => {
                self.form_tab = ft;
            }
            Message::WindowRuleChanged(field) => {
                if let Some((_, ref mut draft)) = self.editing_window_rule {
                    match field {
                        // Matchers
                        WindowRuleField::AppId(s) => draft.appid = if s.is_empty() { None } else { Some(s) },
                        WindowRuleField::Title(s) => draft.title = if s.is_empty() { None } else { Some(s) },
                        // State/Behavior
                        WindowRuleField::IsFloating(v) => draft.isfloating = v,
                        WindowRuleField::IsFullscreen(v) => draft.isfullscreen = v,
                        WindowRuleField::IsGlobal(v) => draft.isglobal = v,
                        WindowRuleField::IsOverlay(v) => draft.isoverlay = v,
                        WindowRuleField::IsOpenSilent(v) => draft.isopensilent = v,
                        WindowRuleField::IsTagSilent(v) => draft.istagsilent = v,
                        WindowRuleField::ForceMaximize(v) => draft.force_maximize = v,
                        WindowRuleField::IgnoreMaximize(v) => draft.ignore_maximize = v,
                        WindowRuleField::IgnoreMinimize(v) => draft.ignore_minimize = v,
                        WindowRuleField::ForceTileState(v) => draft.force_tile_state = v,
                        WindowRuleField::NoOpenMaximized(v) => draft.noopenmaximized = v,
                        WindowRuleField::SingleScratchpad(v) => draft.single_scratchpad = v,
                        WindowRuleField::AllowShortcutsInhibit(v) => draft.allow_shortcuts_inhibit = v,
                        WindowRuleField::IsFakeFullscreen(v) => draft.isfakefullscreen = v,
                        WindowRuleField::IndleInhibitWhenFocus(v) => draft.indleinhibit_when_focus = v,
                        // Geometry
                        WindowRuleField::Width(s) => draft.width = s.parse::<u32>().ok(),
                        WindowRuleField::Height(s) => draft.height = s.parse::<u32>().ok(),
                        WindowRuleField::OffsetX(s) => draft.offsetx = s.parse::<i32>().ok(),
                        WindowRuleField::OffsetY(s) => draft.offsety = s.parse::<i32>().ok(),
                        WindowRuleField::Monitor(s) => draft.monitor = if s.is_empty() { None } else { Some(s) },
                        WindowRuleField::Tags(s) => draft.tags = s.parse::<u8>().ok(),
                        WindowRuleField::NoForceCenter(v) => draft.no_force_center = v,
                        WindowRuleField::IsNoSizeHint(v) => draft.isnosizehint = v,
                        // Visuals
                        WindowRuleField::NoBlur(v) => draft.noblur = v,
                        WindowRuleField::IsNoBorder(v) => draft.isnoborder = v,
                        WindowRuleField::IsNoShadow(v) => draft.isnoshadow = v,
                        WindowRuleField::IsNoRadius(v) => draft.isnoradius = v,
                        WindowRuleField::IsNoAnimation(v) => draft.isnoanimation = v,
                        WindowRuleField::FocusedOpacity(s) => {
                            draft.focused_opacity = s.parse::<f32>().ok();
                            self.float_drafts.focused_opacity = s;
                        }
                        WindowRuleField::UnfocusedOpacity(s) => {
                            draft.unfocused_opacity = s.parse::<f32>().ok();
                            self.float_drafts.unfocused_opacity = s;
                        }
                        WindowRuleField::AllowCsd(v) => draft.allow_csd = v,
                        // Animation
                        WindowRuleField::AnimTypeOpen(v) => draft.animation_type_open = v,
                        WindowRuleField::AnimTypeClose(v) => draft.animation_type_close = v,
                        WindowRuleField::NoFadeIn(v) => draft.nofadein = v,
                        WindowRuleField::NoFadeOut(v) => draft.nofadeout = v,
                        // Layout
                        WindowRuleField::ScrollerProportion(s) => {
                            draft.scroller_proportion = s.parse::<f32>().ok();
                            self.float_drafts.scroller_proportion = s;
                        }
                        WindowRuleField::ScrollerProportionSingle(s) => {
                            draft.scroller_proportion_single = s.parse::<f32>().ok();
                            self.float_drafts.scroller_proportion_single = s;
                        }
                        // Terminal
                        WindowRuleField::IsTerm(v) => draft.isterm = v,
                        WindowRuleField::NoSwallow(v) => draft.noswallow = v,
                        // Special
                        WindowRuleField::GlobalKeyBinding(s) => draft.globalkeybinding = if s.is_empty() { None } else { Some(s) },
                        WindowRuleField::IsUnGlobal(v) => draft.isunglobal = v,
                        WindowRuleField::IsNamedScratchpad(v) => draft.isnamedscratchpad = v,
                        WindowRuleField::ForceTearing(v) => draft.force_tearing = v,
                    }
                }
            }
            Message::OpenWindowPicker => {
                self.available_windows = fetch_toplevels();
                self.show_window_picker = true;
            }
            Message::WindowSelected(app_id) => {
                if let Some((_, ref mut draft)) = self.editing_window_rule {
                    draft.appid = if app_id.is_empty() { None } else { Some(app_id) };
                }
                self.show_window_picker = false;
                self.available_windows = Vec::new();
            }
            Message::CancelWindowPicker => {
                self.show_window_picker = false;
                self.available_windows = Vec::new();
            }
            Message::AddTagRule => {
                self.float_drafts.mfact = String::new();
                self.editing_tag_rule = Some((None, crate::rules::TagRule::default()));
            }
            Message::EditTagRule(i) => {
                if let Some(rule) = self.rules_config.tag_rules.get(i) {
                    self.float_drafts.mfact = rule.mfact.map(|v| format!("{:.2}", v)).unwrap_or_default();
                    self.editing_tag_rule = Some((Some(i), rule.clone()));
                }
            }
            Message::SaveTagRule => {
                if let Some((idx, draft)) = self.editing_tag_rule.take() {
                    if draft.id.is_none() {
                        self.editing_tag_rule = Some((idx, draft));
                        task = self.set_status(StatusMessage::Error(
                            "Tag rule requires an ID".to_string(),
                        ));
                    } else {
                        match idx {
                            Some(i) => {
                                if let Some(existing) = self.rules_config.tag_rules.get_mut(i) {
                                    *existing = draft;
                                }
                            }
                            None => {
                                self.rules_config.tag_rules.push(draft);
                            }
                        }
                        self.editing_tag_rule = None;
                    }
                }
            }
            Message::CancelTagRule => {
                self.editing_tag_rule = None;
            }
            Message::TagRuleChanged(field) => {
                if let Some((_, ref mut draft)) = self.editing_tag_rule {
                    match field {
                        TagRuleField::Id(s) => draft.id = s.trim().parse::<u8>().ok(),
                        TagRuleField::LayoutName(s) => draft.layout_name = if s.is_empty() { None } else { Some(s) },
                        TagRuleField::Nmaster(s) => draft.nmaster = s.trim().parse::<u32>().ok(),
                        TagRuleField::Mfact(s) => {
                            draft.mfact = s.trim().parse::<f32>().ok();
                            self.float_drafts.mfact = s;
                        }
                        TagRuleField::MonitorName(s) => draft.monitor_name = if s.is_empty() { None } else { Some(s) },
                    }
                }
            }
            Message::AddLayerRule => {
                self.editing_layer_rule = Some((None, crate::rules::LayerRule::default()));
            }
            Message::EditLayerRule(i) => {
                if let Some(rule) = self.rules_config.layer_rules.get(i) {
                    self.editing_layer_rule = Some((Some(i), rule.clone()));
                }
            }
            Message::SaveLayerRule => {
                if let Some((idx, draft)) = self.editing_layer_rule.take() {
                    let name_empty = draft.layer_name.as_ref().map(|s| s.is_empty()).unwrap_or(true);
                    if name_empty {
                        self.editing_layer_rule = Some((idx, draft));
                        task = self.set_status(StatusMessage::Error(
                            "Layer rule requires a layer name".to_string(),
                        ));
                    } else {
                        match idx {
                            Some(i) => {
                                if let Some(existing) = self.rules_config.layer_rules.get_mut(i) {
                                    *existing = draft;
                                }
                            }
                            None => {
                                self.rules_config.layer_rules.push(draft);
                            }
                        }
                        self.editing_layer_rule = None;
                    }
                }
            }
            Message::CancelLayerRule => {
                self.editing_layer_rule = None;
            }
            Message::SaveRulesConfig => {
                let expanded = expand_tilde(&self.settings.rules_conf_path);
                match self.rules_config.save(&expanded) {
                    Ok(()) => {
                        // Update stored hash so the next startup doesn't treat
                        // our own save as an external modification.
                        self.settings.rules_conf_hash = crate::rules::hash_file(&expanded);
                        let _ = self.settings.save();
                        task = self.set_status(StatusMessage::Success(
                            format!("Rules saved to {}", self.settings.rules_conf_path)
                        ));
                    }
                    Err(e) => task = self.set_status(StatusMessage::Error(
                        format!("Save error: {}", e)
                    )),
                }
            }
            Message::LayerRuleChanged(field) => {
                if let Some((_, ref mut draft)) = self.editing_layer_rule {
                    match field {
                        LayerRuleField::LayerName(s) => draft.layer_name = if s.is_empty() { None } else { Some(s) },
                        LayerRuleField::NoBlur(v) => draft.noblur = v,
                        LayerRuleField::NoShadow(v) => draft.noshadow = v,
                        LayerRuleField::NoAnim(v) => draft.noanim = v,
                        LayerRuleField::AnimTypeOpen(v) => draft.animation_type_open = v,
                        LayerRuleField::AnimTypeClose(v) => draft.animation_type_close = v,
                    }
                }
            }
            Message::DismissStatus => {
                self.status_message = None;
            }
        }
        task
    }

    pub fn view(&self) -> Element<'_, Message> {
        let nav = self.nav_sidebar();
        let content = match self.current_view {
            View::Monitor => self.monitors_view(),
            View::Rules => self.rules_view(),
        };

        let main_row = row![nav, content].height(Length::Fill);
        let mut root = column![main_row];

        if let Some(ref status) = self.status_message {
            let (msg, style_fn): (&str, fn(&Theme) -> container::Style) = match status {
                StatusMessage::Success(s) => (s.as_str(), container::success),
                StatusMessage::Error(s) => (s.as_str(), container::danger),
            };
            let status_bar = container(text(msg).size(13))
                .width(Length::Fill)
                .padding([4, 10])
                .style(style_fn);
            root = root.push(status_bar);
        }

        root.into()
    }

    fn nav_sidebar(&self) -> Element<'_, Message> {
        let monitor_btn = button(iced_fonts::bootstrap::display().size(20))
            .on_press(Message::SwitchView(View::Monitor))
            .style(if self.current_view == View::Monitor { button::primary } else { button::secondary })
            .padding(12);

        let rules_btn = button(iced_fonts::bootstrap::file_earmark_text().size(20))
            .on_press(Message::SwitchView(View::Rules))
            .style(if self.current_view == View::Rules { button::primary } else { button::secondary })
            .padding(12);

        column![monitor_btn, rules_btn]
            .spacing(8)
            .padding(8)
            .width(Length::Fixed(56.0))
            .height(Length::Fill)
            .into()
    }

    fn monitors_view(&self) -> Element<'_, Message> {
        let canvas = Canvas::new(LayoutCanvas {
            outputs: self.outputs.clone(),
            selected_idx: self.selected_output_idx,
            cache: &self.layout_cache,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        let mut sidebar = column![].spacing(15).width(Length::Fixed(400.0));

        let mut tabs_row = row![].spacing(0);
        for (i, out) in self.outputs.iter().enumerate() {
            let is_selected = Some(i) == self.selected_output_idx;
            let current_btn = button(text(&out.name).align_x(alignment::Horizontal::Center))
                .width(Length::Fixed(80.0))
                .style(if is_selected {
                    button::primary
                } else {
                    button::secondary
                })
                .on_press(Message::MonitorClicked(i));
            tabs_row = tabs_row.push(current_btn);
        }
        sidebar = sidebar.push(container(tabs_row).center_x(Length::Fill));

        if let Some(idx) = self.selected_output_idx {
            if let Some(out) = self.outputs.get(idx) {
                if self.outputs.len() > 1 {
                    sidebar = sidebar.push(
                        row![
                            Space::new().width(100.0),
                            checkbox(out.enabled).on_toggle(Message::EnabledToggled),
                            text("Enabled")
                        ]
                        .spacing(10),
                    );
                }

                let label_width = 100.0;

                let row_desc = row![
                    container(text("Description").size(14)).width(label_width),
                    text(&out.description).size(14)
                ]
                .spacing(10)
                .align_y(alignment::Vertical::Center);
                sidebar = sidebar.push(row_desc);

                let phys_size_text = if out.physical_size.is_empty() {
                    "Unknown".to_string()
                } else {
                    out.physical_size.clone()
                };
                let row_phys = row![
                    container(text("Physical Size").size(14)).width(label_width),
                    text(phys_size_text).size(14)
                ]
                .spacing(10)
                .align_y(alignment::Vertical::Center);
                sidebar = sidebar.push(row_phys);

                let row_scale = row![
                    container(text("DPI Scale").size(14)).width(label_width),
                    text_input("", &self.scale_input)
                        .on_input(Message::ScaleChanged)
                        .width(Length::Fixed(60.0)),
                    button("-").on_press(Message::ScaleDec),
                    button("+").on_press(Message::ScaleInc),
                ]
                .spacing(5)
                .align_y(alignment::Vertical::Center);
                sidebar = sidebar.push(row_scale);

                let row_pos = row![
                    container(text("Position").size(14)).width(label_width),
                    text_input("", &self.x_input)
                        .on_input(Message::XChanged)
                        .width(Length::Fixed(60.0)),
                    button("-").on_press(Message::XDec),
                    button("+").on_press(Message::XInc),
                    text_input("", &self.y_input)
                        .on_input(Message::YChanged)
                        .width(Length::Fixed(60.0)),
                    button("-").on_press(Message::YDec),
                    button("+").on_press(Message::YInc),
                ]
                .spacing(5)
                .align_y(alignment::Vertical::Center);
                sidebar = sidebar.push(row_pos);

                let cm = out
                    .modes
                    .iter()
                    .find(|m| m.current)
                    .cloned()
                    .unwrap_or(OutputMode {
                        width: 1920,
                        height: 1080,
                        refresh_rate: 60.0,
                        current: true,
                        preferred: false,
                    });

                let mut unique_resolutions: Vec<String> = Vec::new();
                for m in &out.modes {
                    let res = format!("{}x{}", m.width, m.height);
                    if !unique_resolutions.contains(&res) {
                        unique_resolutions.push(res);
                    }
                }
                let selected_resolution = Some(format!("{}x{}", cm.width, cm.height));
                let res_options = unique_resolutions.clone();
                let pick_res = pick_list(res_options, selected_resolution, |s| {
                    Message::ResolutionSizeSelected(s)
                })
                .width(Length::Fixed(200.0));

                let row_res = row![
                    container(text("Resolution").size(14)).width(label_width),
                    pick_res
                ]
                .spacing(5)
                .align_y(alignment::Vertical::Center);
                sidebar = sidebar.push(row_res);

                let mut current_rr_idx = 0;
                let mut rr_labels = Vec::new();
                let mut rr_mode_indices = Vec::new();
                for (i, m) in out.modes.iter().enumerate() {
                    if m.width == cm.width && m.height == cm.height {
                        rr_labels.push(format!("{:.3}", m.refresh_rate));
                        rr_mode_indices.push(i);
                        if m.current {
                            current_rr_idx = rr_labels.len() - 1;
                        }
                    }
                }
                let rr_options = rr_labels.clone();
                let selected_rr = if current_rr_idx < rr_labels.len() {
                    Some(rr_labels[current_rr_idx].clone())
                } else {
                    None
                };
                let pick_rr = pick_list(rr_options, selected_rr, move |selected: String| {
                    let local_idx = rr_labels.iter().position(|r| *r == selected).unwrap_or(0);
                    let mode_idx = rr_mode_indices[local_idx];
                    Message::ResolutionSelected(mode_idx)
                })
                .width(Length::Fixed(100.0));

                let row_rr = row![
                    container(text("Refresh Rate").size(14)).width(label_width),
                    pick_rr,
                    text("Hz").size(14)
                ]
                .spacing(5)
                .align_y(alignment::Vertical::Center);
                sidebar = sidebar.push(row_rr);

                let transforms = vec![
                    "normal".to_string(),
                    "90".to_string(),
                    "180".to_string(),
                    "270".to_string(),
                    "flipped".to_string(),
                    "flipped-90".to_string(),
                    "flipped-180".to_string(),
                    "flipped-270".to_string(),
                ];
                let pick_trans = pick_list(transforms.clone(), Some(out.transform.clone()), |t| {
                    Message::TransformSelected(t)
                })
                .width(Length::Fixed(200.0));

                let row_trans = row![
                    container(text("Transform").size(14)).width(label_width),
                    pick_trans
                ]
                .spacing(5)
                .align_y(alignment::Vertical::Center);
                sidebar = sidebar.push(row_trans);
            }
        }

        let actions = row![
            button("Apply").on_press(Message::ApplyClicked),
            button("Save").on_press(Message::SaveClicked),
            button("Restore Default").on_press(Message::RestoreDefaultClicked),
        ]
        .spacing(10);

        sidebar = sidebar.push(Space::new().width(0.0).height(Length::Fill));
        sidebar = sidebar.push(actions);

        row![
            Container::new(canvas)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(container::dark),
            Container::new(Scrollable::new(sidebar).height(Length::Fill))
                .padding(20)
                .style(container::dark)
        ]
        .height(Length::Fill)
        .into()
    }

    fn rules_view(&self) -> Element<'_, Message> {
        let tab_bar = row![
            button("Window")
                .on_press(Message::SelectTab(Tab::Window))
                .style(if self.current_tab == Tab::Window { button::primary } else { button::secondary }),
            button("Tag")
                .on_press(Message::SelectTab(Tab::Tag))
                .style(if self.current_tab == Tab::Tag { button::primary } else { button::secondary }),
            button("Layer")
                .on_press(Message::SelectTab(Tab::Layer))
                .style(if self.current_tab == Tab::Layer { button::primary } else { button::secondary }),
            Space::new().width(Length::Fill),
            button("Save Rules").on_press(Message::SaveRulesConfig),
        ]
        .spacing(4);

        let tab_content = match self.current_tab {
            Tab::Window => self.window_rules_list(),
            Tab::Tag => self.tag_rules_list(),
            Tab::Layer => self.layer_rules_list(),
        };

        Container::new(
            column![tab_bar, tab_content]
                .spacing(12)
                .padding(20)
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(container::dark)
        .into()
    }

    fn window_rules_list(&self) -> Element<'_, Message> {
        // If editing, show form instead of list
        if let Some((ref idx, ref draft)) = self.editing_window_rule {
            return self.window_rule_form(draft, *idx);
        }

        let add_btn = button("Add Rule").on_press(Message::AddWindowRule);

        if self.rules_config.window_rules.is_empty() {
            return column![
                container(text("No window rules yet.").size(14)).padding(20),
                add_btn,
            ]
            .spacing(8)
            .into();
        }

        let rows: Vec<Element<'_, Message>> = self
            .rules_config
            .window_rules
            .iter()
            .enumerate()
            .map(|(i, rule)| {
                row![
                    text(rule.appid.as_deref().unwrap_or("*"))
                        .size(14)
                        .width(Length::Fixed(150.0)),
                    text(rule.title.as_deref().unwrap_or("*"))
                        .size(14)
                        .width(Length::Fill),
                    button("Edit")
                        .on_press(Message::EditWindowRule(i)),
                    button("Delete")
                        .on_press(Message::DeleteWindowRule(i))
                        .style(button::danger),
                ]
                .spacing(8)
                .align_y(alignment::Vertical::Center)
                .into()
            })
            .collect();

        column![
            scrollable(column(rows).spacing(4).width(Length::Fill)).height(Length::Fill),
            add_btn,
        ]
        .spacing(8)
        .into()
    }

    fn window_rule_form<'a>(&'a self, draft: &'a crate::rules::WindowRule, idx: Option<usize>) -> Element<'a, Message> {
        let title_str = if idx.is_none() { "Add Window Rule" } else { "Edit Window Rule" };
        let label_w = 170.0_f32;

        // Matchers section — always visible
        let pick_btn = button("Pick")
            .on_press(Message::OpenWindowPicker)
            .style(button::secondary);

        let appid_row = row![
            container(text("App ID").size(14)).width(Length::Fixed(label_w)),
            text_input("e.g. firefox", draft.appid.as_deref().unwrap_or(""))
                .on_input(|s| Message::WindowRuleChanged(WindowRuleField::AppId(s)))
                .width(Length::Fill),
            pick_btn,
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center);

        let title_row = row![
            container(text("Title").size(14)).width(Length::Fixed(label_w)),
            text_input("e.g. Mozilla Firefox", draft.title.as_deref().unwrap_or(""))
                .on_input(|s| Message::WindowRuleChanged(WindowRuleField::Title(s)))
                .width(Length::Fill),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center);

        let mut matchers_col = column![appid_row, title_row].spacing(8);

        if self.show_window_picker {
            let cancel_btn = button("Cancel")
                .on_press(Message::CancelWindowPicker)
                .style(button::secondary);

            let window_buttons: Vec<Element<'_, Message>> = self
                .available_windows
                .iter()
                .map(|w| {
                    let label = format!("{} ({})", w.title, w.app_id);
                    let app_id = w.app_id.clone();
                    button(text(label).size(13))
                        .on_press(Message::WindowSelected(app_id))
                        .style(button::secondary)
                        .width(Length::Fill)
                        .into()
                })
                .collect();

            let picker_content = if window_buttons.is_empty() {
                column![
                    text("No open windows found.").size(13),
                    cancel_btn,
                ]
                .spacing(6)
            } else {
                let mut col = column(window_buttons).spacing(4);
                col = col.push(cancel_btn);
                col
            };

            let picker_box = container(scrollable(picker_content).height(Length::Fixed(180.0)))
                .width(Length::Fill)
                .padding([6, 8])
                .style(container::bordered_box);

            matchers_col = matchers_col.push(picker_box);
        }

        let matchers = matchers_col;

        // Category tab bar
        let tab_bar = row![
            button("State").on_press(Message::FormTabSelected(FormTab::StateBehavior))
                .style(if self.form_tab == FormTab::StateBehavior { button::primary } else { button::secondary }),
            button("Geometry").on_press(Message::FormTabSelected(FormTab::Geometry))
                .style(if self.form_tab == FormTab::Geometry { button::primary } else { button::secondary }),
            button("Visuals").on_press(Message::FormTabSelected(FormTab::Visuals))
                .style(if self.form_tab == FormTab::Visuals { button::primary } else { button::secondary }),
            button("Animation").on_press(Message::FormTabSelected(FormTab::Animation))
                .style(if self.form_tab == FormTab::Animation { button::primary } else { button::secondary }),
            button("Layout").on_press(Message::FormTabSelected(FormTab::LayoutTerminal))
                .style(if self.form_tab == FormTab::LayoutTerminal { button::primary } else { button::secondary }),
            button("Special").on_press(Message::FormTabSelected(FormTab::Special))
                .style(if self.form_tab == FormTab::Special { button::primary } else { button::secondary }),
        ]
        .spacing(4);

        // Category content (scrollable)
        let category_content = scrollable(self.form_category_fields(draft))
            .height(Length::Fill);

        // Action buttons
        let actions = row![
            button("Save").on_press(Message::SaveWindowRule),
            button("Cancel").on_press(Message::CancelWindowRule),
        ]
        .spacing(8);

        column![
            text(title_str).size(16),
            matchers,
            tab_bar,
            category_content,
            actions,
        ]
        .spacing(12)
        .into()
    }

    fn form_category_fields<'a>(&'a self, draft: &'a crate::rules::WindowRule) -> Element<'a, Message> {
        match self.form_tab {
            FormTab::StateBehavior => self.form_fields_state(draft),
            FormTab::Geometry => self.form_fields_geometry(draft),
            FormTab::Visuals => self.form_fields_visuals(draft),
            FormTab::Animation => self.form_fields_animation(draft),
            FormTab::LayoutTerminal => self.form_fields_layout_terminal(draft),
            FormTab::Special => self.form_fields_special(draft),
        }
    }

    fn form_fields_state<'a>(&'a self, draft: &'a crate::rules::WindowRule) -> Element<'a, Message> {
        column![
            tri_row("Is Floating", "Force the window to float (not tiled)", draft.isfloating, |v| Message::WindowRuleChanged(WindowRuleField::IsFloating(v))),
            tri_row("Is Fullscreen", "Force the window to start fullscreen", draft.isfullscreen, |v| Message::WindowRuleChanged(WindowRuleField::IsFullscreen(v))),
            tri_row("Is Global", "Show on all tags", draft.isglobal, |v| Message::WindowRuleChanged(WindowRuleField::IsGlobal(v))),
            tri_row("Is Overlay", "Render above all other windows", draft.isoverlay, |v| Message::WindowRuleChanged(WindowRuleField::IsOverlay(v))),
            tri_row("Open Silent", "Open without stealing focus", draft.isopensilent, |v| Message::WindowRuleChanged(WindowRuleField::IsOpenSilent(v))),
            tri_row("Tag Silent", "Open on a tag without switching to it", draft.istagsilent, |v| Message::WindowRuleChanged(WindowRuleField::IsTagSilent(v))),
            tri_row("Force Maximize", "Maximize within its tile area", draft.force_maximize, |v| Message::WindowRuleChanged(WindowRuleField::ForceMaximize(v))),
            tri_row("Ignore Maximize", "Ignore the app's maximize requests", draft.ignore_maximize, |v| Message::WindowRuleChanged(WindowRuleField::IgnoreMaximize(v))),
            tri_row("Ignore Minimize", "Ignore the app's minimize requests", draft.ignore_minimize, |v| Message::WindowRuleChanged(WindowRuleField::IgnoreMinimize(v))),
            tri_row("Force Tile State", "Force tiled layout even if floating by default", draft.force_tile_state, |v| Message::WindowRuleChanged(WindowRuleField::ForceTileState(v))),
            tri_row("No Open Maximized", "Do not maximize when opened", draft.noopenmaximized, |v| Message::WindowRuleChanged(WindowRuleField::NoOpenMaximized(v))),
            tri_row("Single Scratchpad", "Only one instance shown at a time when used as scratchpad", draft.single_scratchpad, |v| Message::WindowRuleChanged(WindowRuleField::SingleScratchpad(v))),
            tri_row("Allow Shortcuts Inhibit", "Allow the app to capture keyboard shortcuts", draft.allow_shortcuts_inhibit, |v| Message::WindowRuleChanged(WindowRuleField::AllowShortcutsInhibit(v))),
            tri_row("Fake Fullscreen", "Fake fullscreen — window thinks it's fullscreen but renders in tile", draft.isfakefullscreen, |v| Message::WindowRuleChanged(WindowRuleField::IsFakeFullscreen(v))),
            tri_row("Idle Inhibit on Focus", "", draft.indleinhibit_when_focus, |v| Message::WindowRuleChanged(WindowRuleField::IndleInhibitWhenFocus(v))),
        ]
        .spacing(6)
        .into()
    }

    fn form_fields_geometry<'a>(&'a self, draft: &'a crate::rules::WindowRule) -> Element<'a, Message> {
        column![
            str_row("Width (px)", "Initial width in pixels", &draft.width.map(|v| v.to_string()).unwrap_or_default(),
                |s| Message::WindowRuleChanged(WindowRuleField::Width(s))),
            str_row("Height (px)", "Initial height in pixels", &draft.height.map(|v| v.to_string()).unwrap_or_default(),
                |s| Message::WindowRuleChanged(WindowRuleField::Height(s))),
            str_row("Offset X", "Horizontal position offset in pixels", &draft.offsetx.map(|v| v.to_string()).unwrap_or_default(),
                |s| Message::WindowRuleChanged(WindowRuleField::OffsetX(s))),
            str_row("Offset Y", "Vertical position offset in pixels", &draft.offsety.map(|v| v.to_string()).unwrap_or_default(),
                |s| Message::WindowRuleChanged(WindowRuleField::OffsetY(s))),
            str_row("Monitor", "Force window to open on this monitor (by name)", draft.monitor.as_deref().unwrap_or(""),
                |s| Message::WindowRuleChanged(WindowRuleField::Monitor(s))),
            str_row("Tags (bitmask)", "Bitmask of tags to open on (e.g. 1 = tag 1, 3 = tags 1&2)", &draft.tags.map(|v| v.to_string()).unwrap_or_default(),
                |s| Message::WindowRuleChanged(WindowRuleField::Tags(s))),
            tri_row("No Force Center", "Do not center the floating window on open", draft.no_force_center, |v| Message::WindowRuleChanged(WindowRuleField::NoForceCenter(v))),
            tri_row("No Size Hint", "Ignore size hints from the application", draft.isnosizehint, |v| Message::WindowRuleChanged(WindowRuleField::IsNoSizeHint(v))),
        ]
        .spacing(6)
        .into()
    }

    fn form_fields_visuals<'a>(&'a self, draft: &'a crate::rules::WindowRule) -> Element<'a, Message> {
        column![
            tri_row("No Blur", "Disable background blur for this window", draft.noblur, |v| Message::WindowRuleChanged(WindowRuleField::NoBlur(v))),
            tri_row("No Border", "Remove the window border", draft.isnoborder, |v| Message::WindowRuleChanged(WindowRuleField::IsNoBorder(v))),
            tri_row("No Shadow", "Remove the window shadow", draft.isnoshadow, |v| Message::WindowRuleChanged(WindowRuleField::IsNoShadow(v))),
            tri_row("No Radius", "Remove rounded corners", draft.isnoradius, |v| Message::WindowRuleChanged(WindowRuleField::IsNoRadius(v))),
            tri_row("No Animation", "Disable animations for this window", draft.isnoanimation, |v| Message::WindowRuleChanged(WindowRuleField::IsNoAnimation(v))),
            str_row("Focused Opacity (0.0-1.0)", "Opacity when focused (0.0–1.0)", &self.float_drafts.focused_opacity,
                |s| Message::WindowRuleChanged(WindowRuleField::FocusedOpacity(s))),
            str_row("Unfocused Opacity (0.0-1.0)", "Opacity when not focused (0.0–1.0)", &self.float_drafts.unfocused_opacity,
                |s| Message::WindowRuleChanged(WindowRuleField::UnfocusedOpacity(s))),
            tri_row("Allow CSD", "Allow client-side decorations (app draws its own titlebar)", draft.allow_csd, |v| Message::WindowRuleChanged(WindowRuleField::AllowCsd(v))),
        ]
        .spacing(6)
        .into()
    }

    fn form_fields_animation<'a>(&'a self, draft: &'a crate::rules::WindowRule) -> Element<'a, Message> {
        let open_anim = AnimTypePick::from_opt(&draft.animation_type_open);
        let close_anim = AnimTypePick::from_opt(&draft.animation_type_close);

        let mut open_row = row![
            container(text("Open Animation").size(14)).width(Length::Fixed(170.0)),
            pick_list(
                ANIM_TYPE_OPTS,
                Some(open_anim),
                |ap: AnimTypePick| Message::WindowRuleChanged(WindowRuleField::AnimTypeOpen(ap.to_opt())),
            ).width(Length::Fixed(120.0)),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center);
        if let Some(tip) = info_tip("Animation style when the window opens") {
            open_row = open_row.push(tip);
        }

        let mut close_row = row![
            container(text("Close Animation").size(14)).width(Length::Fixed(170.0)),
            pick_list(
                ANIM_TYPE_OPTS,
                Some(close_anim),
                |ap: AnimTypePick| Message::WindowRuleChanged(WindowRuleField::AnimTypeClose(ap.to_opt())),
            ).width(Length::Fixed(120.0)),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center);
        if let Some(tip) = info_tip("Animation style when the window closes") {
            close_row = close_row.push(tip);
        }

        column![
            open_row,
            close_row,
            tri_row("No Fade In", "Skip fade-in on open", draft.nofadein, |v| Message::WindowRuleChanged(WindowRuleField::NoFadeIn(v))),
            tri_row("No Fade Out", "Skip fade-out on close", draft.nofadeout, |v| Message::WindowRuleChanged(WindowRuleField::NoFadeOut(v))),
        ]
        .spacing(6)
        .into()
    }

    fn form_fields_layout_terminal<'a>(&'a self, draft: &'a crate::rules::WindowRule) -> Element<'a, Message> {
        column![
            str_row("Scroller Proportion", "Width proportion in scroller layout (0.0–1.0)", &self.float_drafts.scroller_proportion,
                |s| Message::WindowRuleChanged(WindowRuleField::ScrollerProportion(s))),
            str_row("Scroller Proportion (single)", "Width proportion when the only window in scroller (0.0–1.0)", &self.float_drafts.scroller_proportion_single,
                |s| Message::WindowRuleChanged(WindowRuleField::ScrollerProportionSingle(s))),
            tri_row("Is Terminal", "Mark as terminal (used for swallowing)", draft.isterm, |v| Message::WindowRuleChanged(WindowRuleField::IsTerm(v))),
            tri_row("No Swallow", "Prevent this window from being swallowed by a terminal", draft.noswallow, |v| Message::WindowRuleChanged(WindowRuleField::NoSwallow(v))),
        ]
        .spacing(6)
        .into()
    }

    fn form_fields_special<'a>(&'a self, draft: &'a crate::rules::WindowRule) -> Element<'a, Message> {
        column![
            str_row("Global Keybinding", "Keybinding to toggle this window globally (e.g. ctrl+alt-o)", draft.globalkeybinding.as_deref().unwrap_or(""),
                |s| Message::WindowRuleChanged(WindowRuleField::GlobalKeyBinding(s))),
            tri_row("Is Un-Global", "Remove any global keybinding from this window", draft.isunglobal, |v| Message::WindowRuleChanged(WindowRuleField::IsUnGlobal(v))),
            tri_row("Named Scratchpad", "Register as a named scratchpad", draft.isnamedscratchpad, |v| Message::WindowRuleChanged(WindowRuleField::IsNamedScratchpad(v))),
            tri_row("Force Tearing", "Allow screen tearing for this window (lower latency for games)", draft.force_tearing, |v| Message::WindowRuleChanged(WindowRuleField::ForceTearing(v))),
        ]
        .spacing(6)
        .into()
    }

    fn tag_rules_list(&self) -> Element<'_, Message> {
        if let Some((ref idx, ref draft)) = self.editing_tag_rule {
            return self.tag_rule_form(draft, *idx);
        }

        let add_btn = button("Add Rule").on_press(Message::AddTagRule);

        if self.rules_config.tag_rules.is_empty() {
            return column![
                container(text("No tag rules yet.").size(14)).padding(20),
                add_btn,
            ]
            .spacing(8)
            .into();
        }

        let rows: Vec<Element<'_, Message>> = self
            .rules_config
            .tag_rules
            .iter()
            .enumerate()
            .map(|(i, rule)| {
                let id_str = rule
                    .id
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "*".to_string());
                row![
                    text(id_str).size(14).width(Length::Fixed(80.0)),
                    Space::new().width(Length::Fill),
                    button("Edit")
                        .on_press(Message::EditTagRule(i)),
                    button("Delete")
                        .on_press(Message::DeleteTagRule(i))
                        .style(button::danger),
                ]
                .spacing(8)
                .align_y(alignment::Vertical::Center)
                .into()
            })
            .collect();

        column![
            scrollable(column(rows).spacing(4).width(Length::Fill)).height(Length::Fill),
            add_btn,
        ]
        .spacing(8)
        .into()
    }

    fn tag_rule_form<'a>(&'a self, draft: &'a crate::rules::TagRule, idx: Option<usize>) -> Element<'a, Message> {
        let title_str = if idx.is_some() { "Edit Tag Rule" } else { "Add Tag Rule" };

        let form_rows = column![
            str_row("Tag ID (0-255)", "Tag number (0-based index)", &draft.id.map(|n| n.to_string()).unwrap_or_default(),
                |s| Message::TagRuleChanged(TagRuleField::Id(s))),
            str_row("Layout Name", "Default layout for this tag (e.g. tile, monocle, dwindle)", draft.layout_name.as_deref().unwrap_or(""),
                |s| Message::TagRuleChanged(TagRuleField::LayoutName(s))),
            str_row("Nmaster", "Number of windows in the master area", &draft.nmaster.map(|n| n.to_string()).unwrap_or_default(),
                |s| Message::TagRuleChanged(TagRuleField::Nmaster(s))),
            str_row("Mfact (0.0-1.0)", "Master area size ratio (0.0–1.0)", &self.float_drafts.mfact,
                |s| Message::TagRuleChanged(TagRuleField::Mfact(s))),
            str_row("Monitor Name", "Assign this tag to a specific monitor", draft.monitor_name.as_deref().unwrap_or(""),
                |s| Message::TagRuleChanged(TagRuleField::MonitorName(s))),
        ]
        .spacing(8);

        let save_cancel_row = row![
            button("Save").on_press(Message::SaveTagRule),
            button("Cancel").on_press(Message::CancelTagRule),
        ]
        .spacing(8);

        scrollable(
            column![
                text(title_str).size(16),
                form_rows,
                save_cancel_row,
            ]
            .spacing(12)
            .padding(20)
        )
        .into()
    }

    fn layer_rules_list(&self) -> Element<'_, Message> {
        if let Some((ref idx, ref draft)) = self.editing_layer_rule {
            return self.layer_rule_form(draft, *idx);
        }

        let add_btn = button("Add Rule").on_press(Message::AddLayerRule);

        if self.rules_config.layer_rules.is_empty() {
            return column![
                container(text("No layer rules yet.").size(14)).padding(20),
                add_btn,
            ]
            .spacing(8)
            .into();
        }

        let rows: Vec<Element<'_, Message>> = self
            .rules_config
            .layer_rules
            .iter()
            .enumerate()
            .map(|(i, rule)| {
                row![
                    text(rule.layer_name.as_deref().unwrap_or("*"))
                        .size(14)
                        .width(Length::Fill),
                    button("Edit")
                        .on_press(Message::EditLayerRule(i)),
                    button("Delete")
                        .on_press(Message::DeleteLayerRule(i))
                        .style(button::danger),
                ]
                .spacing(8)
                .align_y(alignment::Vertical::Center)
                .into()
            })
            .collect();

        column![
            scrollable(column(rows).spacing(4).width(Length::Fill)).height(Length::Fill),
            add_btn,
        ]
        .spacing(8)
        .into()
    }

    fn layer_rule_form<'a>(&'a self, draft: &'a crate::rules::LayerRule, idx: Option<usize>) -> Element<'a, Message> {
        let title_str = if idx.is_some() { "Edit Layer Rule" } else { "Add Layer Rule" };

        let anim_open = AnimTypePick::from_opt(&draft.animation_type_open);
        let anim_close = AnimTypePick::from_opt(&draft.animation_type_close);

        let mut open_row = row![
            container(text("Anim Type Open").size(14)).width(Length::Fixed(170.0)),
            pick_list(
                ANIM_TYPE_OPTS,
                Some(anim_open),
                |ap: AnimTypePick| Message::LayerRuleChanged(LayerRuleField::AnimTypeOpen(ap.to_opt())),
            ).width(Length::Fixed(120.0)),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center);
        if let Some(tip) = info_tip("Animation style when the layer opens") {
            open_row = open_row.push(tip);
        }
        let open_row: Element<'_, Message> = open_row.into();

        let mut close_row = row![
            container(text("Anim Type Close").size(14)).width(Length::Fixed(170.0)),
            pick_list(
                ANIM_TYPE_OPTS,
                Some(anim_close),
                |ap: AnimTypePick| Message::LayerRuleChanged(LayerRuleField::AnimTypeClose(ap.to_opt())),
            ).width(Length::Fixed(120.0)),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center);
        if let Some(tip) = info_tip("Animation style when the layer closes") {
            close_row = close_row.push(tip);
        }
        let close_row: Element<'_, Message> = close_row.into();

        let form_rows = column![
            str_row("Layer Name (required)", "Name of the layer surface to match (e.g. waybar, rofi)", draft.layer_name.as_deref().unwrap_or(""),
                |s| Message::LayerRuleChanged(LayerRuleField::LayerName(s))),
            tri_row("No Blur", "Disable blur behind this layer", draft.noblur, |v| Message::LayerRuleChanged(LayerRuleField::NoBlur(v))),
            tri_row("No Shadow", "Disable shadow for this layer", draft.noshadow, |v| Message::LayerRuleChanged(LayerRuleField::NoShadow(v))),
            tri_row("No Animation", "Disable animations for this layer", draft.noanim, |v| Message::LayerRuleChanged(LayerRuleField::NoAnim(v))),
            open_row,
            close_row,
        ]
        .spacing(8);

        let save_cancel_row = row![
            button("Save").on_press(Message::SaveLayerRule),
            button("Cancel").on_press(Message::CancelLayerRule),
        ]
        .spacing(8);

        scrollable(
            column![
                text(title_str).size(16),
                form_rows,
                save_cancel_row,
            ]
            .spacing(12)
            .padding(20)
        )
        .into()
    }
}

#[derive(Default)]
pub struct CanvasState {
    dragging: Option<(usize, Point, (i32, i32))>,
    hovered: Option<usize>,
}

struct LayoutCanvas<'a> {
    outputs: Vec<Output>,
    selected_idx: Option<usize>,
    cache: &'a Cache,
}

impl<'a> LayoutCanvas<'a> {
    fn logical_size(out: &Output, cm: &OutputMode) -> (i32, i32) {
        let w = (cm.width as f32 / out.scale) as i32;
        let h = (cm.height as f32 / out.scale) as i32;
        match out.transform.as_str() {
            "90" | "270" | "flipped-90" | "flipped-270" => (h, w),
            _ => (w, h),
        }
    }

    fn calculate_layout(&self, bounds: Rectangle) -> (f32, f32, f32, i32, i32) {
        let mut total_w = 0;
        let mut max_h = 1080;

        for out in &self.outputs {
            let cm = out
                .modes
                .iter()
                .find(|m| m.current)
                .cloned()
                .unwrap_or(OutputMode {
                    width: 800,
                    height: 600,
                    refresh_rate: 60.0,
                    current: true,
                    preferred: false,
                });
            let (w_logical, h_logical) = Self::logical_size(out, &cm);
            total_w += w_logical;
            if h_logical > max_h {
                max_h = h_logical;
            }
        }

        let span_x = (total_w as f32 * 1.5).max(4000.0);
        let span_y = (max_h as f32 * 2.5).max(3000.0);
        let scale = (bounds.width / span_x).min(bounds.height / span_y);

        let first_w = self
            .outputs
            .first()
            .map(|out| {
                let cm = out
                    .modes
                    .iter()
                    .find(|m| m.current)
                    .cloned()
                    .unwrap_or(OutputMode {
                        width: 800,
                        height: 600,
                        refresh_rate: 60.0,
                        current: true,
                        preferred: false,
                    });
                Self::logical_size(out, &cm).0 as f32
            })
            .unwrap_or(1920.0);

        let offset_x = bounds.width / 2.0 - (first_w / 2.0) * scale;
        let offset_y = bounds.height / 2.0 - (max_h as f32 / 2.0) * scale;

        (scale, offset_x, offset_y, 0, 0)
    }

    fn transformed_geometry(
        &self,
        out: &Output,
        scale: f32,
        offset_x: f32,
        offset_y: f32,
        min_x: i32,
        min_y: i32,
    ) -> (f32, f32, f32, f32) {
        let cm = out
            .modes
            .iter()
            .find(|m| m.current)
            .cloned()
            .unwrap_or(OutputMode {
                width: 800,
                height: 600,
                refresh_rate: 60.0,
                current: true,
                preferred: false,
            });
        let (w_logical, h_logical) = Self::logical_size(out, &cm);

        let w = w_logical as f32 * scale;
        let h = h_logical as f32 * scale;
        let x = (out.position.0 - min_x) as f32 * scale + offset_x;
        let y = (out.position.1 - min_y) as f32 * scale + offset_y;

        (x, y, w, h)
    }
}

impl<'a> Program<Message> for LayoutCanvas<'a> {
    type State = CanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        let (scale, offset_x, offset_y, min_x, min_y) = self.calculate_layout(bounds);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    for (i, out) in self.outputs.iter().enumerate() {
                        let (x, y, w, h) =
                            self.transformed_geometry(out, scale, offset_x, offset_y, min_x, min_y);

                        let rect = Rectangle::new(Point::new(x, y), Size::new(w, h));
                        if rect.contains(cursor_position) {
                            state.dragging = Some((i, cursor_position, out.position));
                        }
                    }
                    if let Some((i, _, _)) = state.dragging {
                        return Some(Action::publish(Message::MonitorClicked(i)).and_capture());
                    }
                    state.dragging = None;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.dragging = None;
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if let Some((idx, start_cursor, start_logical)) = state.dragging {
                    let delta_x = (position.x - start_cursor.x) / scale;
                    let delta_y = (position.y - start_cursor.y) / scale;

                    let mut new_x = start_logical.0 + delta_x.round() as i32;
                    let mut new_y = start_logical.1 + delta_y.round() as i32;
                    if new_x < 0 {
                        new_x = 0;
                    }
                    if new_y < 0 {
                        new_y = 0;
                    }

                    let snap_threshold = 40;

                    if let Some(out) = self.outputs.get(idx) {
                        let cm = out
                            .modes
                            .iter()
                            .find(|m| m.current)
                            .cloned()
                            .unwrap_or(OutputMode {
                                width: 800,
                                height: 600,
                                refresh_rate: 60.0,
                                current: true,
                                preferred: false,
                            });
                        let (w, h) = Self::logical_size(out, &cm);

                        let mut snapped_x = new_x;
                        let mut snapped_y = new_y;
                        let mut min_dist_x = snap_threshold;
                        let mut min_dist_y = snap_threshold;

                        let my_left = new_x;
                        let my_right = new_x + w;
                        let my_top = new_y;
                        let my_bottom = new_y + h;

                        for (i, other) in self.outputs.iter().enumerate() {
                            if i == idx {
                                continue;
                            }
                            let other_cm = other
                                .modes
                                .iter()
                                .find(|m| m.current)
                                .cloned()
                                .unwrap_or(OutputMode {
                                    width: 800,
                                    height: 600,
                                    refresh_rate: 60.0,
                                    current: true,
                                    preferred: false,
                                });
                            let (other_w, other_h) = Self::logical_size(other, &other_cm);

                            let other_left = other.position.0;
                            let other_right = other.position.0 + other_w;
                            let other_top = other.position.1;
                            let other_bottom = other.position.1 + other_h;

                            let x_overlap = my_left < other_right + snap_threshold
                                && my_right > other_left - snap_threshold;
                            let y_overlap = my_top < other_bottom + snap_threshold
                                && my_bottom > other_top - snap_threshold;

                            if y_overlap {
                                if (my_left - other_right).abs() < min_dist_x {
                                    min_dist_x = (my_left - other_right).abs();
                                    snapped_x = other_right;
                                }
                                if (my_right - other_left).abs() < min_dist_x {
                                    min_dist_x = (my_right - other_left).abs();
                                    snapped_x = other_left - w;
                                }
                                if (my_left - other_left).abs() < min_dist_x {
                                    min_dist_x = (my_left - other_left).abs();
                                    snapped_x = other_left;
                                }
                            }

                            if x_overlap {
                                if (my_top - other_bottom).abs() < min_dist_y {
                                    min_dist_y = (my_top - other_bottom).abs();
                                    snapped_y = other_bottom;
                                }
                                if (my_bottom - other_top).abs() < min_dist_y {
                                    min_dist_y = (my_bottom - other_top).abs();
                                    snapped_y = other_top - h;
                                }
                                if (my_top - other_top).abs() < min_dist_y {
                                    min_dist_y = (my_top - other_top).abs();
                                    snapped_y = other_top;
                                }
                            }
                        }

                        if snapped_x == new_x {
                            snapped_x = (snapped_x as f32 / 10.0).round() as i32 * 10;
                        }
                        if snapped_y == new_y {
                            snapped_y = (snapped_y as f32 / 10.0).round() as i32 * 10;
                        }

                        if snapped_x < 0 {
                            snapped_x = 0;
                        }
                        if snapped_y < 0 {
                            snapped_y = 0;
                        }

                        return Some(Action::publish(Message::MonitorPositioned(
                            idx, snapped_x, snapped_y,
                        )));
                    }
                } else {
                    let mut new_hovered = None;
                    for (i, out) in self.outputs.iter().enumerate() {
                        let (x, y, w, h) =
                            self.transformed_geometry(out, scale, offset_x, offset_y, min_x, min_y);
                        let rect = Rectangle::new(Point::new(x, y), Size::new(w, h));
                        if rect.contains(*position) {
                            new_hovered = Some(i);
                        }
                    }
                    if state.hovered != new_hovered {
                        state.hovered = new_hovered;
                        self.cache.clear();
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::from_rgb8(15, 15, 15));

            let (scale, offset_x, offset_y, min_x, min_y) = self.calculate_layout(bounds);

            for (i, out) in self.outputs.iter().enumerate() {
                let (x, y, w, h) =
                    self.transformed_geometry(out, scale, offset_x, offset_y, min_x, min_y);

                let rect = Rectangle::new(Point::new(x, y), Size::new(w, h));

                let is_selected = Some(i) == self.selected_idx;
                let is_hovered = Some(i) == state.hovered;

                let fill_color = if is_selected {
                    Color::from_rgb8(220, 220, 220)
                } else if is_hovered {
                    Color::from_rgb8(60, 60, 60)
                } else {
                    Color::from_rgb8(35, 35, 35)
                };

                let stroke_color = if is_selected {
                    Color::from_rgb8(255, 255, 255)
                } else if is_hovered {
                    Color::from_rgb8(150, 150, 150)
                } else {
                    Color::from_rgb8(20, 20, 20)
                };

                frame.fill_rectangle(rect.position(), rect.size(), fill_color);

                frame.stroke(
                    &Path::rectangle(rect.position(), rect.size()),
                    canvas::Stroke::default()
                        .with_color(stroke_color)
                        .with_width(if is_selected { 3.0 } else { 2.0 }),
                );

                let text_x = x + 16.0;
                let mut text_y = y + 16.0;
                let font_scale = scale.min(2.0).max(0.5);

                let mut name_text = canvas::Text::default();
                name_text.content = out.name.clone();
                name_text.position = Point::new(text_x, text_y);
                name_text.size = iced::Pixels(48.0 * font_scale);
                name_text.color = if is_selected {
                    Color::BLACK
                } else {
                    Color::from_rgb8(230, 230, 230)
                };
                frame.fill_text(name_text);

                text_y += 50.0 * font_scale;

                let text_size = 18.0 * font_scale;
                let approx_char_width = text_size * 0.6;
                let max_chars = ((w - 32.0) / approx_char_width).max(10.0) as usize;

                let mut lines = Vec::new();
                let mut current_line = String::new();

                for word in out.description.split_whitespace() {
                    if current_line.len() + word.len() + 1 > max_chars && !current_line.is_empty() {
                        lines.push(current_line);
                        current_line = word.to_string();
                    } else {
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(word);
                    }
                }
                if !current_line.is_empty() {
                    lines.push(current_line);
                }

                for line in lines {
                    let mut desc_text = canvas::Text::default();
                    desc_text.content = line;
                    desc_text.position = Point::new(text_x, text_y);
                    desc_text.size = iced::Pixels(text_size);
                    desc_text.color = if is_selected {
                        Color::from_rgb8(40, 40, 40)
                    } else {
                        Color::from_rgb8(160, 160, 160)
                    };
                    frame.fill_text(desc_text);
                    text_y += text_size * 1.3;
                }
            }
        });

        vec![geometry]
    }
}
