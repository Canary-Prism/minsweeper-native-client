use std::cmp::max;
use futures_util::FutureExt;
mod cell;
mod grid;
mod restart;

use crate::texture::{Border, Texture};
use derive_more::From;
use iced::widget::{button, container, responsive, row, svg, text, Grid, Row, Svg};
use iced::{widget, Element, Task};
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::{mouse, Background, Color, ContentFit, Length, Padding, Size};
use minsweeper_rs::board::{BoardSize, Point};
use minsweeper_rs::minsweeper::nonblocking::AsyncMinsweeperGame;
use minsweeper_rs::solver::{Move, Operation, Solver};
use minsweeper_rs::{CellState, CellType, GameStatus};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use formatx::formatx;
use futures_util::future::{AbortHandle, AbortRegistration, Aborted};
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::minsweeper::restart::RestartButton;
use crate::settings_menu::{Auto, KnownSolver};

pub type MinsweeperType = Arc<AsyncMinsweeperGame<SolverType, FnType, FnType>>;
pub type SolverType = Arc<dyn Solver + Send + Sync>;
pub type FnType = fn();

pub struct MinsweeperGame {
    game: MinsweeperType,
    size: BoardSize,
    solver: SolverType,
    texture: Texture,
    auto: Option<Auto>,
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

    pub fn new(size: BoardSize, solver: SolverType, texture: Texture, auto: Option<Auto>, flag_chord: bool, hover_chord: bool) -> Self {
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

    pub fn set_auto(&mut self, auto: Option<Auto>) {
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
                }).map(|_| Message::Repaint)
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
            revealing.store(true, Ordering::Relaxed);
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
                    revealing.store(false, Ordering::Relaxed);
                }
                let mut handles = handles.lock().await;
                handles.remove(&uuid);
            })
        });


        handles_lock.insert(uuid, handle);


        drop(handles_lock);
        if let Some(auto) = &self.auto && !self.autoing.fetch_or(true, Ordering::Relaxed) {
            let solver = auto.solver()
                    .map(Into::into)
                    .unwrap_or(self.solver.clone());
            let game = self.game.clone();
            let handles = self.handles.clone();
            let delay = auto.delay();
            let autoing = self.autoing.clone();


            task = task.then(move |_| Self::auto_task(solver.clone(), game.clone(), handles.clone(), delay, autoing.clone()).map(|_| ()))
        }

        task.map(|_| Message::Repaint)
    }

    fn auto_task(solver: SolverType, game: MinsweeperType, handles: Arc<Mutex<HashMap<Uuid, AbortHandle>>>, delay: Duration, autoing: Arc<AtomicBool>) -> Task<Message> {
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
                    tokio::time::sleep(delay).await;
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
                if self.game.blocking_gamestate().board[point].cell_state == CellState::Unknown
                        || matches!(self.game.blocking_gamestate().board[point].cell_type, CellType::Safe(1..)) {
                    cell.pressed = true;
                }
            }
            cell::Message::Release(_button) => {
                cell.pressed = false;
            }
            cell::Message::SelfPress(button) => {
                if self.game.blocking_gamestate().board[point].cell_state == CellState::Unknown
                        || matches!(self.game.blocking_gamestate().board[point].cell_type, CellType::Safe(1..)) {
                    cell.pressed = true;
                }

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
                cell.revealing.store(value, Ordering::Relaxed);
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

        container(widget::column![
            container(
                row![
                    self.border(Border::LeftRight)
                        .height(32),
                    container(self.number_display(self.remaining_mines(), self.remaining_mine_digit()))
                        .padding(Padding::default().horizontal(10)),
                    Element::new(RestartButton::new(self.texture, self.game.blocking_gamestate().status, self.any_revealing(), Message::Restart)),
                ].align_y(Vertical::Center)
            ).width(Length::Fill).align_x(Horizontal::Center),
            responsive(|size|
                row![
                    self.border(Border::LeftRight)
                            .height(size.height),
                    responsive(|size|

                        container(Grid::from_iter(self.points()
                            .map(|point| (point, &self.cells[point]))
                            .map(|(point, e)| e.view()
                                .map(move |message| Message::Cell((point, message)))))
                            .columns(self.size.width().get())
                            .width(self.cell_size(size) * self.size.width().get() as f32))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                        .into()),
                    self.border(Border::LeftRight)
                            .height(size.height),
                ].into()
            ),

        ]).style(|_theme| container::Style {
            background: Some(Background::Color(self.texture.get_background_colour())),
            ..Default::default()
        }).into()
    }

    fn cell_size(&self, size: Size) -> f32 {
        f32::min(size.width / self.size.width().get() as f32, size.height / self.size.height().get() as f32)
    }


    fn remaining_mines(&self) -> isize {
        let gamestate = self.game.blocking_gamestate();
        match gamestate.status {
            GameStatus::Playing | GameStatus::Lost => gamestate.remaining_mines,
            GameStatus::Won => 0,
            GameStatus::Never => self.size.mines().get() as isize
        }
    }

    fn remaining_mine_digit(&self) -> usize {
        let size = self.size;
        max(size.mines().to_string().len(),
            (size.mines().get() as isize - size.width().get() as isize * size.height().get() as isize).to_string().len())
    }

    fn number_display(&'_ self, number: isize, length: usize) -> Row<'_, Message> {
        const NUMBER_SIZE_MULTIPLIER: u32 = 2;
        row(formatx!(format!("{{:0{}}}", length).as_str(), number)
                .expect("number display should never fail")
                .chars()
                .map(|c|
                        svg(svg::Handle::from_memory(self.texture.get_digit(c)))
                                .width(13 * NUMBER_SIZE_MULTIPLIER)
                                .height(23 * NUMBER_SIZE_MULTIPLIER)
                                .into()))
    }

    fn any_revealing(&self) -> bool {
        self.cells
                .iter()
                .any(|cell| cell.is_down())
    }

    fn border(&'_ self, border: Border) -> Svg<'_> {
        let svg = svg(svg::Handle::from_memory(self.texture.get_border(border)));

        let (width, height) = match border {
            Border::TopLeft | Border::TopBottom | Border::TopRight | Border::BottomLeft | Border::BottomRight
            | Border::LeftRight | Border::MiddleLeft | Border::MiddleRight => (120, 120),
            Border::CounterLeft | Border::CounterRight => (10, 270),
            Border::CounterTop | Border::CounterBottom => (130, 10),
        };

        svg.content_fit(ContentFit::Fill)
                .width(width / 5)
                .height(height / 5)
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