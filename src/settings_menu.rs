use std::any::{Any, TypeId};
use crate::texture::Texture;
use crate::DIRS;
use derive_more::From;
use iced::widget::*;
use iced::{Border, Color, Element, Length};
use iced_aw::menu_bar;
use iced_aw::{menu, menu_items};
use minsweeper_rs::board::{BoardSize, ConventionalSize};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;
use std::fs::{create_dir, File};
use std::io;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::LazyLock;
use minsweeper_rs::solver::mia::MiaSolver;
use minsweeper_rs::solver::Solver;
use minsweeper_rs::solver::start::{SafeStart, ZeroStart};

static SETTINGS_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let folder = DIRS.data_dir();
    if !folder.is_dir() {
        create_dir(folder)
                .unwrap_or_else(|_| panic!("failed to create directory {:?}", folder))
    }

    folder.join("settings.json")
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    size: SerializableBoardSize,
    texture: Texture,
    solver: KnownSolver
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            size: SerializableBoardSize(ConventionalSize::Beginner.size()),
            texture: Texture::default(),
            solver: KnownSolver::default()
        }
    }
}

#[derive(Clone, Debug, From)]
pub enum Message {
    MenuLabel,
    ChangeSize(BoardSize),
    ChangeTexture(Texture),
    ChangeSolver(KnownSolver)
}

impl Settings {

    fn save(&self) -> io::Result<()> {
        let file = File::create(&*SETTINGS_PATH)?;

        serde_json::to_writer(file, self)?;
        Ok(())
    }

    pub fn load() -> io::Result<Self> {
        let file = File::open(&*SETTINGS_PATH)?;

        let settings = serde_json::from_reader(file)?;

        Ok(settings)
    }

    pub fn size(&self) -> BoardSize {
        self.size.0
    }

    pub fn texture(&self) -> Texture {
        self.texture
    }

    pub fn solver(&self) -> Rc<dyn Solver> {
        self.solver.into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ChangeSize(size) => self.size = SerializableBoardSize(size),
            Message::ChangeTexture(texture) => self.texture = texture,
            Message::ChangeSolver(solver) => self.solver = solver,
            Message::MenuLabel => {}
        }

        if let Err(e) = self.save() {
            eprintln!("failed to save settings data: {}", e);
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(menu_bar!(
            (menu_label("Size"), menu!(
                (menu_button("Beginner", ConventionalSize::Beginner.size())),
                (menu_button("Intermediate", ConventionalSize::Intermediate.size())),
                (menu_button("Expert", ConventionalSize::Expert.size())),
            ).max_width(100.0)),
            (menu_label("Theme"), menu!(
                (menu_radio("Dark", Texture::Dark, self.texture)),
                (menu_radio("Light", Texture::Light, self.texture)),
                (menu_radio("Gay", Texture::Gay, self.texture)),
            ).max_width(100.0)),
            (menu_label("Solver"), menu!(
                (menu_radio("MiaSolver", KnownSolver::MiaSolver, self.solver)),
                (menu_radio("SafeStart", KnownSolver::SafeStart, self.solver)),
                (menu_radio("ZeroStart", KnownSolver::ZeroStart, self.solver)),
            ).max_width(100.0)),
        ).close_on_background_click_global(true)
                .close_on_item_click_global(true)).into()
    }
}

fn menu_label<'a>(content: impl Into<Element<'a, Message>>) -> button::Button<'a, Message> {
    button(content)
            .on_press(Message::MenuLabel)
            .style(|theme, status| {
                use iced::widget::button::{Status, Style};

                let palette = theme.extended_palette();
                let base = Style {
                    text_color: palette.background.base.text,
                    border: Border::default(),
                    ..Style::default()
                };
                match status {
                    Status::Active => base.with_background(Color::TRANSPARENT),
                    Status::Hovered => base.with_background(Color::from_rgb(
                        palette.primary.weak.color.r * 1.2,
                        palette.primary.weak.color.g * 1.2,
                        palette.primary.weak.color.b * 1.2,
                    )),
                    Status::Disabled => base.with_background(Color::from_rgb(0.5, 0.5, 0.5)),
                    Status::Pressed => base.with_background(palette.primary.weak.color),
                    // Status::Disabled => base.with_background(Color::from_rgb(1.0, 0.0, 0.0)),
                    // Status::Pressed => base.with_background(Color::from_rgb(0.0, 1.0, 0.0)),
                    // _ => iced::widget::button::primary(theme, status)
                }
            })
}

