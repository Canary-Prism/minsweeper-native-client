use std::fmt::{Debug, Formatter};
use crate::minsweeper::MinsweeperType;
use crate::texture::Texture;
use iced::widget::{mouse_area, svg};
use iced::{mouse, Element};
use minsweeper_rs::solver::Operation;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;

pub struct Cell {
    game: MinsweeperType,
    pub texture: Texture,
    point: minsweeper_rs::board::Point,
    pub hovering: bool,
    pub pressed: bool,
    pub force: bool,
    pub revealing: Arc<AtomicI32>
}

impl Debug for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} hovering: {}, pressed: {}, force: {}, revealing: {:?}",
               self.point, self.hovering, self.pressed, self.force, self.revealing)
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Press(mouse::Button),
    Release(mouse::Button),
    SelfPress(mouse::Button),
    SelfRelease(mouse::Button),
    ForceArmed(bool),
    Revealing(bool),
    Enter,
    Exit
}

impl Message {
    pub fn to_action(&self) -> Option<Vec<Operation>> {
        match self {
            Message::SelfPress(mouse::Button::Right) => Some(vec![Operation::Flag]),
            Message::SelfRelease(mouse::Button::Left) => Some(vec![Operation::Reveal, Operation::Chord]),
            _ => None
        }
    }

    pub fn is_left_click(&self) -> bool {
        matches!(self, Message::SelfRelease(mouse::Button::Left))
    }
    pub fn is_right_click(&self) -> bool {
        matches!(self, Message::SelfPress(mouse::Button::Right))
    }
}

impl Cell {

    pub fn new(point: minsweeper_rs::board::Point, texture: Texture, game: MinsweeperType) -> Self {
        Self { game, texture, point, hovering: false, pressed: false, force: false, revealing: Default::default() }
    }

    // pub fn update(&mut self, message: Message) {
    //     match message {
    //         Message::Press(_button) => {
    //             self.pressed = true;
    //         }
    //         Message::Release(_button) => {
    //             self.pressed = false;
    //         }
    //         Message::SelfPress(button) => {
    //             self.pressed = true;
    //             if matches!(button, mouse::Button::Right) {
    //                 // _ = self.game.borrow_mut().right_click(self.point);
    //                 self.pressed = false;
    //             }
    //         }
    //         Message::SelfRelease(button) => {
    //             if self.pressed && matches!(button, mouse::Button::Left) {
    //                 // _ = self.game.borrow_mut().left_click(self.point);
    //             }
    //             self.pressed = false;
    //         }
    //         Message::ForceArmed(value) => {
    //             self.force = value;
    //         }
    //         Message::Revealing(value) => {
    //             self.revealing.store(value, Ordering::Relaxed)
    //         }
    //         Message::Enter => {
    //             self.hovering = true
    //         }
    //         Message::Exit => {
    //             self.hovering = false
    //         }
    //     }
    // }

    pub fn is_down(&self) -> bool {
        (self.pressed && self.hovering) || self.revealing.load(Ordering::Relaxed) > 0
    }

    fn is_armed(&self) -> bool {
        self.is_down() || self.force
    }

    // fn subscriptions() -> Subscription<Message> {
    //
    // }

    pub fn view(&self) -> Element<'_, Message> {
        mouse_area(svg(svg::Handle::from_memory(
            self.texture.get_cell_asset(self.game.blocking_gamestate().board[self.point], self.is_armed()))))
                .on_press(Message::Press(mouse::Button::Left))
                .on_middle_press(Message::SelfPress(mouse::Button::Middle))
                .on_right_press(Message::SelfPress(mouse::Button::Right))
                .on_release(Message::SelfRelease(mouse::Button::Left))
                .on_middle_release(Message::SelfRelease(mouse::Button::Middle))
                .on_right_release(Message::SelfRelease(mouse::Button::Right))
                .on_enter(Message::Enter)
                .on_exit(Message::Exit)
                .into()
    }
}