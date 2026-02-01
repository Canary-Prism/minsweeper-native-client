use derive_more::From;
use directories::ProjectDirs;
use iced::{widget, Element, Subscription, Task};
use iced_core::{mouse, Event};
use std::sync::LazyLock;

mod minsweeper;
mod settings_menu;
mod texture;

static DIRS: LazyLock<ProjectDirs> = LazyLock::new(||
        ProjectDirs::from("", "canaryprism", "minsweeper-native-client")
                .expect("couldn't obtain project directories"));

fn main() -> iced::Result {
    iced::application(State::init, State::update, State::view)
            .subscription(State::subscriptions)
            .run()
}

#[derive(Debug)]
pub struct State {
    settings_menu: settings_menu::SettingsMenu,
    minsweeper: minsweeper::MinsweeperGame,
}

impl Default for State {
    fn default() -> Self {
        let settings_menu = settings_menu::SettingsMenu::default();
        Self {
            minsweeper: make_game(settings_menu.settings()),
            settings_menu,
        }
    }
}

#[derive(Clone, Debug, From)]
pub enum Message {
    Settings(settings_menu::Message),
    Minsweeper(minsweeper::Message),
}

impl State {

    pub fn init() -> (Self, Task<Message>) {
        (Self::default(), Task::done(minsweeper::Message::Restart).map(Into::into))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Settings(e) => {
                let task = self.settings_menu.update(e.clone());
                use settings_menu::Message::*;
                match e {
                    ChangeSize(_) | ChangeSolver(_) => {
                        self.minsweeper = make_game(self.settings_menu.settings());
                        return Task::done(minsweeper::Message::Restart)
                                .map(Into::into)
                    }
                    ChangeTexture(texture) => {
                        self.minsweeper.change_textures(texture)
                    }
                    Auto(_) | ChangeAutoSolver(_) | ChangeAutoDelay(_) => {
                        self.minsweeper.set_auto(self.settings_menu.settings().auto().cloned())
                    }
                    FlagChord(value) => {
                        self.minsweeper.set_flag_chord(value)
                    }
                    HoverChord(value) => {
                        self.minsweeper.set_hover_chord(value)
                    }
                    _ => {}
                }
                task.map(Into::into)
            }
            Message::Minsweeper(e) => {
                self.minsweeper.update(e)
                        .map(Into::into)
            }
        }
    }


    fn subscriptions(&self) -> Subscription<Message> {
        iced::event::listen()
                .filter_map(|e| if let Event::Mouse(mouse::Event::ButtonReleased(e)) = e {
                    Some(minsweeper::Message::MouseRelease(e).into())
                } else {
                    None
                })
    }

    pub fn view(&self) -> Element<'_, Message> {
        let base = widget::column![
            self.settings_menu.view().map(Into::into),
            self.minsweeper.view().map(Into::into),
        ];
        self.process_dialog(base)
    }

    pub fn process_dialog<'a>(&self, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
        let mut view = content.into();
        for dialog in self.dialogs() {
            view = iced_dialog::dialog(true, view, dialog)
                    .into();
        }
        view
    }

    pub fn dialogs<'a>(&self) -> impl Iterator<Item = Element<'a, Message>> {
        let mut vec = vec![];

        vec.append(&mut self.settings_menu.dialogs()
                .map(|e| e.map(Into::into)).collect());

        vec.into_iter()
    }
}

fn make_game(settings: &settings_menu::Settings) -> minsweeper::MinsweeperGame {
    minsweeper::MinsweeperGame::new(settings.size(), settings.solver(), settings.texture(), settings.auto().cloned(), settings.flag_chord(), settings.hover_chord())
}