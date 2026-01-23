mod cell;
mod grid;

use crate::texture::Texture;
use derive_more::From;
use iced::widget::{button, container, responsive, row, svg, text, Grid};
use iced::{widget, Element};
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::{mouse, Length, Padding, Size};
use minsweeper_rs::board::{BoardSize, Point};
use minsweeper_rs::solver::Solver;
use minsweeper_rs::{CellType, Minsweeper};
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::str::FromStr;

pub struct MinsweeperGame {
    game: Rc<RefCell<minsweeper_rs::minsweeper::MinsweeperGame<Rc<dyn Solver>>>>,
    size: BoardSize,
    solver: Rc<dyn Solver>,
    texture: Texture,
    cells: grid::Grid<cell::Cell>
}

impl Debug for MinsweeperGame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Minsweeper")
    }
}

#[derive(Clone, Debug, From)]
pub enum Message {
    Restart,
    Cell((Point, cell::Message)),
    MouseRelease(mouse::Button)
}

impl MinsweeperGame {

    pub fn new(size: BoardSize, solver: Rc<dyn Solver>, texture: Texture) -> Self {
        let mut game = minsweeper_rs::minsweeper::MinsweeperGame::new(size, Box::new(|| {}), Box::new(|| {}));
        game.start_with_solver(solver.clone());
        let game = Rc::new(RefCell::new(game));
        let cells = grid::Grid::new(size.width().get(), size.height().get(),
                                    |point | cell::Cell::new(point, texture, game.clone()));
        Self {
            game,
            size,
            solver,
            texture,
            cells
        }
    }

    pub fn change_textures(&mut self, texture: Texture) {
        self.texture = texture;
        for cell in &mut self.cells {
            cell.texture = texture;
        }
    }

    pub fn points(&self) -> impl Iterator<Item = Point> {
        (0..self.size.height().into())
                .flat_map(|y| (0..self.size.width().into())
                        .map(move |x| (x, y)))
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Cell((point, e)) => {
                self.cells[point].update(e.clone());
                // if matches!(e, cell::Message::SelfRelease(_) | cell::Message::SelfPress(_)) {
                //
                // }
                let down = self.cells[point].is_down();
                if matches!(Minsweeper::gamestate(&*self.game.borrow())
                            .board[point].cell_type, CellType::Safe(_)) {
                    for neighbour in self.size.neighbours(point) {
                        self.cells[neighbour].update(cell::Message::ForceArmed(down))
                    }
                }
            }
            Message::MouseRelease(button) => for cell in &mut self.cells {
                cell.update(cell::Message::Release(button))
            },
            Message::Restart => {
                self.game.borrow_mut().start_with_solver(self.solver.clone());
            }
        }
    }

    // fn subscriptions() -> Subscription<Message> {
    //
    // }

    pub fn view(&self) -> Element<'_, Message> {

        widget::column![
            container(row![
                container(text!("remaining mines: {}", self.game.borrow().gamestate().remaining_mines))
                    .padding(Padding::default().horizontal(10)),
                button(svg(svg::Handle::from_memory(
                        self.texture.get_restart_button(self.game.borrow().gamestate().status, false, false))))
                    .width(Length::Fixed(70.0))
                    .height(Length::Fixed(70.0))
                    .on_press(Message::Restart)
            ]).width(Length::Fill).align_x(Horizontal::Center),
            responsive(|size| container(Grid::from_iter(self.points()
                .map(|point| (point, &self.cells[point]))
                .map(|(point, e)| e.view()
                    .map(move |message| Message::Cell((point, message)))))
                .columns(self.size.width().get())
                .width(self.cell_size(size) * self.size.width().get() as f32))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into())
        ].into()
    }

    fn cell_size(&self, size: Size) -> f32 {
        f32::min(size.width / self.size.width().get() as f32, size.height / self.size.height().get() as f32)
    }

}