use std::fmt::Display;

/// Types of values an ARIA attribute can accept.
#[derive(Debug, Clone)]
pub enum AriaValueType {
    /// "true" or "false"
    Bool,
    /// "true", "false", or "mixed"
    TristateBool,
    /// "true", "false", or "undefined"
    BoolOrUndefined,
    /// One of a fixed set of string values
    Enum(&'static [&'static str]),
    /// An integer number
    Integer,
    /// A floating-point number
    Number,
    /// An ID reference
    IdRef,
    /// A space-separated list of ID references
    IdRefList,
    /// Freeform text (always valid)
    FreeText,
}

impl AriaValueType {
    /// Check whether a static value is valid for this type.
    pub fn is_valid(&self, value: &str) -> bool {
        match self {
            AriaValueType::Bool => matches!(value, "true" | "false"),
            AriaValueType::TristateBool => matches!(value, "true" | "false" | "mixed"),
            AriaValueType::BoolOrUndefined => {
                matches!(value, "true" | "false" | "undefined")
            }
            AriaValueType::Enum(variants) => variants.contains(&value),
            AriaValueType::Integer => value.parse::<i64>().is_ok(),
            AriaValueType::Number => value.parse::<f64>().is_ok(),
            AriaValueType::IdRef | AriaValueType::IdRefList | AriaValueType::FreeText => true,
        }
    }

    /// Return a human-readable description of the expected values.
    pub fn expected_description(&self) -> String {
        match self {
            AriaValueType::Bool => "\"true\" or \"false\"".to_string(),
            AriaValueType::TristateBool => "\"true\", \"false\", or \"mixed\"".to_string(),
            AriaValueType::BoolOrUndefined => "\"true\", \"false\", or \"undefined\"".to_string(),
            AriaValueType::Enum(variants) => {
                use std::fmt::Write;
                let mut out = String::from("one of: ");
                for (i, v) in variants.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    write!(out, "\"{}\"", v).unwrap();
                }
                out
            }
            AriaValueType::Integer => "an integer".to_string(),
            AriaValueType::Number => "a number".to_string(),
            AriaValueType::IdRef => "an element ID reference".to_string(),
            AriaValueType::IdRefList => {
                "a space-separated list of element ID references".to_string()
            }
            AriaValueType::FreeText => "any text".to_string(),
        }
    }
}

