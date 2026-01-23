use crate::texture::Texture;
use iced::widget::{mouse_area, svg};
use iced::{mouse, Element};
use minsweeper_rs::minsweeper::MinsweeperGame;
use minsweeper_rs::Minsweeper;
use std::cell::RefCell;
use std::rc::Rc;
use minsweeper_rs::solver::Solver;

pub struct Cell {
    game: Rc<RefCell<MinsweeperGame<Rc<dyn Solver>>>>,
    pub texture: Texture,
    point: minsweeper_rs::board::Point,
    hovering: bool,
    pressed: bool,
    force: bool
}

#[derive(Clone, Debug)]
pub enum Message {
    Press(mouse::Button),
    Release(mouse::Button),
    SelfPress(mouse::Button),
    SelfRelease(mouse::Button),
    ForceArmed(bool),
    Enter,
    Exit
}

impl Cell {

    pub fn new(point: minsweeper_rs::board::Point, texture: Texture, game: Rc<RefCell<MinsweeperGame<Rc<dyn Solver>>>>) -> Self {
        Self { game, texture, point, hovering: false, pressed: false, force: false }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Press(button) => {
                self.pressed = true;
            }
            Message::Release(button) => {
                self.pressed = false;
            }
            Message::SelfPress(button) => {
                self.pressed = true;
                if matches!(button, mouse::Button::Right) {
                    _ = self.game.borrow_mut().right_click(self.point);
                    self.pressed = false;
                }
            }
            Message::SelfRelease(button) => {
                if self.pressed && matches!(button, mouse::Button::Left) {
                    _ = self.game.borrow_mut().left_click(self.point);
                }
                self.pressed = false;
            }
            Message::ForceArmed(value) => {
                self.force = value;
            }
            Message::Enter => {
                self.hovering = true
            }
            Message::Exit => {
                self.hovering = false
            }
        }
    }

    pub fn is_down(&self) -> bool {
        self.pressed && self.hovering
    }

    fn is_armed(&self) -> bool {
        self.is_down() || self.force
    }

    // fn subscriptions() -> Subscription<Message> {
    //
    // }

    pub fn view(&self) -> Element<'_, Message> {
        mouse_area(svg(svg::Handle::from_memory(
            self.texture.get_cell_asset(self.game.borrow().gamestate().board[self.point], self.is_armed()))))
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