use crate::mancala::MancalaGameState;

#[derive(Debug)]
pub enum MancalaMove {
    A,
    B,
    C,
    D,
    E,
    F,
}

impl MancalaMove {
    pub fn from_string(str: &str) -> Option<Self> {
        match str.to_ascii_uppercase().as_str() {
            "A" => Some(Self::A),
            "B" => Some(Self::B),
            "C" => Some(Self::C),
            "D" => Some(Self::D),
            "E" => Some(Self::E),
            "F" => Some(Self::F),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            MancalaMove::A => "A",
            MancalaMove::B => "B",
            MancalaMove::C => "C",
            MancalaMove::D => "D",
            MancalaMove::E => "E",
            MancalaMove::F => "F",
        }
    }

    pub fn from_index(index: &usize) -> Option<Self> {
        match index % (MancalaGameState::PITS_PER_SIDE + 1) {
            0 => Some(Self::A),
            1 => Some(Self::B),
            2 => Some(Self::C),
            3 => Some(Self::D),
            4 => Some(Self::E),
            5 => Some(Self::F),
            _ => None,
        }
    }

    pub fn to_index(&self) -> usize {
        match self {
            MancalaMove::A => 0,
            MancalaMove::B => 1,
            MancalaMove::C => 2,
            MancalaMove::D => 3,
            MancalaMove::E => 4,
            MancalaMove::F => 5,
        }
    }
}