/// WAI-ARIA attribute names (e.g. `aria-checked`, `aria-label`).
///
/// Each variant maps to a specific `aria-*` attribute as defined in the
/// [WAI-ARIA 1.2 specification](https://www.w3.org/TR/wai-aria-1.2/#state_prop_def).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum Aria {
    #[serde(rename = "aria-activedescendant")]
    ActiveDescendant,
    #[serde(rename = "aria-atomic")]
    Atomic,
    #[serde(rename = "aria-autocomplete")]
    Autocomplete,
    #[serde(rename = "aria-braillelabel")]
    BrailleLabel,
    #[serde(rename = "aria-brailleroledescription")]
    BrailleRoleDescription,
    #[serde(rename = "aria-busy")]
    Busy,
    #[serde(rename = "aria-checked")]
    Checked,
    #[serde(rename = "aria-colcount")]
    ColCount,
    #[serde(rename = "aria-colindex")]
    ColIndex,
    #[serde(rename = "aria-colindextext")]
    ColIndexText,
    #[serde(rename = "aria-colspan")]
    ColSpan,
    #[serde(rename = "aria-controls")]
    Controls,
    #[serde(rename = "aria-current")]
    Current,
    #[serde(rename = "aria-describedby")]
    DescribedBy,
    #[serde(rename = "aria-description")]
    Description,
    #[serde(rename = "aria-details")]
    Details,
    #[serde(rename = "aria-disabled")]
    Disabled,
    #[serde(rename = "aria-dropeffect")]
    DropEffect,
    #[serde(rename = "aria-errormessage")]
    ErrorMessage,
    #[serde(rename = "aria-expanded")]
    Expanded,
    #[serde(rename = "aria-flowto")]
    FlowTo,
    #[serde(rename = "aria-grabbed")]
    Grabbed,
    #[serde(rename = "aria-haspopup")]
    HasPopup,
    #[serde(rename = "aria-hidden")]
    Hidden,
    #[serde(rename = "aria-invalid")]
    Invalid,
    #[serde(rename = "aria-keyshortcuts")]
    KeyShortcuts,
    #[serde(rename = "aria-label")]
    Label,
    #[serde(rename = "aria-labelledby")]
    LabelledBy,
    #[serde(rename = "aria-level")]
    Level,
    #[serde(rename = "aria-live")]
    Live,
    #[serde(rename = "aria-modal")]
    Modal,
    #[serde(rename = "aria-multiline")]
    Multiline,
    #[serde(rename = "aria-multiselectable")]
    Multiselectable,
    #[serde(rename = "aria-orientation")]
    Orientation,
    #[serde(rename = "aria-owns")]
    Owns,
    #[serde(rename = "aria-placeholder")]
    Placeholder,
    #[serde(rename = "aria-posinset")]
    PosInSet,
    #[serde(rename = "aria-pressed")]
    Pressed,
    #[serde(rename = "aria-readonly")]
    ReadOnly,
    #[serde(rename = "aria-relevant")]
    Relevant,
    #[serde(rename = "aria-required")]
    Required,
    #[serde(rename = "aria-roledescription")]
    RoleDescription,
    #[serde(rename = "aria-rowcount")]
    RowCount,
    #[serde(rename = "aria-rowindex")]
    RowIndex,
    #[serde(rename = "aria-rowindextext")]
    RowIndexText,
    #[serde(rename = "aria-rowspan")]
    RowSpan,
    #[serde(rename = "aria-selected")]
    Selected,
    #[serde(rename = "aria-setsize")]
    SetSize,
    #[serde(rename = "aria-sort")]
    Sort,
    #[serde(rename = "aria-valuemax")]
    ValueMax,
    #[serde(rename = "aria-valuemin")]
    ValueMin,
    #[serde(rename = "aria-valuenow")]
    ValueNow,
    #[serde(rename = "aria-valuetext")]
    ValueText,
}

