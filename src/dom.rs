//! The HTML/ARIA vocabulary the linter understands: elements ([`Tag`]),
//! WAI-ARIA roles ([`Role`]) and properties ([`Aria`]), and attribute names
//! ([`AttributeName`]).
//!
//! Names are parsed and displayed through [`std::str::FromStr`] and
//! [`std::fmt::Display`] (e.g. `"aria-label".parse::<Aria>()`), and serialize
//! to the same strings.

use std::fmt;
use std::str::FromStr;

/// Implements `Serialize`/`Deserialize` through the type's string form so the
/// strum attributes are the single source of truth for names.
macro_rules! serde_via_str {
    ($($t:ty),* $(,)?) => {$(
        impl serde::Serialize for $t {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(&self.to_string())
            }
        }
        impl<'de> serde::Deserialize<'de> for $t {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let s = <std::borrow::Cow<'de, str>>::deserialize(d)?;
                s.parse().map_err(|_| {
                    serde::de::Error::custom(format!("unknown {} `{}`", stringify!($t), s))
                })
            }
        }
    )*};
}
pub(crate) use serde_via_str;

/// Types of values an ARIA attribute can accept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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
    pub fn is_valid(self, value: &str) -> bool {
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
    pub fn expected_description(self) -> String {
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
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::EnumString,
    strum::Display,
    strum::IntoStaticStr,
    strum::EnumIter,
)]
#[non_exhaustive]
pub enum Aria {
    /// `aria-activedescendant`
    #[strum(serialize = "aria-activedescendant")]
    ActiveDescendant,
    /// `aria-atomic`
    #[strum(serialize = "aria-atomic")]
    Atomic,
    /// `aria-autocomplete`
    #[strum(serialize = "aria-autocomplete")]
    Autocomplete,
    /// `aria-braillelabel`
    #[strum(serialize = "aria-braillelabel")]
    BrailleLabel,
    /// `aria-brailleroledescription`
    #[strum(serialize = "aria-brailleroledescription")]
    BrailleRoleDescription,
    /// `aria-busy`
    #[strum(serialize = "aria-busy")]
    Busy,
    /// `aria-checked`
    #[strum(serialize = "aria-checked")]
    Checked,
    /// `aria-colcount`
    #[strum(serialize = "aria-colcount")]
    ColCount,
    /// `aria-colindex`
    #[strum(serialize = "aria-colindex")]
    ColIndex,
    /// `aria-colindextext`
    #[strum(serialize = "aria-colindextext")]
    ColIndexText,
    /// `aria-colspan`
    #[strum(serialize = "aria-colspan")]
    ColSpan,
    /// `aria-controls`
    #[strum(serialize = "aria-controls")]
    Controls,
    /// `aria-current`
    #[strum(serialize = "aria-current")]
    Current,
    /// `aria-describedby`
    #[strum(serialize = "aria-describedby")]
    DescribedBy,
    /// `aria-description`
    #[strum(serialize = "aria-description")]
    Description,
    /// `aria-details`
    #[strum(serialize = "aria-details")]
    Details,
    /// `aria-disabled`
    #[strum(serialize = "aria-disabled")]
    Disabled,
    /// `aria-dropeffect`
    #[strum(serialize = "aria-dropeffect")]
    DropEffect,
    /// `aria-errormessage`
    #[strum(serialize = "aria-errormessage")]
    ErrorMessage,
    /// `aria-expanded`
    #[strum(serialize = "aria-expanded")]
    Expanded,
    /// `aria-flowto`
    #[strum(serialize = "aria-flowto")]
    FlowTo,
    /// `aria-grabbed`
    #[strum(serialize = "aria-grabbed")]
    Grabbed,
    /// `aria-haspopup`
    #[strum(serialize = "aria-haspopup")]
    HasPopup,
    /// `aria-hidden`
    #[strum(serialize = "aria-hidden")]
    Hidden,
    /// `aria-invalid`
    #[strum(serialize = "aria-invalid")]
    Invalid,
    /// `aria-keyshortcuts`
    #[strum(serialize = "aria-keyshortcuts")]
    KeyShortcuts,
    /// `aria-label`
    #[strum(serialize = "aria-label")]
    Label,
    /// `aria-labelledby`
    #[strum(serialize = "aria-labelledby")]
    LabelledBy,
    /// `aria-level`
    #[strum(serialize = "aria-level")]
    Level,
    /// `aria-live`
    #[strum(serialize = "aria-live")]
    Live,
    /// `aria-modal`
    #[strum(serialize = "aria-modal")]
    Modal,
    /// `aria-multiline`
    #[strum(serialize = "aria-multiline")]
    Multiline,
    /// `aria-multiselectable`
    #[strum(serialize = "aria-multiselectable")]
    Multiselectable,
    /// `aria-orientation`
    #[strum(serialize = "aria-orientation")]
    Orientation,
    /// `aria-owns`
    #[strum(serialize = "aria-owns")]
    Owns,
    /// `aria-placeholder`
    #[strum(serialize = "aria-placeholder")]
    Placeholder,
    /// `aria-posinset`
    #[strum(serialize = "aria-posinset")]
    PosInSet,
    /// `aria-pressed`
    #[strum(serialize = "aria-pressed")]
    Pressed,
    /// `aria-readonly`
    #[strum(serialize = "aria-readonly")]
    ReadOnly,
    /// `aria-relevant`
    #[strum(serialize = "aria-relevant")]
    Relevant,
    /// `aria-required`
    #[strum(serialize = "aria-required")]
    Required,
    /// `aria-roledescription`
    #[strum(serialize = "aria-roledescription")]
    RoleDescription,
    /// `aria-rowcount`
    #[strum(serialize = "aria-rowcount")]
    RowCount,
    /// `aria-rowindex`
    #[strum(serialize = "aria-rowindex")]
    RowIndex,
    /// `aria-rowindextext`
    #[strum(serialize = "aria-rowindextext")]
    RowIndexText,
    /// `aria-rowspan`
    #[strum(serialize = "aria-rowspan")]
    RowSpan,
    /// `aria-selected`
    #[strum(serialize = "aria-selected")]
    Selected,
    /// `aria-setsize`
    #[strum(serialize = "aria-setsize")]
    SetSize,
    /// `aria-sort`
    #[strum(serialize = "aria-sort")]
    Sort,
    /// `aria-valuemax`
    #[strum(serialize = "aria-valuemax")]
    ValueMax,
    /// `aria-valuemin`
    #[strum(serialize = "aria-valuemin")]
    ValueMin,
    /// `aria-valuenow`
    #[strum(serialize = "aria-valuenow")]
    ValueNow,
    /// `aria-valuetext`
    #[strum(serialize = "aria-valuetext")]
    ValueText,
}

