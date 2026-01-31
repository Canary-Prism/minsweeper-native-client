use iced_core::Color;
use minsweeper_rs::{Cell, CellState, CellType, GameStatus};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Texture {
    #[default]
    Dark,
    Light,
    Gay
}

impl Texture {
    pub fn get_cell_asset(self, cell: Cell, down: bool) -> &'static [u8] {
        use Texture::*;
        match self {
            Dark => match (cell.cell_state, cell.cell_type) {
                (CellState::Revealed, CellType::Safe(0)) => include_bytes!("dark/cell/celldown.svg"),
                (CellState::Revealed, CellType::Safe(1)) => include_bytes!("dark/cell/cell1.svg"),
                (CellState::Revealed, CellType::Safe(2)) => include_bytes!("dark/cell/cell2.svg"),
                (CellState::Revealed, CellType::Safe(3)) => include_bytes!("dark/cell/cell3.svg"),
                (CellState::Revealed, CellType::Safe(4)) => include_bytes!("dark/cell/cell4.svg"),
                (CellState::Revealed, CellType::Safe(5)) => include_bytes!("dark/cell/cell5.svg"),
                (CellState::Revealed, CellType::Safe(6)) => include_bytes!("dark/cell/cell6.svg"),
                (CellState::Revealed, CellType::Safe(7)) => include_bytes!("dark/cell/cell7.svg"),
                (CellState::Revealed, CellType::Safe(8)) => include_bytes!("dark/cell/cell8.svg"),
                (CellState::Revealed, CellType::Safe(_)) => unreachable!(),
                (CellState::Revealed, CellType::Mine) => include_bytes!("dark/cell/blast.svg"),
                (CellState::Revealed, CellType::Unknown) if down => include_bytes!("dark/cell/celldown.svg"),
                (CellState::Revealed, CellType::Unknown) => include_bytes!("dark/cell/cellup.svg"),

                (CellState::Unknown, CellType::Mine) => include_bytes!("dark/cell/cellmine.svg"),
                (CellState::Unknown, _) if down => include_bytes!("dark/cell/celldown.svg"),
                (CellState::Unknown, _) => include_bytes!("dark/cell/cellup.svg"),

                (CellState::Flagged, CellType::Safe(_)) => include_bytes!("dark/cell/falsemine.svg"),
                (CellState::Flagged, _) => include_bytes!("dark/cell/cellflag.svg"),
            }
            Light => match (cell.cell_state, cell.cell_type) {
                (CellState::Revealed, CellType::Safe(0)) => include_bytes!("light/cell/celldown.svg"),
                (CellState::Revealed, CellType::Safe(1)) => include_bytes!("light/cell/cell1.svg"),
                (CellState::Revealed, CellType::Safe(2)) => include_bytes!("light/cell/cell2.svg"),
                (CellState::Revealed, CellType::Safe(3)) => include_bytes!("light/cell/cell3.svg"),
                (CellState::Revealed, CellType::Safe(4)) => include_bytes!("light/cell/cell4.svg"),
                (CellState::Revealed, CellType::Safe(5)) => include_bytes!("light/cell/cell5.svg"),
                (CellState::Revealed, CellType::Safe(6)) => include_bytes!("light/cell/cell6.svg"),
                (CellState::Revealed, CellType::Safe(7)) => include_bytes!("light/cell/cell7.svg"),
                (CellState::Revealed, CellType::Safe(8)) => include_bytes!("light/cell/cell8.svg"),
                (CellState::Revealed, CellType::Safe(_)) => unreachable!(),
                (CellState::Revealed, CellType::Mine) => include_bytes!("light/cell/blast.svg"),
                (CellState::Revealed, CellType::Unknown) if down => include_bytes!("light/cell/celldown.svg"),
                (CellState::Revealed, CellType::Unknown) => include_bytes!("light/cell/cellup.svg"),

                (CellState::Unknown, CellType::Mine) => include_bytes!("light/cell/cellmine.svg"),
                (CellState::Unknown, _) if down => include_bytes!("light/cell/celldown.svg"),
                (CellState::Unknown, _) => include_bytes!("light/cell/cellup.svg"),

                (CellState::Flagged, CellType::Safe(_)) => include_bytes!("light/cell/falsemine.svg"),
                (CellState::Flagged, _) => include_bytes!("light/cell/cellflag.svg"),
            }
            Gay => match (cell.cell_state, cell.cell_type) {
                (CellState::Revealed, CellType::Safe(0)) => include_bytes!("gay/cell/celldown.svg"),
                (CellState::Revealed, CellType::Safe(1)) => include_bytes!("gay/cell/cell1.svg"),
                (CellState::Revealed, CellType::Safe(2)) => include_bytes!("gay/cell/cell2.svg"),
                (CellState::Revealed, CellType::Safe(3)) => include_bytes!("gay/cell/cell3.svg"),
                (CellState::Revealed, CellType::Safe(4)) => include_bytes!("gay/cell/cell4.svg"),
                (CellState::Revealed, CellType::Safe(5)) => include_bytes!("gay/cell/cell5.svg"),
                (CellState::Revealed, CellType::Safe(6)) => include_bytes!("gay/cell/cell6.svg"),
                (CellState::Revealed, CellType::Safe(7)) => include_bytes!("gay/cell/cell7.svg"),
                (CellState::Revealed, CellType::Safe(8)) => include_bytes!("gay/cell/cell8.svg"),
                (CellState::Revealed, CellType::Safe(_)) => unreachable!(),
                (CellState::Revealed, CellType::Mine) => include_bytes!("gay/cell/blast.svg"),
                (CellState::Revealed, CellType::Unknown) if down => include_bytes!("gay/cell/celldown.svg"),
                (CellState::Revealed, CellType::Unknown) => include_bytes!("gay/cell/cellup.svg"),

                (CellState::Unknown, CellType::Mine) => include_bytes!("gay/cell/cellmine.svg"),
                (CellState::Unknown, _) if down => include_bytes!("gay/cell/celldown.svg"),
                (CellState::Unknown, _) => include_bytes!("gay/cell/cellup.svg"),

                (CellState::Flagged, CellType::Safe(_)) => include_bytes!("gay/cell/falsemine.svg"),
                (CellState::Flagged, _) => include_bytes!("gay/cell/cellflag.svg"),
            }
        }
    }

    pub fn get_restart_button(self, game_status: GameStatus, down: bool, revealing: bool) -> &'static [u8] {
        use Texture::*;
        match self {
            Dark => if down {
                include_bytes!("dark/faces/smilefacedown.svg")
            } else if revealing {
                include_bytes!("dark/faces/clickface.svg")
            } else {
                match game_status {
                    GameStatus::Playing | GameStatus::Never => include_bytes!("dark/faces/smileface.svg"),
                    GameStatus::Won => include_bytes!("dark/faces/winface.svg").as_slice(),
                    GameStatus::Lost => include_bytes!("dark/faces/lostface.svg"),
                }
            }
            Light => if down {
                include_bytes!("light/faces/smilefacedown.svg")
            } else if revealing {
                include_bytes!("light/faces/clickface.svg")
            } else {
                match game_status {
                    GameStatus::Playing | GameStatus::Never => include_bytes!("light/faces/smileface.svg"),
                    GameStatus::Won => include_bytes!("light/faces/winface.svg"),
                    GameStatus::Lost => include_bytes!("light/faces/lostface.svg"),
                }
            }
            Gay => if down {
                include_bytes!("gay/faces/smilefacedown.svg")
            } else if revealing {
                include_bytes!("gay/faces/clickface.svg")
            } else {
                match game_status {
                    GameStatus::Playing | GameStatus::Never => include_bytes!("gay/faces/smileface.svg"),
                    GameStatus::Won => include_bytes!("gay/faces/winface.svg"),
                    GameStatus::Lost => include_bytes!("gay/faces/lostface.svg"),
                }
            }
        }
    }

    pub fn get_digit(self, digit: char) -> &'static [u8] {
        match self {
            Texture::Dark => match digit {
                '0' => include_bytes!("dark/counter/counter0.svg"),
                '1' => include_bytes!("dark/counter/counter1.svg"),
                '2' => include_bytes!("dark/counter/counter2.svg"),
                '3' => include_bytes!("dark/counter/counter3.svg"),
                '4' => include_bytes!("dark/counter/counter4.svg"),
                '5' => include_bytes!("dark/counter/counter5.svg"),
                '6' => include_bytes!("dark/counter/counter6.svg"),
                '7' => include_bytes!("dark/counter/counter7.svg"),
                '8' => include_bytes!("dark/counter/counter8.svg"),
                '9' => include_bytes!("dark/counter/counter9.svg"),
                '-' => include_bytes!("dark/counter/counter-.svg"),
                _ => unimplemented!()
            }
            Texture::Light => match digit {
                '0' => include_bytes!("light/counter/counter0.svg"),
                '1' => include_bytes!("light/counter/counter1.svg"),
                '2' => include_bytes!("light/counter/counter2.svg"),
                '3' => include_bytes!("light/counter/counter3.svg"),
                '4' => include_bytes!("light/counter/counter4.svg"),
                '5' => include_bytes!("light/counter/counter5.svg"),
                '6' => include_bytes!("light/counter/counter6.svg"),
                '7' => include_bytes!("light/counter/counter7.svg"),
                '8' => include_bytes!("light/counter/counter8.svg"),
                '9' => include_bytes!("light/counter/counter9.svg"),
                '-' => include_bytes!("light/counter/counter-.svg"),
                _ => unimplemented!()
            }
            Texture::Gay => match digit {
                '0' => include_bytes!("gay/counter/counter0.svg"),
                '1' => include_bytes!("gay/counter/counter1.svg"),
                '2' => include_bytes!("gay/counter/counter2.svg"),
                '3' => include_bytes!("gay/counter/counter3.svg"),
                '4' => include_bytes!("gay/counter/counter4.svg"),
                '5' => include_bytes!("gay/counter/counter5.svg"),
                '6' => include_bytes!("gay/counter/counter6.svg"),
                '7' => include_bytes!("gay/counter/counter7.svg"),
                '8' => include_bytes!("gay/counter/counter8.svg"),
                '9' => include_bytes!("gay/counter/counter9.svg"),
                '-' => include_bytes!("gay/counter/counter-.svg"),
                _ => unimplemented!()
            }
        }
    }

    pub fn get_background_colour(self) -> Color {
        match self {
            Texture::Dark => include_str!("dark/background").parse()
                    .expect("dark/background should contain valid colour data"),
            Texture::Light => include_str!("light/background").parse()
                    .expect("light/background should contain valid colour data"),
            Texture::Gay => include_str!("dark/background").parse()
                    .expect("gay/background should contain valid colour data"),
        }
    }

    pub fn get_border(self, border: Border) -> &'static [u8] {
        match self {
            Texture::Dark => match border {
                Border::TopLeft => include_bytes!("dark/border/topleft.svg"),
                Border::TopBottom => include_bytes!("dark/border/topbottom.svg"),
                Border::TopRight => include_bytes!("dark/border/topright.svg"),
                Border::BottomLeft => include_bytes!("dark/border/bottomleft.svg"),
                Border::BottomRight => include_bytes!("dark/border/bottomright.svg"),
                Border::LeftRight => include_bytes!("dark/border/leftright.svg"),
                Border::MiddleLeft => include_bytes!("dark/border/middleleft.svg"),
                Border::MiddleRight => include_bytes!("dark/border/middleright.svg"),
                Border::CounterLeft => include_bytes!("dark/border/counterleft.svg"),
                Border::CounterRight => include_bytes!("dark/border/counterright.svg"),
                Border::CounterTop => include_bytes!("dark/border/countertop.svg"),
                Border::CounterBottom => include_bytes!("dark/border/counterbottom.svg"),
            }
            Texture::Light => match border {
                Border::TopLeft => include_bytes!("light/border/topleft.svg"),
                Border::TopBottom => include_bytes!("light/border/topbottom.svg"),
                Border::TopRight => include_bytes!("light/border/topright.svg"),
                Border::BottomLeft => include_bytes!("light/border/bottomleft.svg"),
                Border::BottomRight => include_bytes!("light/border/bottomright.svg"),
                Border::LeftRight => include_bytes!("light/border/leftright.svg"),
                Border::MiddleLeft => include_bytes!("light/border/middleleft.svg"),
                Border::MiddleRight => include_bytes!("light/border/middleright.svg"),
                Border::CounterLeft => include_bytes!("light/border/counterleft.svg"),
                Border::CounterRight => include_bytes!("light/border/counterright.svg"),
                Border::CounterTop => include_bytes!("light/border/countertop.svg"),
                Border::CounterBottom => include_bytes!("light/border/counterbottom.svg"),
            }
            Texture::Gay => match border {
                Border::TopLeft => include_bytes!("gay/border/topleft.svg"),
                Border::TopBottom => include_bytes!("gay/border/topbottom.svg"),
                Border::TopRight => include_bytes!("gay/border/topright.svg"),
                Border::BottomLeft => include_bytes!("gay/border/bottomleft.svg"),
                Border::BottomRight => include_bytes!("gay/border/bottomright.svg"),
                Border::LeftRight => include_bytes!("gay/border/leftright.svg"),
                Border::MiddleLeft => include_bytes!("gay/border/middleleft.svg"),
                Border::MiddleRight => include_bytes!("gay/border/middleright.svg"),
                Border::CounterLeft => include_bytes!("gay/border/counterleft.svg"),
                Border::CounterRight => include_bytes!("gay/border/counterright.svg"),
                Border::CounterTop => include_bytes!("gay/border/countertop.svg"),
                Border::CounterBottom => include_bytes!("gay/border/counterbottom.svg"),
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Border {
    TopLeft,
    TopBottom,
    TopRight,
    BottomLeft,
    BottomRight,
    LeftRight,
    MiddleLeft,
    MiddleRight,
    CounterLeft,
    CounterRight,
    CounterTop,
    CounterBottom,
}