impl Aria {
    pub const fn value_type(&self) -> AriaValueType {
        match self {
            Aria::Autocomplete => AriaValueType::Enum(&["inline", "list", "both", "none"]),
            Aria::Checked => AriaValueType::TristateBool,
            Aria::Current => {
                AriaValueType::Enum(&["page", "step", "location", "date", "time", "true", "false"])
            }
            Aria::Disabled => AriaValueType::Bool,
            Aria::Expanded => AriaValueType::BoolOrUndefined,
            Aria::Grabbed => AriaValueType::BoolOrUndefined,
            Aria::HasPopup => {
                AriaValueType::Enum(&["true", "false", "menu", "listbox", "tree", "grid", "dialog"])
            }
            Aria::Hidden => AriaValueType::BoolOrUndefined,
            Aria::Invalid => AriaValueType::Enum(&["true", "false", "grammar", "spelling"]),
            Aria::Live => AriaValueType::Enum(&["assertive", "off", "polite"]),
            Aria::Modal => AriaValueType::Bool,
            Aria::Multiline => AriaValueType::Bool,
            Aria::Multiselectable => AriaValueType::Bool,
            Aria::Orientation => AriaValueType::Enum(&["horizontal", "vertical", "undefined"]),
            Aria::Pressed => AriaValueType::TristateBool,
            Aria::ReadOnly => AriaValueType::Bool,
            Aria::Relevant => {
                AriaValueType::Enum(&["additions", "additions text", "all", "removals", "text"])
            }
            Aria::Required => AriaValueType::Bool,
            Aria::Selected => AriaValueType::BoolOrUndefined,
            Aria::Sort => AriaValueType::Enum(&["ascending", "descending", "none", "other"]),
            Aria::Atomic => AriaValueType::Bool,
            Aria::Busy => AriaValueType::Bool,
            // Numeric values
            Aria::ColCount => AriaValueType::Integer,
            Aria::ColIndex => AriaValueType::Integer,
            Aria::ColSpan => AriaValueType::Integer,
            Aria::Level => AriaValueType::Integer,
            Aria::PosInSet => AriaValueType::Integer,
            Aria::RowCount => AriaValueType::Integer,
            Aria::RowIndex => AriaValueType::Integer,
            Aria::RowSpan => AriaValueType::Integer,
            Aria::SetSize => AriaValueType::Integer,
            Aria::ValueMax => AriaValueType::Number,
            Aria::ValueMin => AriaValueType::Number,
            Aria::ValueNow => AriaValueType::Number,
            // String / ID reference values (no restriction)
            Aria::ActiveDescendant => AriaValueType::IdRef,
            Aria::Controls => AriaValueType::IdRefList,
            Aria::DescribedBy => AriaValueType::IdRefList,
            Aria::Details => AriaValueType::IdRef,
            Aria::ErrorMessage => AriaValueType::IdRef,
            Aria::FlowTo => AriaValueType::IdRefList,
            Aria::Label => AriaValueType::FreeText,
            Aria::LabelledBy => AriaValueType::IdRefList,
            Aria::Owns => AriaValueType::IdRefList,
            Aria::Placeholder => AriaValueType::FreeText,
            Aria::RoleDescription => AriaValueType::FreeText,
            Aria::ValueText => AriaValueType::FreeText,
            Aria::KeyShortcuts => AriaValueType::FreeText,
            Aria::Description => AriaValueType::FreeText,
            Aria::BrailleLabel => AriaValueType::FreeText,
            Aria::BrailleRoleDescription => AriaValueType::FreeText,
            Aria::ColIndexText => AriaValueType::FreeText,
            Aria::RowIndexText => AriaValueType::FreeText,
            Aria::DropEffect => {
                AriaValueType::Enum(&["copy", "execute", "link", "move", "none", "popup"])
            }
        }
    }

    /// Whether this ARIA property is a global state/property (supported by all roles).
    pub fn is_global(&self) -> bool {
        matches!(
            self,
            Aria::Atomic
                | Aria::Busy
                | Aria::Controls
                | Aria::Current
                | Aria::DescribedBy
                | Aria::Description
                | Aria::Details
                | Aria::Disabled
                | Aria::DropEffect
                | Aria::ErrorMessage
                | Aria::FlowTo
                | Aria::Grabbed
                | Aria::HasPopup
                | Aria::Hidden
                | Aria::Invalid
                | Aria::KeyShortcuts
                | Aria::Label
                | Aria::LabelledBy
                | Aria::Live
                | Aria::Owns
                | Aria::Relevant
                | Aria::RoleDescription
                | Aria::BrailleLabel
                | Aria::BrailleRoleDescription
        )
    }