impl Aria {
    /// The kind of value this property accepts.
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
    pub fn is_supported_by_role(&self, role: Role) -> bool {
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

/// WAI-ARIA roles that can be assigned to HTML elements via the `role` attribute.
///
/// Covers concrete, abstract, and landmark roles from the
/// [WAI-ARIA 1.2 specification](https://www.w3.org/TR/wai-aria-1.2/#role_definitions).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::EnumString,
    strum::Display,
    strum::IntoStaticStr,
    strum::EnumIter,
)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
#[non_exhaustive]
pub enum Role {
    // ── Concrete roles ──────────────────────────────────────────────
    /// The `alert` role.
    Alert,
    /// The `alertdialog` role.
    AlertDialog,
    /// The `application` role.
    Application,
    /// The `article` role.
    Article,
    /// The `banner` role.
    Banner,
    /// The `button` role.
    Button,
    /// The `cell` role.
    Cell,
    /// The `checkbox` role.
    Checkbox,
    /// The `columnheader` role.
    ColumnHeader,
    /// The `combobox` role.
    Combobox,
    /// The `complementary` role.
    Complementary,
    /// The `contentinfo` role.
    ContentInfo,
    /// The `definition` role.
    Definition,
    /// The `dialog` role.
    Dialog,
    /// The `directory` role.
    Directory,
    /// The `document` role.
    Document,
    /// The `feed` role.
    Feed,
    /// The `figure` role.
    Figure,
    /// The `form` role.
    Form,
    /// The `grid` role.
    Grid,
    /// The `gridcell` role.
    GridCell,
    /// The `group` role.
    Group,
    /// The `heading` role.
    Heading,
    /// The `img` role.
    Img,
    /// The `link` role.
    Link,
    /// The `list` role.
    List,
    /// The `listbox` role.
    ListBox,
    /// The `listitem` role.
    ListItem,
    /// The `log` role.
    Log,
    /// The `main` role.
    Main,
    /// The `marquee` role.
    Marquee,
    /// The `math` role.
    Math,
    /// The `menu` role.
    Menu,
    /// The `menubar` role.
    Menubar,
    /// The `menuitem` role.
    MenuItem,
    /// The `menuitemcheckbox` role.
    MenuItemCheckbox,
    /// The `menuitemradio` role.
    MenuItemRadio,
    /// The `meter` role.
    Meter,
    /// The `navigation` role.
    Navigation,
    /// The `none` role.
    None,
    /// The `note` role.
    Note,
    /// The `option` role.
    Option,
    /// The `presentation` role.
    Presentation,
    /// The `progressbar` role.
    ProgressBar,
    /// The `radio` role.
    Radio,
    /// The `radiogroup` role.
    RadioGroup,
    /// The `region` role.
    Region,
    /// The `row` role.
    Row,
    /// The `rowgroup` role.
    RowGroup,
    /// The `rowheader` role.
    RowHeader,
    /// The `scrollbar` role.
    ScrollBar,
    /// The `search` role.
    Search,
    /// The `searchbox` role.
    SearchBox,
    /// The `separator` role.
    Separator,
    /// The `slider` role.
    Slider,
    /// The `spinbutton` role.
    SpinButton,
    /// The `status` role.
    Status,
    /// The `switch` role.
    Switch,
    /// The `tab` role.
    Tab,
    /// The `table` role.
    Table,
    /// The `tablist` role.
    TabList,
    /// The `tabpanel` role.
    TabPanel,
    /// The `term` role.
    Term,
    /// The `textbox` role.
    TextBox,
    /// The `timer` role.
    Timer,
    /// The `toolbar` role.
    Toolbar,
    /// The `tooltip` role.
    Tooltip,
    /// The `tree` role.
    Tree,
    /// The `treegrid` role.
    TreeGrid,
    /// The `treeitem` role.
    TreeItem,

