use futures_util::FutureExt;
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
use minsweeper_rs::solver::{Move, Operation, Solver};
use minsweeper_rs::{CellState, CellType, GameStatus};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use futures_util::future::{AbortHandle, AbortRegistration, Aborted};
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
    auto: bool,
    flag_chord: bool,
    hover_chord: bool,
    cells: grid::Grid<cell::Cell>,
    handles: Arc<Mutex<HashMap<Uuid, AbortHandle>>>,
    autoing: Arc<AtomicBool>,
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
    Repaint,
}

impl MinsweeperGame {

    pub fn new(size: BoardSize, solver: SolverType, texture: Texture, auto: bool, flag_chord: bool, hover_chord: bool) -> Self {
        let game = AsyncMinsweeperGame::new(size,
                                                                      (|| {}) as fn(), (|| {}) as fn());
        let game = Arc::new(game);
        let cells = grid::Grid::new(size.width().get(), size.height().get(),
                                    |point| cell::Cell::new(point, texture, game.clone()));
        Self {
            game,
            size,
            solver,
            texture,
            auto,
            flag_chord,
            hover_chord,
            cells,
            handles: Default::default(),
            autoing: Default::default(),
        }
    }

    pub fn change_textures(&mut self, texture: Texture) {
        self.texture = texture;
        for cell in &mut self.cells {
            cell.texture = texture;
        }
    }

    pub fn set_auto(&mut self, auto: bool) {
        self.auto = auto;
    }

    pub fn set_flag_chord(&mut self, flag_chord: bool) {
        self.flag_chord = flag_chord;
    }

    pub fn set_hover_chord(&mut self, hover_chord: bool) {
        self.hover_chord = hover_chord;
    }


    pub fn points(&self) -> impl Iterator<Item = Point> {
        (0..self.size.height().into())
                .flat_map(|y| (0..self.size.width().into())
                        .map(move |x| (x, y)))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Cell((point, e)) => {
                if self.game.blocking_gamestate().status != GameStatus::Playing {
                    return Task::none();
                }
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
            Message::Repaint => {}
        }
        Task::none()
    }

    fn left_click(&self, point: Point) -> Task<Message> {
        let cell = &self.cells[point];
        let revealings = if matches!(self.game.blocking_gamestate().board[point].cell_type, CellType::Safe(_)) {
            self.size.neighbours(point)
                    .map(|point| self.cells[point].revealing.clone())
                    .collect()
        } else {
            vec![cell.revealing.clone()]
        };

        for revealing in &revealings {
            revealing.fetch_add(1, Ordering::Relaxed);
        }

        let game = self.game.clone();
        let flag_chord = self.flag_chord;

        let mut handles_lock = self.handles.blocking_lock();

        let (abortable, handle) = futures_util::future::abortable(async move {
            left_click(&game, point, flag_chord).await
        });
        let task = Task::future(abortable);
        let uuid = Uuid::new_v4();

        let handles = self.handles.clone();
        let mut task = task.then(move |_| {
            let handles = handles.clone();
            let revealings = revealings.clone();
            Task::future(async move {
                for revealing in revealings {
                    revealing.fetch_sub(1, Ordering::Relaxed);
                }
                let mut handles = handles.lock().await;
                handles.remove(&uuid);
            })
        });


        handles_lock.insert(uuid, handle);


        drop(handles_lock);
        if self.auto && !self.autoing.fetch_or(true, Ordering::Relaxed) {
            let solver = self.solver.clone();
            let game = self.game.clone();
            let handles = self.handles.clone();
            let autoing = self.autoing.clone();


            task = task.then(move |_| Self::auto_task(solver.clone(), game.clone(), handles.clone(), autoing.clone()).map(|_| ()))
        }

        task.map(|_| Message::Repaint)
    }