    /// Check if this ARIA property is supported by the given role.
    /// Global properties are always supported. Non-global properties
    /// are checked against role-specific support lists per WAI-ARIA 1.2.
    pub fn is_supported_by_role(&self, role: &Role) -> bool {
        if self.is_global() {
            return true;
        }
        match self {
            Aria::ActiveDescendant => matches!(
                role,
                Role::Application
                    | Role::Combobox
                    | Role::Grid
                    | Role::Group
                    | Role::ListBox
                    | Role::Menu
                    | Role::Menubar
                    | Role::RadioGroup
                    | Role::Row
                    | Role::SearchBox
                    | Role::TabList
                    | Role::TextBox
                    | Role::Tree
                    | Role::TreeGrid
            ),
            Aria::Autocomplete => matches!(role, Role::Combobox | Role::SearchBox | Role::TextBox),
            Aria::Checked => matches!(
                role,
                Role::Checkbox
                    | Role::MenuItemCheckbox
                    | Role::MenuItemRadio
                    | Role::Option
                    | Role::Radio
                    | Role::Switch
            ),
            Aria::ColCount => matches!(role, Role::Grid | Role::Table | Role::TreeGrid),
            Aria::ColIndex => matches!(
                role,
                Role::Cell | Role::ColumnHeader | Role::GridCell | Role::Row | Role::RowHeader
            ),
            Aria::ColIndexText => matches!(
                role,
                Role::Cell | Role::ColumnHeader | Role::GridCell | Role::Row | Role::RowHeader
            ),
            Aria::ColSpan => matches!(
                role,
                Role::Cell | Role::ColumnHeader | Role::GridCell | Role::RowHeader
            ),
            Aria::Expanded => matches!(
                role,
                Role::Application
                    | Role::Button
                    | Role::Checkbox
                    | Role::Combobox
                    | Role::GridCell
                    | Role::Link
                    | Role::ListBox
                    | Role::MenuItem
                    | Role::MenuItemCheckbox
                    | Role::MenuItemRadio
                    | Role::Row
                    | Role::RowHeader
                    | Role::Tab
                    | Role::TreeItem
            ),
            Aria::Level => matches!(
                role,
                Role::Heading | Role::ListItem | Role::Row | Role::TabList
            ),
            Aria::Modal => matches!(role, Role::AlertDialog | Role::Dialog),
            Aria::Multiline => matches!(role, Role::TextBox),
            Aria::Multiselectable => {
                matches!(
                    role,
                    Role::Grid | Role::ListBox | Role::TabList | Role::Tree | Role::TreeGrid
                )
            }
            Aria::Orientation => matches!(
                role,
                Role::Combobox
                    | Role::ListBox
                    | Role::Menu
                    | Role::Menubar
                    | Role::RadioGroup
                    | Role::ScrollBar
                    | Role::Separator
                    | Role::Slider
                    | Role::TabList
                    | Role::Toolbar
                    | Role::Tree
                    | Role::TreeGrid
            ),
            Aria::Placeholder => matches!(role, Role::SearchBox | Role::TextBox),
            Aria::PosInSet => matches!(
                role,
                Role::Article
                    | Role::ListItem
                    | Role::MenuItem
                    | Role::MenuItemCheckbox
                    | Role::MenuItemRadio
                    | Role::Option
                    | Role::Radio
                    | Role::Row
                    | Role::Tab
                    | Role::TreeItem
            ),
            Aria::Pressed => matches!(role, Role::Button),
            Aria::ReadOnly => matches!(
                role,
                Role::Checkbox
                    | Role::Combobox
                    | Role::Grid
                    | Role::GridCell
                    | Role::ListBox
                    | Role::RadioGroup
                    | Role::Slider
                    | Role::SpinButton
                    | Role::TextBox
            ),
            Aria::Required => matches!(
                role,
                Role::Checkbox
                    | Role::Combobox
                    | Role::GridCell
                    | Role::ListBox
                    | Role::RadioGroup
                    | Role::SpinButton
                    | Role::TextBox
                    | Role::Tree
            ),
            Aria::RowCount => matches!(role, Role::Grid | Role::Table | Role::TreeGrid),
            Aria::RowIndex => matches!(
                role,
                Role::Cell | Role::ColumnHeader | Role::GridCell | Role::Row | Role::RowHeader
            ),
            Aria::RowIndexText => matches!(
                role,
                Role::Cell | Role::ColumnHeader | Role::GridCell | Role::Row | Role::RowHeader
            ),
            Aria::RowSpan => matches!(
                role,
                Role::Cell | Role::ColumnHeader | Role::GridCell | Role::RowHeader
            ),
            Aria::Selected => matches!(
                role,
                Role::Cell
                    | Role::ColumnHeader
                    | Role::GridCell
                    | Role::Option
                    | Role::Row
                    | Role::RowHeader
                    | Role::Tab
                    | Role::TreeItem
            ),
            Aria::SetSize => matches!(
                role,
                Role::Article
                    | Role::ListItem
                    | Role::MenuItem
                    | Role::MenuItemCheckbox
                    | Role::MenuItemRadio
                    | Role::Option
                    | Role::Radio
                    | Role::Row
                    | Role::Tab
                    | Role::TreeItem
            ),
            Aria::Sort => matches!(role, Role::ColumnHeader | Role::RowHeader),
            Aria::ValueMax => matches!(
                role,
                Role::Meter
                    | Role::ProgressBar
                    | Role::ScrollBar
                    | Role::Separator
                    | Role::Slider
                    | Role::SpinButton
            ),
            Aria::ValueMin => matches!(
                role,
                Role::Meter
                    | Role::ProgressBar
                    | Role::ScrollBar
                    | Role::Separator
                    | Role::Slider
                    | Role::SpinButton
            ),
            Aria::ValueNow => matches!(
                role,
                Role::Meter
                    | Role::ProgressBar
                    | Role::ScrollBar
                    | Role::Separator
                    | Role::Slider
                    | Role::SpinButton
            ),
            Aria::ValueText => matches!(
                role,
                Role::Meter
                    | Role::ProgressBar
                    | Role::ScrollBar
                    | Role::Separator
                    | Role::Slider
                    | Role::SpinButton
            ),
            // For any property not explicitly listed above, assume it's supported
            // to avoid false positives.
            _ => true,
        }
    }
}