    // ── Abstract roles (WAI-ARIA ontology only, never valid on elements) ──
    /// The `command` role.
    Command,
    /// The `composite` role.
    Composite,
    /// The `input` role.
    Input,
    /// The `landmark` role.
    Landmark,
    /// The `range` role.
    Range,
    /// The `roletype` role.
    Roletype,
    /// The `section` role.
    Section,
    /// The `sectionhead` role.
    Sectionhead,
    /// The `select` role.
    Select,
    /// The `structure` role.
    Structure,
    /// The `widget` role.
    Widget,
    /// The `window` role.
    Window,
}

impl Role {
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

    /// Whether this is a widget role that users interact with directly.
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

/// HTML attribute names relevant to accessibility linting.
///
/// Includes event handlers (e.g. `onclick`), global attributes (e.g. `tabindex`),
/// and element-specific attributes (e.g. `alt`, `href`) that the linter inspects.
/// Framework spellings are normalized when parsing: Leptos `on:click` and
/// `attr:aria-label`, Dioxus `aria_label`, Yew `html_for`, and raw
/// identifiers like `r#type`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AttributeName {
    /// `onmouseover`
    OnMouseOver,
    /// `onmouseout`
    OnMouseOut,
    /// `onclick`
    OnClick,
    /// `onkeydown`
    OnKeyDown,
    /// `onkeypress`
    OnKeyPress,
    /// `onkeyup`
    OnKeyUp,
    /// `onfocus`
    OnFocus,
    /// `onblur`
    OnBlur,
    /// `onchange`
    OnChange,
    /// `oninput`
    OnInput,
    /// `onsubmit`
    OnSubmit,
    /// `accesskey`
    AccessKey,
    /// `alt`
    Alt,
    /// `autocomplete`
    Autocomplete,
    /// `autofocus`
    AutoFocus,
    /// `class`
    Class,
    /// `for`
    For,
    /// `href`
    Href,
    /// `id`
    Id,
    /// `kind`
    Kind,
    /// `lang`
    Lang,
    /// `muted`
    Muted,
    /// `role`
    Role,
    /// `scope`
    Scope,
    /// `src`
    Src,
    /// `tabindex`
    TabIndex,
    /// `title`
    Title,
    /// `type`
    Type,
    /// `value`
    Value,
    /// A WAI-ARIA `aria-*` attribute.
    Aria(Aria),
    /// Any other attribute (including unknown `aria-*` names), as written.
    Unknown(String),
}

impl AttributeName {
    /// Parse an attribute name as written in a macro. Never fails: names the
    /// linter does not model become [`AttributeName::Unknown`].
    pub fn parse(raw: &str) -> AttributeName {
        let name = raw.strip_prefix("r#").unwrap_or(raw);
        let normalized;
        let name = if let Some(event) = name.strip_prefix("on:") {
            // Leptos: `on:click`.
            normalized = format!("on{event}");
            normalized.as_str()
        } else if name.starts_with("aria_") {
            // Dioxus: `aria_label`.
            normalized = name.replace('_', "-");
            normalized.as_str()
        } else {
            // Leptos: `attr:aria-label`.
            name.strip_prefix("attr:").unwrap_or(name)
        };
        let known = match name {
            "onmouseover" => Some(AttributeName::OnMouseOver),
            "onmouseout" => Some(AttributeName::OnMouseOut),
            "onclick" => Some(AttributeName::OnClick),
            "onkeydown" => Some(AttributeName::OnKeyDown),
            "onkeypress" => Some(AttributeName::OnKeyPress),
            "onkeyup" => Some(AttributeName::OnKeyUp),
            "onfocus" => Some(AttributeName::OnFocus),
            "onblur" => Some(AttributeName::OnBlur),
            "onchange" => Some(AttributeName::OnChange),
            "oninput" => Some(AttributeName::OnInput),
            "onsubmit" => Some(AttributeName::OnSubmit),
            "accesskey" => Some(AttributeName::AccessKey),
            "alt" => Some(AttributeName::Alt),
            "autocomplete" => Some(AttributeName::Autocomplete),
            "autofocus" => Some(AttributeName::AutoFocus),
            "class" => Some(AttributeName::Class),
            "for" | "html_for" | "htmlFor" => Some(AttributeName::For),
            "href" => Some(AttributeName::Href),
            "id" => Some(AttributeName::Id),
            "kind" => Some(AttributeName::Kind),
            "lang" => Some(AttributeName::Lang),
            "muted" => Some(AttributeName::Muted),
            "role" => Some(AttributeName::Role),
            "scope" => Some(AttributeName::Scope),
            "src" => Some(AttributeName::Src),
            "tabindex" => Some(AttributeName::TabIndex),
            "title" => Some(AttributeName::Title),
            "type" => Some(AttributeName::Type),
            "value" => Some(AttributeName::Value),
            _ => name.parse::<Aria>().ok().map(AttributeName::Aria),
        };
        known.unwrap_or_else(|| AttributeName::Unknown(raw.to_string()))
    }