fn menu_button<'a>(content: impl Into<Element<'a, Message>>, message: impl Into<Message>) -> button::Button<'a, Message> {
    button(content)
            .on_press(message.into())
            .style(|theme, status| {
                use iced::widget::button::{Status, Style};

                let palette = theme.extended_palette();
                let base = Style {
                    text_color: palette.background.base.text,
                    border: Border::default().rounded(6.0),
                    ..Style::default()
                };
                match status {
                    Status::Active => base.with_background(Color::TRANSPARENT),
                    Status::Hovered => base.with_background(Color::from_rgb(
                        palette.primary.weak.color.r * 1.2,
                        palette.primary.weak.color.g * 1.2,
                        palette.primary.weak.color.b * 1.2,
                    )),
                    Status::Disabled => base.with_background(Color::from_rgb(0.5, 0.5, 0.5)),
                    Status::Pressed => base.with_background(palette.primary.weak.color),
                    // Status::Disabled => base.with_background(Color::from_rgb(1.0, 0.0, 0.0)),
                    // Status::Pressed => base.with_background(Color::from_rgb(0.0, 1.0, 0.0)),
                    // _ => iced::widget::button::primary(theme, status)
                }
            })
            .width(Length::Fill)
}

fn menu_radio<'a, T: Into<Message> + Copy + Eq>(label: impl Into<String>, value: T, selected: T) -> radio::Radio<'a, Message> {
    radio(label, value, Some(selected), Into::into)
            .width(Length::Fill)
}

#[derive(Debug)]
struct SerializableBoardSize(BoardSize);

impl Serialize for SerializableBoardSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
            S: Serializer
    {
        let mut s = serializer.serialize_struct("BoardSize", 3)?;

        s.serialize_field("width", &self.0.width())?;
        s.serialize_field("height", &self.0.height())?;
        s.serialize_field("mines", &self.0.mines())?;

        s.end()
    }
}

impl<'de> Deserialize<'de> for SerializableBoardSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
            D: Deserializer<'de>
    {
        struct Mewo;
        impl<'de> Visitor<'de> for Mewo {
            type Value = SerializableBoardSize;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "a struct with fields width height and mines")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                    A: MapAccess<'de>,
            {
                let mut width = None;
                let mut height = None;
                let mut mines = None;

                while let Some((key, value)) = map.next_entry::<String, _>()? {
                    match key.as_str() {
                        "width" => width = Some(value),
                        "height" => height = Some(value),
                        "mines" => mines = Some(value),
                        _ => return Err(de::Error::unknown_field(&key, &["width", "height", "mines"]))
                    }
                }

                BoardSize::new(
                    width.ok_or(de::Error::missing_field("width"))?,
                    height.ok_or(de::Error::missing_field("height"))?,
                    mines.ok_or(de::Error::missing_field("mines"))?)
                        .map_err(de::Error::custom)
                        .map(SerializableBoardSize)
            }
        }
        deserializer.deserialize_struct("BoardSize", &["width", "height", "mines"], Mewo)
    }
}


#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
enum KnownSolver {
    #[default]
    MiaSolver,
    SafeStart,
    ZeroStart
}

impl From<KnownSolver> for Rc<dyn Solver> {
    fn from(value: KnownSolver) -> Self {
        match value {
            KnownSolver::MiaSolver => Rc::new(MiaSolver),
            KnownSolver::SafeStart => Rc::new(SafeStart),
            KnownSolver::ZeroStart => Rc::new(ZeroStart)
        }
    }
}