    fn auto_task(solver: SolverType, game: MinsweeperType, handles: Arc<Mutex<HashMap<Uuid, AbortHandle>>>, autoing: Arc<AtomicBool>) -> Task<Message> {
        #[derive(Debug)]
        enum Phase {
            Start, SolveNext(Uuid), End(Uuid)
        }
        Task::stream(futures_util::stream::unfold(Phase::Start, move |phase| {
            let handles = handles.clone();
            let game = game.clone();
            let solver = solver.clone();
            let autoing = autoing.clone();
            async move {
                let handles = handles.clone();
                let game = game.clone();
                if let Phase::SolveNext(uuid) | Phase::End(uuid) = phase {
                    handles.lock().await.remove(&uuid);
                }

                if matches!(phase, Phase::SolveNext(_)) {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }

                let phase = if matches!(phase, Phase::Start | Phase::SolveNext(_)) {
                    let uuid = Uuid::new_v4();
                    let (abortable, handle) = futures_util::future::abortable(async move {
                        let gamestate = game.gamestate().await;
                        let Some(Move { actions, .. }) = solver.solve(&gamestate) else {
                            return false
                        };

                        for action in actions {
                            match action.operation {
                                Operation::Reveal | Operation::Chord => left_click(&game, action.point, false).await,
                                Operation::Flag => right_click(&game, action.point).await,
                            }
                        }

                        true
                    });
                    handles.lock().await.insert(uuid, handle);

                    match abortable.await {
                        Ok(true) => Some(Phase::SolveNext(uuid)),
                        _ => Some(Phase::End(uuid))
                    }
                } else {
                    autoing.store(false, Ordering::Relaxed);
                    None
                };

                phase.map(|phase| ((), phase))
            }
        })).map(|_| Message::Repaint)
        // let mewo = futures_util::stream::unfold(Phase::Start, move |phase| {
        //     let game = game.clone();
        //     let handles = handles.clone();
        //     match phase {
        //         Phase::Start => {
        //             let task = async move {
        //
        //                 let uuid = Uuid::new_v4();
        //                 let (abortable, handle) = futures_util::future::abortable(async move {
        //                     let gamestate = game.gamestate().await;
        //                     let Some(Move { actions, .. }) = solver.solve(&gamestate) else {
        //                         return Phase::End(uuid)
        //                     };
        //
        //                     for action in actions {
        //                         match action.operation {
        //                             Operation::Reveal | Operation::Chord => left_click(&game, action.point, false).await,
        //                             Operation::Flag => right_click(&game, action.point).await,
        //                         }
        //                     }
        //
        //                     Phase::SolveNext(uuid)
        //                 });
        //                 let future = abortable
        //                         .map(|e| {
        //                             match e {
        //                                 Ok(e) => e,
        //                                 _ => Phase::Nothing
        //                             }
        //                         });
        //
        //                 handles.lock().await.insert(uuid, handle);
        //
        //                 future
        //             };
        //
        //             Some(task.then(|e| e))
        //         }
        //         Phase::SolveNext(uuid) => {
        //             let task = async move {
        //                 handles.lock().await.remove(&uuid);
        //                 let uuid = Uuid::new_v4();
        //                 (async {
        //                     let gamestate = game.gamestate().await;
        //                     let Some(Move { actions, .. }) = solver.solve(&gamestate) else {
        //                         return Phase::End(uuid)
        //                     };
        //
        //                     for action in actions {
        //                         match action.operation {
        //                             Operation::Reveal | Operation::Chord => left_click(&game, action.point, false).await,
        //                             Operation::Flag => right_click(&game, action.point).await,
        //                         }
        //                     }
        //
        //                     Phase::SolveNext(uuid)
        //                 })
        //             };
        //
        //             Some(task.then(|e| e))
        //         }
        //         Phase::End(uuid) => {
        //             Some(Task::future(async move {
        //                 handles.lock().await.remove(&uuid);
        //                 Phase::Nothing
        //             }))
        //         }
        //         Phase::Nothing => {
        //             None
        //         }
        //     }
        //     // if let Some(uuid) = phase {
        //     //     let task = Task::future(async move {
        //     //         handles.lock().await.remove(uuid);
        //     //         Task::future(async {
        //     //             let gamestate = game.gamestate().await;
        //     //             let Some(Move { actions, .. }) = solver.solve(&gamestate) else {
        //     //                 return None
        //     //             };
        //     //
        //     //             for action in actions {
        //     //                 match action.operation {
        //     //                     Operation::Reveal | Operation::Chord => left_click(&game, action.point, false).await,
        //     //                     Operation::Flag => right_click(&game, action.point).await,
        //     //                 }
        //     //             }
        //     //
        //     //             Some(uuid)
        //     //         })
        //     //     });
        //     //
        //     //     task.then(|e| e)
        //     // } else {
        //     //     Task::future(async move {
        //     //         handles.lock().await.remove(&phase);
        //     //         None
        //     //     })
        //     // }
        // });
        // let (abortable, handle) = futures_util::future::abortable(async move {
        //     // while let Some(Move { actions, .. }) = solver.solve(&game.gamestate().await) {
        //     //     for action in actions {
        //     //         match action.operation {
        //     //             Operation::Reveal | Operation::Chord => left_click(&game, action.point, false).await,
        //     //             Operation::Flag => right_click(&game, action.point).await,
        //     //         }
        //     //     }
        //     // }
        //
        //     loop {
        //         let gamestate = game.gamestate().await;
        //         let Some(Move { actions, .. }) = solver.solve(&gamestate) else {
        //             break
        //         };
        //
        //         for action in actions {
        //             match action.operation {
        //                 Operation::Reveal | Operation::Chord => left_click(&game, action.point, false).await,
        //                 Operation::Flag => right_click(&game, action.point).await,
        //             }
        //         }
        //
        //         interval.tick().await;
        //     }
        // });
        // (Task::future(abortable).map(|_| Message::Repaint), handle)
    }