impl Display for Aria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).map_err(|_| std::fmt::Error::default())?;
        write!(f, "{}", s.trim_matches('"'))
    }
}

/// WAI-ARIA roles that can be assigned to HTML elements via the `role` attribute.
///
/// Covers concrete, abstract, and landmark roles from the
/// [WAI-ARIA 1.2 specification](https://www.w3.org/TR/wai-aria-1.2/#role_definitions).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum Role {
    // ── Concrete roles ──────────────────────────────────────────────
    Alert,
    AlertDialog,
    Application,
    Article,
    Banner,
    Button,
    Cell,
    Checkbox,
    ColumnHeader,
    Combobox,
    Complementary,
    ContentInfo,
    Definition,
    Dialog,
    Directory,
    Document,
    Feed,
    Figure,
    Form,
    Grid,
    GridCell,
    Group,
    Heading,
    Img,
    Link,
    List,
    ListBox,
    ListItem,
    Log,
    Main,
    Marquee,
    Math,
    Menu,
    Menubar,
    MenuItem,
    MenuItemCheckbox,
    MenuItemRadio,
    Meter,
    Navigation,
    None,
    Note,
    Option,
    Presentation,
    ProgressBar,
    Radio,
    RadioGroup,
    Region,
    Row,
    RowGroup,
    RowHeader,
    ScrollBar,
    Search,
    SearchBox,
    Separator,
    Slider,
    SpinButton,
    Status,
    Switch,
    Tab,
    Table,
    TabList,
    TabPanel,
    Term,
    TextBox,
    Timer,
    Toolbar,
    Tooltip,
    Tree,
    TreeGrid,
    TreeItem,

    // ── Abstract roles (WAI-ARIA ontology only, never valid on elements) ──
    Command,
    Composite,
    Input,
    Landmark,
    Range,
    Roletype,
    Section,
    Sectionhead,
    Select,
    Structure,
    Widget,
    Window,
}

impl Role {
    pub fn from_str(name: &str) -> Option<Role> {
        serde_json::from_str(&format!("\"{}\"", name)).ok()
    }

    /// Whether this role is an abstract WAI-ARIA role.
    /// Abstract roles exist for ontology purposes only and must never be
    /// used as a `role` attribute value on an element.
    pub fn is_abstract(&self) -> bool {
        matches!(
            self,
            Role::Command
                | Role::Composite
                | Role::Input
                | Role::Landmark
                | Role::Range
                | Role::Roletype
                | Role::Section
                | Role::Sectionhead
                | Role::Select
                | Role::Structure
                | Role::Widget
                | Role::Window
        )
    }

