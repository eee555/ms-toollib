use std::cell::Cell;
use std::cmp;
use std::fmt;

pub struct Tile {
    pub x: usize,
    pub y: usize,
    pub index: usize,
    pub is_covered: Cell<bool>,
    pub value: Cell<u8>,
    pub is_flagged: Cell<bool>,
    pub found_bomb: Cell<bool>,
    pub is_bomb: Cell<Option<bool>>,
    pub exploded: Cell<bool>,
    pub is_safe: Cell<bool>,
    pub is_next_move: Cell<bool>,
    pub is_start: Cell<bool>,
    pub on_edge: Cell<bool>,
    pub hint: Cell<bool>,
    pub probability: Cell<f64>,
    pub hint_text: Cell<String>,
    pub has_hint: Cell<bool>,
}

impl Clone for Tile {
    fn clone(&self) -> Self {
        Tile {
            x: self.x,
            y: self.y,
            index: self.index,
            is_covered: Cell::new(self.is_covered.get()),
            value: Cell::new(self.value.get()),
            is_flagged: Cell::new(self.is_flagged.get()),
            found_bomb: Cell::new(self.found_bomb.get()),
            is_bomb: Cell::new(self.is_bomb.get()),
            exploded: Cell::new(self.exploded.get()),
            is_safe: Cell::new(self.is_safe.get()),
            is_next_move: Cell::new(self.is_next_move.get()),
            is_start: Cell::new(self.is_start.get()),
            on_edge: Cell::new(self.on_edge.get()),
            hint: Cell::new(self.hint.get()),
            probability: Cell::new(self.probability.get()),
            hint_text: Cell::new(self.hint_text.take()),
            has_hint: Cell::new(self.has_hint.get()),
        }
    }
}

impl Tile {
    pub fn new(x: usize, y: usize, index: usize) -> Self {
        Tile {
            x,
            y,
            index,
            is_covered: Cell::new(true),
            value: Cell::new(0),
            is_flagged: Cell::new(false),
            found_bomb: Cell::new(false),
            is_bomb: Cell::new(None),
            exploded: Cell::new(false),
            is_safe: Cell::new(false),
            is_next_move: Cell::new(false),
            is_start: Cell::new(false),
            on_edge: Cell::new(false),
            hint: Cell::new(false),
            probability: Cell::new(-1.0),
            hint_text: Cell::new(String::new()),
            has_hint: Cell::new(false),
        }
    }

    pub fn is_adjacent(&self, other: &Tile) -> bool {
        let dx = if self.x > other.x { self.x - other.x } else { other.x - self.x };
        let dy = if self.y > other.y { self.y - other.y } else { other.y - self.y };
        dx < 2 && dy < 2 && !(dx == 0 && dy == 0)
    }

    pub fn is_equal(&self, other: &Tile) -> bool {
        self.x == other.x && self.y == other.y
    }

    pub fn as_text(&self) -> String {
        format!("({},{})", self.x, self.y)
    }

    pub fn set_probability(&self, prob: f64, progress: Option<f64>, safety2: Option<f64>) {
        self.probability.set(prob);
        self.has_hint.set(true);

        let text = if (prob - 1.0).abs() < 1e-12 {
            "Safe".to_string()
        } else if prob.abs() < 1e-12 {
            "Mine".to_string()
        } else if progress.is_none() {
            format!("\n{:.2}% safe", prob * 100.0)
        } else {
            format!(
                "\n{:.2}% safe\n{:.2}% second safe\n{:.2}% progress",
                prob * 100.0,
                safety2.unwrap_or(0.0) * 100.0,
                progress.unwrap() * 100.0
            )
        };
        self.hint_text.set(text);
    }

    pub fn is_covered(&self) -> bool {
        self.is_covered.get()
    }

    pub fn set_covered(&self, covered: bool) {
        self.is_covered.set(covered);
    }

    pub fn set_value(&self, value: u8) {
        self.value.set(value);
        self.is_covered.set(false);
    }

    pub fn set_value_only(&self, value: u8) {
        if self.is_flagged.get() {
            eprintln!("{} Assigning value {} to a flagged tile!", self.as_text(), value);
        }
        self.value.set(value);
    }

    pub fn get_value(&self) -> u8 {
        self.value.get()
    }

    pub fn set_found_bomb(&self) {
        self.found_bomb.set(true);
    }

    pub fn unset_found_bomb(&self) {
        self.found_bomb.set(false);
    }

    pub fn is_solver_found_bomb(&self) -> bool {
        self.found_bomb.get()
    }

    pub fn set_bomb(&self, bomb: bool) {
        self.is_bomb.set(Some(bomb));
    }

    pub fn as_bomb(&self) -> Option<bool> {
        self.is_bomb.get()
    }
}

impl cmp::PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}
