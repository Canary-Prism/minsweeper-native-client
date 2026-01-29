mod cell;
mod grid;

use crate::texture::Texture;
use derive_more::From;
use iced::widget::{button, container, responsive, row, svg, text, Grid};
use iced::{widget, Element, Task};
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::{mouse, Length, Padding, Size};
use minsweeper_rs::board::{BoardSize, Point};
use minsweeper_rs::minsweeper::nonblocking::AsyncMinsweeperGame;
use minsweeper_rs::solver::Solver;
use minsweeper_rs::{CellType, Minsweeper};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub type MinsweeperType = Arc<AsyncMinsweeperGame<SolverType, FnType, FnType>>;
pub type SolverType = Arc<dyn Solver + Send + Sync>;
pub type FnType = fn();

pub struct MinsweeperGame {
    game: MinsweeperType,
    size: BoardSize,
    solver: SolverType,
    texture: Texture,
    cells: grid::Grid<cell::Cell>,
    handles: Arc<Mutex<HashMap<Uuid, futures_util::stream::AbortHandle>>>
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
    MouseRelease(mouse::Button),
}

impl MinsweeperGame {

    pub fn new(size: BoardSize, solver: SolverType, texture: Texture) -> Self {
        let mut game = AsyncMinsweeperGame::new(size,
                                                                      (|| {}) as fn(), (|| {}) as fn());
        game.start_with_solver(solver.clone());
        let game = Arc::new(game);
        let cells = grid::Grid::new(size.width().get(), size.height().get(),
                                    |point| cell::Cell::new(point, texture, game.clone()));
        Self {
            game,
            size,
            solver,
            texture,
            cells,
            handles: Default::default()
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

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Cell((point, e)) => {
                // self.cells[point].update(e.clone());
                let task = self.update_cell(point, e);
                // if matches!(e, cell::Message::SelfRelease(_) | cell::Message::SelfPress(_)) {
                //
                // }

                return task
            }
            Message::MouseRelease(button) => for cell in self.size.clone().points() {
                let _ = self.update_cell(cell, cell::Message::Release(button));
            },
            Message::Restart => {
                for (_, handle) in self.handles.blocking_lock().iter() {
                    handle.abort();
                }
                self.handles.blocking_lock().clear();

                let game = self.game.clone();
                let solver = self.solver.clone();
                return Task::future(async move {
                    game.start_with_solver(solver).await
                }).discard()
            }
        }
        Task::none()
    }

    pub fn update_cell(&mut self, point: Point, message: cell::Message) -> Task<Message> {
        let cell = &mut self.cells[point];
        match message {
            cell::Message::Press(_button) => {
                cell.pressed = true;
            }
            cell::Message::Release(_button) => {
                cell.pressed = false;
            }
            cell::Message::SelfPress(button) => {
                cell.pressed = true;

                if matches!(button, mouse::Button::Right) {
                    cell.pressed = false;
                    let game = self.game.clone();

                    let mut handles_lock = self.handles.blocking_lock();

                    let (abortable, handle) = futures_util::future::abortable(async move {
                        _ = game.right_click(point).await;
                    });
                    let task = Task::future(abortable);
                    let uuid = Uuid::new_v4();

                    let handles = self.handles.clone();
                    let task = task.then(move |_| {
                        let handles = handles.clone();
                        Task::future(async move {
                            let handles = handles.lock().await;
                            let Some(handle) = handles.get(&uuid) else { return };
                            handle.abort();
                        })
                    });


                    handles_lock.insert(uuid, handle);

                    return task.discard()
                }
            }
            cell::Message::SelfRelease(button) => {
                if cell.pressed && matches!(button, mouse::Button::Left) {
                    // _ = cell.game.borrow_mut().left_click(cell.point);
                    let revealings = if matches!(self.game.blocking_gamestate().board[point].cell_type, CellType::Safe(_)) {
                        self.size.neighbours(point)
                                .map(|point| self.cells[point].revealing.clone())
                                .collect()
                    } else {
                        vec![cell.revealing.clone()]
                    };

                    for revealing in &revealings {
                        revealing.store(true, Ordering::Relaxed)
                    }

                    let game = self.game.clone();

                    let mut handles_lock = self.handles.blocking_lock();

                    let (abortable, handle) = futures_util::future::abortable(async move {
                        _ = game.left_click(point).await;
                    });
                    let task = Task::future(abortable);
                    let uuid = Uuid::new_v4();

                    let handles = self.handles.clone();
                    let task = task.then(move |_| {
                        let handles = handles.clone();
                        let revealings = revealings.clone();
                        Task::future(async move {
                            for revealing in revealings {
                                revealing.store(false, Ordering::Relaxed)
                            }
                            let mut handles = handles.lock().await;
                            handles.remove(&uuid);
                        })
                    });


                    handles_lock.insert(uuid, handle);

                    return task.discard()

                }
                cell.pressed = false;
            }
            cell::Message::ForceArmed(value) => {
                cell.force = value;
            }
            cell::Message::Revealing(value) => {
                cell.revealing.store(value, Ordering::Relaxed)
            }
            cell::Message::Enter => {
                cell.hovering = true
            }
            cell::Message::Exit => {
                cell.hovering = false
            }
        }
        if matches!(message, cell::Message::Press(_) | cell::Message::Release(_) | cell::Message::SelfPress(_) | cell::Message::SelfRelease(_) | cell::Message::Enter | cell::Message::Exit) {
            let down = cell.is_down();
            if matches!(self.game.blocking_gamestate().board[point].cell_type, CellType::Safe(_)) {
                for neighbour in self.size.clone().neighbours(point) {
                    let _ = self.update_cell(neighbour, cell::Message::ForceArmed(down));
                }
            }
        }
        Task::none()
    }

    // fn subscriptions() -> Subscription<Message> {
    //
    // }

    pub fn view(&self) -> Element<'_, Message> {

        widget::column![
            container(row![
                container(text!("remaining mines: {}", self.game.blocking_gamestate().remaining_mines))
                    .padding(Padding::default().horizontal(10)),
                button(svg(svg::Handle::from_memory(
                        self.texture.get_restart_button(self.game.blocking_gamestate().status, false, false))))
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

impl Drop for MinsweeperGame {
    fn drop(&mut self) {
        for (_, handle) in self.handles.blocking_lock().iter() {
            handle.abort();
        }
    }
}