    fn right_click(&self, point: Point) -> Task<Message> {
        let game = self.game.clone();

        let mut handles_lock = self.handles.blocking_lock();

        let (abortable, handle) = futures_util::future::abortable(async move {
            right_click(&game, point).await
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

        task.map(|_| Message::Repaint)
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
                    return self.right_click(point);
                }
            }
            cell::Message::SelfRelease(button) => {
                if cell.pressed && matches!(button, mouse::Button::Left) {
                    return self.left_click(point)
                }
                cell.pressed = false;
            }
            cell::Message::ForceArmed(value) => {
                cell.force = value;
            }
            cell::Message::Revealing(value) => {
                cell.revealing.fetch_add(if value { 1 } else { -1 }, Ordering::Relaxed);
            }
            cell::Message::Enter => {
                cell.hovering = true;
                if self.hover_chord && matches!(self.game.blocking_gamestate().board[point].cell_type, CellType::Safe(n) if n > 0) {
                    return self.left_click(point);
                }
            }
            cell::Message::Exit => {
                cell.hovering = false
            }
        }


        if matches!(message, cell::Message::Press(_) | cell::Message::Release(_) | cell::Message::SelfPress(_) | cell::Message::SelfRelease(_) | cell::Message::Enter | cell::Message::Exit) {
            let down = cell.is_down();
            if (cell.pressed || matches!(message, cell::Message::Press(_) | cell::Message::Release(_) | cell::Message::SelfPress(_) | cell::Message::SelfRelease(_)))
                    && matches!(self.game.blocking_gamestate().board[point].cell_type, CellType::Safe(_)) {
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

async fn left_click(game: &MinsweeperType, point: Point, flag_chord: bool) {
    let gamestate = game.gamestate().await;
    if flag_chord
            && let CellType::Safe(n) = gamestate.board[point].cell_type
            && n as usize == gamestate.board.size()
                    .neighbours(point)
                    .filter(|point| matches!(gamestate.board[*point].cell_type, CellType::Unknown))
                    .count() {
        for point in gamestate.board.size()
                .neighbours(point).filter(|point| matches!(gamestate.board[*point].cell_state, CellState::Unknown)) {
            right_click(game, point).await;
        }
    }

    _ = game.left_click(point).await;
}

async fn right_click(game: &MinsweeperType, point: Point) {
    _ = game.right_click(point).await;
}

impl Drop for MinsweeperGame {
    fn drop(&mut self) {
        for (_, handle) in self.handles.blocking_lock().iter() {
            handle.abort();
        }
    }
}