    /// Required ARIA properties for this role per WAI-ARIA 1.2.
    pub fn required_aria_props(&self) -> &'static [Aria] {
        match self {
            Role::Checkbox => &[Aria::Checked],
            Role::Combobox => &[Aria::Controls, Aria::Expanded],
            Role::Heading => &[Aria::Level],
            Role::Meter => &[Aria::ValueNow, Aria::ValueMax, Aria::ValueMin],
            Role::MenuItemCheckbox => &[Aria::Checked],
            Role::MenuItemRadio => &[Aria::Checked],
            Role::Radio => &[Aria::Checked],
            Role::ScrollBar => &[Aria::Controls, Aria::ValueNow],
            Role::Slider => &[Aria::ValueNow],
            Role::Switch => &[Aria::Checked],
            _ => &[],
        }
    }

    /// If a semantic HTML tag exists for this role, return a suggestion string.
    pub fn preferred_tag(&self) -> Option<&'static str> {
        match self {
            Role::Banner => Some("<header>"),
            Role::Button => Some("<button>"),
            Role::Complementary => Some("<aside>"),
            Role::ContentInfo => Some("<footer>"),
            Role::Form => Some("<form>"),
            Role::Heading => Some("<h1>-<h6>"),
            Role::Img => Some("<img>"),
            Role::Link => Some("<a>"),
            Role::List => Some("<ul> or <ol>"),
            Role::ListItem => Some("<li>"),
            Role::Main => Some("<main>"),
            Role::Navigation => Some("<nav>"),
            Role::ProgressBar => Some("<progress>"),
            Role::Region => Some("<section>"),
            Role::Row => Some("<tr>"),
            Role::RowGroup => Some("<tbody>, <thead>, or <tfoot>"),
            Role::RowHeader => Some("<th>"),
            Role::Table => Some("<table>"),
            Role::TextBox => Some("<input> or <textarea>"),
            _ => None,
        }
    }

    pub fn is_interactive(&self) -> bool {
        matches!(
            self,
            Role::Button
                | Role::Checkbox
                | Role::Combobox
                | Role::GridCell
                | Role::Link
                | Role::ListBox
                | Role::Menu
                | Role::Menubar
                | Role::MenuItem
                | Role::MenuItemCheckbox
                | Role::MenuItemRadio
                | Role::Option
                | Role::Radio
                | Role::ScrollBar
                | Role::SearchBox
                | Role::Slider
                | Role::SpinButton
                | Role::Switch
                | Role::Tab
                | Role::TextBox
                | Role::TreeItem
        )
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).map_err(|_| std::fmt::Error::default())?;
        write!(f, "{}", s.trim_matches('"'))
    }
}

/// HTML attribute names relevant to accessibility linting.
///
/// Includes event handlers (e.g. `onclick`), global attributes (e.g. `tabindex`),
/// and element-specific attributes (e.g. `alt`, `href`) that the linter inspects.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[non_exhaustive]
pub enum AttributeName {
    #[serde(rename = "onmouseover", alias = "on:mouseover")]
    OnMouseOver,
    #[serde(rename = "onmouseout", alias = "on:mouseout")]
    OnMouseOut,
    #[serde(rename = "onclick", alias = "on:click")]
    OnClick,
    #[serde(rename = "onkeydown", alias = "on:keydown")]
    OnKeyDown,
    #[serde(rename = "onkeypress", alias = "on:keypress")]
    OnKeyPress,
    #[serde(rename = "onkeyup", alias = "on:keyup")]
    OnKeyUp,
    #[serde(rename = "onfocus", alias = "on:focus")]
    OnFocus,
    #[serde(rename = "onblur", alias = "on:blur")]
    OnBlur,
    #[serde(rename = "onchange", alias = "on:change")]
    OnChange,
    #[serde(rename = "oninput", alias = "on:input")]
    OnInput,
    #[serde(rename = "onsubmit", alias = "on:submit")]
    OnSubmit,
    #[serde(rename = "accesskey")]
    AccessKey,
    #[serde(rename = "alt")]
    Alt,
    #[serde(rename = "autocomplete")]
    Autocomplete,
    #[serde(rename = "autofocus")]
    AutoFocus,
    #[serde(rename = "class")]
    Class,
    #[serde(rename = "for", alias = "html_for")]
    For,
    #[serde(rename = "href")]
    Href,
    #[serde(rename = "lang")]
    Lang,
    #[serde(rename = "muted")]
    Muted,
    #[serde(rename = "role")]
    Role,
    #[serde(rename = "scope")]
    Scope,
    #[serde(rename = "src")]
    Src,
    #[serde(rename = "tabindex")]
    TabIndex,
    #[serde(rename = "title")]
    Title,
    #[serde(rename = "type")]
    Type,
    #[serde(untagged)]
    Aria(Aria),
    #[serde(untagged)]
    /// Either a non-standard attribute or one not relevant to this crate.
    Unknown(String),
}