    /// Whether this is a keyboard or mouse event handler (`onclick`,
    /// `onkeydown`, …) that implies the element is interactive.
    pub fn is_interaction_handler(&self) -> bool {
        matches!(
            self,
            AttributeName::OnClick
                | AttributeName::OnKeyDown
                | AttributeName::OnKeyUp
                | AttributeName::OnKeyPress
                | AttributeName::OnMouseOver
                | AttributeName::OnMouseOut
        )
    }
}

impl FromStr for AttributeName {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AttributeName::parse(s))
    }
}

impl fmt::Display for AttributeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AttributeName::OnMouseOver => "onmouseover",
            AttributeName::OnMouseOut => "onmouseout",
            AttributeName::OnClick => "onclick",
            AttributeName::OnKeyDown => "onkeydown",
            AttributeName::OnKeyPress => "onkeypress",
            AttributeName::OnKeyUp => "onkeyup",
            AttributeName::OnFocus => "onfocus",
            AttributeName::OnBlur => "onblur",
            AttributeName::OnChange => "onchange",
            AttributeName::OnInput => "oninput",
            AttributeName::OnSubmit => "onsubmit",
            AttributeName::AccessKey => "accesskey",
            AttributeName::Alt => "alt",
            AttributeName::Autocomplete => "autocomplete",
            AttributeName::AutoFocus => "autofocus",
            AttributeName::Class => "class",
            AttributeName::For => "for",
            AttributeName::Href => "href",
            AttributeName::Id => "id",
            AttributeName::Kind => "kind",
            AttributeName::Lang => "lang",
            AttributeName::Muted => "muted",
            AttributeName::Role => "role",
            AttributeName::Scope => "scope",
            AttributeName::Src => "src",
            AttributeName::TabIndex => "tabindex",
            AttributeName::Title => "title",
            AttributeName::Type => "type",
            AttributeName::Value => "value",
            AttributeName::Aria(aria) => return fmt::Display::fmt(aria, f),
            AttributeName::Unknown(name) => name,
        };
        f.write_str(s)
    }
}

