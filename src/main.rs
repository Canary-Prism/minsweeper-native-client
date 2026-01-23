use std::rc::Rc;
use derive_more::From;
use directories::ProjectDirs;
use iced::{widget, Element, Subscription};
use iced_core::{mouse, Event};
use minsweeper_rs::solver::mia::MiaSolver;
use std::sync::LazyLock;

mod minsweeper;
mod settings_menu;
mod texture;

static DIRS: LazyLock<ProjectDirs> = LazyLock::new(||
        ProjectDirs::from("", "canaryprism", "minsweeper-native-client")
                .expect("couldn't obtain project directories"));

fn main() -> iced::Result {
    iced::application(State::default, State::update, State::view)
            .subscription(State::subscriptions)
            .run()
}

#[derive(Debug)]
pub struct State {
    settings: settings_menu::Settings,
    minsweeper: minsweeper::MinsweeperGame,
}

impl Default for State {
    fn default() -> Self {
        let settings = settings_menu::Settings::load()
                .unwrap_or_else(|e| {
                    eprintln!("failed to load settings: {}", e);
                    settings_menu::Settings::default()
                });
        Self {
            minsweeper: make_game(&settings),
            settings
        }
    }
}

#[derive(Clone, Debug, From)]
pub enum Message {
    Settings(settings_menu::Message),
    Minsweeper(minsweeper::Message)
}

impl State {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Settings(e) => {
                self.settings.update(e.clone());
                use settings_menu::Message::*;
                match e {
                    MenuLabel => {}
                    ChangeSize(_) | ChangeSolver(..) => {
                        self.minsweeper = make_game(&self.settings)
                    }
                    ChangeTexture(texture) => {
                        self.minsweeper.change_textures(texture)
                    }
                }
            }
            Message::Minsweeper(e) => {
                self.minsweeper.update(e)
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
        widget::column![
            self.settings.view().map(Into::into),
            self.minsweeper.view().map(Into::into)
        ].into()
    }
}

fn make_game(settings: &settings_menu::Settings) -> minsweeper::MinsweeperGame {
    minsweeper::MinsweeperGame::new(settings.size(), settings.solver(), settings.texture())
}