impl AttributeName {
    pub fn from_str(name: &str) -> Option<AttributeName> {
        serde_json::from_str(&format!("\"{}\"", name)).ok()
    }
}

impl Display for AttributeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).map_err(|_| std::fmt::Error::default())?;
        write!(f, "{}", s.trim_matches('"'))
    }
}

/// HTML element tag names recognised by the linter.
///
/// Covers the standard HTML5 element set. Used to match parsed elements
/// against tag-specific lint rules and implicit ARIA role mappings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Tag {
    A,
    Abbr,
    Address,
    Area,
    Article,
    Aside,
    Audio,
    B,
    Base,
    Bdi,
    Bdo,
    Blink,
    Blockquote,
    Body,
    Br,
    Button,
    Canvas,
    Caption,
    Cite,
    Code,
    Col,
    Colgroup,
    Data,
    Datalist,
    Dd,
    Del,
    Details,
    Dfn,
    Dialog,
    Div,
    Dl,
    Dt,
    Em,
    Embed,
    Fieldset,
    Figcaption,
    Figure,
    Footer,
    Form,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Head,
    Header,
    HGroup,
    Hr,
    Html,
    I,
    Iframe,
    Img,
    Input,
    Ins,
    Kbd,
    Label,
    Legend,
    Li,
    Link,
    Main,
    Map,
    Mark,
    Marquee,
    Math,
    Menu,
    Meta,
    Meter,
    Nav,
    Noscript,
    Object,
    Ol,
    Optgroup,
    Option,
    Output,
    P,
    Param,
    Picture,
    Pre,
    Progress,
    Q,
    Rp,
    Rt,
    Ruby,
    S,
    Samp,
    Script,
    Section,
    Select,
    Small,
    Source,
    Span,
    Strong,
    Style,
    Sub,
    Summary,
    Sup,
    Svg,
    Table,
    Tbody,
    Td,
    Template,
    Textarea,
    Tfoot,
    Th,
    Thead,
    Time,
    Title,
    Tr,
    Track,
    U,
    Ul,
    Var,
    Video,
    Wbr,
}

impl Tag {
    pub fn from_str(name: &str) -> Option<Tag> {
        serde_json::from_str(&format!("\"{}\"", name)).ok()
    }

    pub fn is_interactive(&self) -> bool {
        matches!(
            self,
            Tag::A
                | Tag::Button
                | Tag::Details
                | Tag::Input
                | Tag::Select
                | Tag::Textarea
                | Tag::Summary
        )
    }

    pub fn supports_aria(&self) -> bool {
        !matches!(
            self,
            Tag::Base | Tag::Head | Tag::Html | Tag::Meta | Tag::Script | Tag::Style | Tag::Title
        )
    }

    pub fn is_heading(&self) -> bool {
        matches!(
            self,
            Tag::H1 | Tag::H2 | Tag::H3 | Tag::H4 | Tag::H5 | Tag::H6
        )
    }

    /// Whether this element has no semantic role (e.g. div, span).
    /// These are "static" elements in jsx-a11y terminology.
    pub fn is_static(&self) -> bool {
        !self.is_interactive() && self.implicit_role().is_none()
    }

    /// Whether this element has a non-interactive semantic role.
    pub fn is_non_interactive_semantic(&self) -> bool {
        !self.is_interactive() && self.implicit_role().is_some()
    }