serde_via_str!(Aria, Role, Tag, AttributeName);

/// HTML element tag names recognised by the linter.
///
/// Covers the standard HTML5 element set. Used to match parsed elements
/// against tag-specific lint rules and implicit ARIA role mappings.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::EnumString,
    strum::Display,
    strum::IntoStaticStr,
    strum::EnumIter,
)]
#[strum(serialize_all = "lowercase")]
#[non_exhaustive]
pub enum Tag {
    /// The `<a>` element.
    A,
    /// The `<abbr>` element.
    Abbr,
    /// The `<address>` element.
    Address,
    /// The `<area>` element.
    Area,
    /// The `<article>` element.
    Article,
    /// The `<aside>` element.
    Aside,
    /// The `<audio>` element.
    Audio,
    /// The `<b>` element.
    B,
    /// The `<base>` element.
    Base,
    /// The `<bdi>` element.
    Bdi,
    /// The `<bdo>` element.
    Bdo,
    /// The `<blink>` element.
    Blink,
    /// The `<blockquote>` element.
    Blockquote,
    /// The `<body>` element.
    Body,
    /// The `<br>` element.
    Br,
    /// The `<button>` element.
    Button,
    /// The `<canvas>` element.
    Canvas,
    /// The `<caption>` element.
    Caption,
    /// The `<cite>` element.
    Cite,
    /// The `<code>` element.
    Code,
    /// The `<col>` element.
    Col,
    /// The `<colgroup>` element.
    Colgroup,
    /// The `<data>` element.
    Data,
    /// The `<datalist>` element.
    Datalist,
    /// The `<dd>` element.
    Dd,
    /// The `<del>` element.
    Del,
    /// The `<details>` element.
    Details,
    /// The `<dfn>` element.
    Dfn,
    /// The `<dialog>` element.
    Dialog,
    /// The `<div>` element.
    Div,
    /// The `<dl>` element.
    Dl,
    /// The `<dt>` element.
    Dt,
    /// The `<em>` element.
    Em,
    /// The `<embed>` element.
    Embed,
    /// The `<fieldset>` element.
    Fieldset,
    /// The `<figcaption>` element.
    Figcaption,
    /// The `<figure>` element.
    Figure,
    /// The `<footer>` element.
    Footer,
    /// The `<form>` element.
    Form,
    /// The `<h1>` element.
    H1,
    /// The `<h2>` element.
    H2,
    /// The `<h3>` element.
    H3,
    /// The `<h4>` element.
    H4,
    /// The `<h5>` element.
    H5,
    /// The `<h6>` element.
    H6,
    /// The `<head>` element.
    Head,
    /// The `<header>` element.
    Header,
    /// The `<hgroup>` element.
    HGroup,
    /// The `<hr>` element.
    Hr,
    /// The `<html>` element.
    Html,
    /// The `<i>` element.
    I,
    /// The `<iframe>` element.
    Iframe,
    /// The `<img>` element.
    Img,
    /// The `<input>` element.
    Input,
    /// The `<ins>` element.
    Ins,
    /// The `<kbd>` element.
    Kbd,
    /// The `<label>` element.
    Label,
    /// The `<legend>` element.
    Legend,
    /// The `<li>` element.
    Li,
    /// The `<link>` element.
    Link,
    /// The `<main>` element.
    Main,
    /// The `<map>` element.
    Map,
    /// The `<mark>` element.
    Mark,
    /// The `<marquee>` element.
    Marquee,
    /// The `<math>` element.
    Math,
    /// The `<menu>` element.
    Menu,
    /// The `<meta>` element.
    Meta,
    /// The `<meter>` element.
    Meter,
    /// The `<nav>` element.
    Nav,
    /// The `<noscript>` element.
    Noscript,
    /// The `<object>` element.
    Object,
    /// The `<ol>` element.
    Ol,
    /// The `<optgroup>` element.
    Optgroup,
    /// The `<option>` element.
    Option,
    /// The `<output>` element.
    Output,
    /// The `<p>` element.
    P,
    /// The `<param>` element.
    Param,
    /// The `<picture>` element.
    Picture,
    /// The `<pre>` element.
    Pre,
    /// The `<progress>` element.
    Progress,
    /// The `<q>` element.
    Q,
    /// The `<rp>` element.
    Rp,
    /// The `<rt>` element.
    Rt,
    /// The `<ruby>` element.
    Ruby,
    /// The `<s>` element.
    S,
    /// The `<samp>` element.
    Samp,
    /// The `<script>` element.
    Script,
    /// The `<section>` element.
    Section,
    /// The `<select>` element.
    Select,
    /// The `<small>` element.
    Small,
    /// The `<source>` element.
    Source,
    /// The `<span>` element.
    Span,
    /// The `<strong>` element.
    Strong,
    /// The `<style>` element.
    Style,
    /// The `<sub>` element.
    Sub,
    /// The `<summary>` element.
    Summary,
    /// The `<sup>` element.
    Sup,
    /// The `<svg>` element.
    Svg,
    /// The `<table>` element.
    Table,
    /// The `<tbody>` element.
    Tbody,
    /// The `<td>` element.
    Td,
    /// The `<template>` element.
    Template,
    /// The `<textarea>` element.
    Textarea,
    /// The `<tfoot>` element.
    Tfoot,
    /// The `<th>` element.
    Th,
    /// The `<thead>` element.
    Thead,
    /// The `<time>` element.
    Time,
    /// The `<title>` element.
    Title,
    /// The `<tr>` element.
    Tr,
    /// The `<track>` element.
    Track,
    /// The `<u>` element.
    U,
    /// The `<ul>` element.
    Ul,
    /// The `<var>` element.
    Var,
    /// The `<video>` element.
    Video,
    /// The `<wbr>` element.
    Wbr,
}