    /// HTML void elements that cannot have children.
    pub fn is_void_element(&self) -> bool {
        matches!(
            self,
            Tag::Area
                | Tag::Base
                | Tag::Br
                | Tag::Col
                | Tag::Embed
                | Tag::Hr
                | Tag::Img
                | Tag::Input
                | Tag::Link
                | Tag::Meta
                | Tag::Param
                | Tag::Source
                | Tag::Track
                | Tag::Wbr
        )
    }

    pub fn implicit_role(&self) -> Option<Role> {
        match self {
            Tag::A => Some(Role::Link),    // when href is present
            Tag::Area => Some(Role::Link), // when href is present
            Tag::Article => Some(Role::Article),
            Tag::Aside => Some(Role::Complementary),
            Tag::Body => Some(Role::Document),
            Tag::Button => Some(Role::Button),
            Tag::Datalist => Some(Role::ListBox),
            Tag::Details => Some(Role::Group),
            Tag::Dialog => Some(Role::Dialog),
            Tag::Fieldset => Some(Role::Group),
            Tag::Figure => Some(Role::Figure),
            Tag::Footer => Some(Role::ContentInfo),
            Tag::Form => Some(Role::Form),
            Tag::H1 => Some(Role::Heading),
            Tag::H2 => Some(Role::Heading),
            Tag::H3 => Some(Role::Heading),
            Tag::H4 => Some(Role::Heading),
            Tag::H5 => Some(Role::Heading),
            Tag::H6 => Some(Role::Heading),
            Tag::Header => Some(Role::Banner),
            Tag::Hr => Some(Role::Separator),
            Tag::Img => Some(Role::Img),
            Tag::Input => Some(Role::TextBox), // default, depends on type
            Tag::Li => Some(Role::ListItem),
            Tag::Main => Some(Role::Main),
            Tag::Math => Some(Role::Math),
            Tag::Menu => Some(Role::List),
            Tag::Meter => Some(Role::Meter),
            Tag::Nav => Some(Role::Navigation),
            Tag::Ol => Some(Role::List),
            Tag::Optgroup => Some(Role::Group),
            Tag::Option => Some(Role::Option),
            Tag::Output => Some(Role::Status),
            Tag::Progress => Some(Role::ProgressBar),
            Tag::Section => Some(Role::Region),
            Tag::Select => Some(Role::Combobox), // or listbox depending on size
            Tag::Summary => Some(Role::Button),
            Tag::Table => Some(Role::Table),
            Tag::Tbody => Some(Role::RowGroup),
            Tag::Td => Some(Role::Cell),
            Tag::Textarea => Some(Role::TextBox),
            Tag::Tfoot => Some(Role::RowGroup),
            Tag::Th => Some(Role::ColumnHeader),
            Tag::Thead => Some(Role::RowGroup),
            Tag::Tr => Some(Role::Row),
            Tag::Ul => Some(Role::List),
            _ => None,
        }
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).map_err(|_| std::fmt::Error::default())?;
        write!(f, "{}", s.trim_matches('"'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_value_validation() {
        let vtype = AriaValueType::Bool;
        assert!(vtype.is_valid("true"));
        assert!(vtype.is_valid("false"));
        assert!(!vtype.is_valid("yes"));
        assert!(!vtype.is_valid("1"));
    }

    #[test]
    fn test_tristate_value_validation() {
        let vtype = AriaValueType::TristateBool;
        assert!(vtype.is_valid("true"));
        assert!(vtype.is_valid("false"));
        assert!(vtype.is_valid("mixed"));
        assert!(!vtype.is_valid("maybe"));
    }

    #[test]
    fn test_implicit_roles() {
        assert_eq!(Tag::Button.implicit_role(), Some(Role::Button));
        assert_eq!(Tag::Nav.implicit_role(), Some(Role::Navigation));
        assert_eq!(Tag::H1.implicit_role(), Some(Role::Heading));
    }
}