impl Tag {
    /// Whether the element is natively interactive (focusable and operable).
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

    /// Whether the element may carry ARIA roles, states and properties.
    pub fn supports_aria(&self) -> bool {
        !matches!(
            self,
            Tag::Base | Tag::Head | Tag::Html | Tag::Meta | Tag::Script | Tag::Style | Tag::Title
        )
    }

    /// Whether the element is one of `<h1>`–`<h6>`.
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

    /// The element's implicit ARIA role, if it has one.
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
    fn test_names_round_trip() {
        assert_eq!("aria-label".parse::<Aria>(), Ok(Aria::Label));
        assert_eq!(Aria::LabelledBy.to_string(), "aria-labelledby");
        assert_eq!("alertdialog".parse::<Role>(), Ok(Role::AlertDialog));
        assert_eq!("Button".parse::<Role>(), Ok(Role::Button));
        assert_eq!("hgroup".parse::<Tag>(), Ok(Tag::HGroup));
        assert!(
            "A".parse::<Tag>().is_err(),
            "PascalCase components are not elements"
        );
        assert_eq!(AttributeName::parse("on:click"), AttributeName::OnClick);
        assert_eq!(
            AttributeName::parse("attr:aria-hidden"),
            AttributeName::Aria(Aria::Hidden)
        );
        assert_eq!(AttributeName::parse("r#type"), AttributeName::Type);
        assert_eq!(AttributeName::parse("html_for"), AttributeName::For);
        assert_eq!(
            AttributeName::parse("aria_labelledby"),
            AttributeName::Aria(Aria::LabelledBy)
        );
        assert_eq!(
            AttributeName::parse("aria-foo"),
            AttributeName::Unknown("aria-foo".into())
        );
        assert_eq!(serde_json::to_string(&Tag::Img).unwrap(), "\"img\"");
        assert_eq!(
            serde_json::from_str::<Role>("\"button\"").unwrap(),
            Role::Button
        );
    }

    #[test]
    fn test_implicit_roles() {
        assert_eq!(Tag::Button.implicit_role(), Some(Role::Button));
        assert_eq!(Tag::Nav.implicit_role(), Some(Role::Navigation));
        assert_eq!(Tag::H1.implicit_role(), Some(Role::Heading));
    }
}
