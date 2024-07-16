#[cfg(any(feature = "py", feature = "rs"))]
use rand::Rng;

// pub fn laymine_safely(
//     row: usize,
//     column: usize,
//     mine_num: usize,
//     x0: usize,
//     y0: usize,
// ) -> SafeBoard {
//     let board = laymine(row, column, mine_num, x0, y0);
//     SafeBoard::new(board)
// }

#[cfg(any(feature = "py", feature = "rs"))]
fn encode(v: i32, rng: &mut rand::rngs::ThreadRng) -> (i32, i32, i32) {
    let a = rng.gen_range(-2_0000_0000i32..2_0000_0001);
    let b = rng.gen_range(-1_0000i32..1_0001);
    let code = [
        7, 1, 3, 14, 16, 17, 9, 11, 12, 5, 6, 13, 19, 19, 18, 15, 4, 8, 2, 0,
    ][(v + 3) as usize];
    let c = code + rng.gen_range(-1000_0000i32..1000_0001) * 20 - a - b;
    (a, b, c)
}

#[cfg(any(feature = "py", feature = "rs"))]
fn decode(a: i32, b: i32, c: i32) -> i32 {
    let mut t = (a + b + c) % 20;
    if t < 0 {
        t += 20
    }
    t
}

/// 安全局面的行
#[cfg(any(feature = "py", feature = "rs"))]
#[derive(Clone, Debug)]
pub struct SafeBoardRow {
    value_1: Vec<i32>,
    value_2: Vec<i32>,
    value_3: Vec<i32>,
    table: [i32; 20],
    /// 迭代器的计数器
    counter: usize,
}

/// 安全局面
#[cfg(any(feature = "py", feature = "rs"))]
#[derive(Clone, Debug)]
pub struct SafeBoard {
    value: Vec<SafeBoardRow>,
    /// 迭代器的计数器
    counter: usize,
}

#[cfg(any(feature = "py", feature = "rs"))]
impl SafeBoardRow {
    pub fn new(v: Vec<i32>) -> SafeBoardRow {
        let mut rng = rand::thread_rng();
        let mut value_1 = vec![];
        let mut value_2 = vec![];
        let mut value_3 = vec![];
        for i in v {
            let (a, b, c) = encode(i, &mut rng);
            value_1.push(a);
            value_2.push(b);
            value_3.push(c);
        }
        SafeBoardRow {
            value_1,
            value_2,
            value_3,
            table: [
                16, -2, 15, -1, 13, 6, 7, -3, 14, 3, 9, 4, 5, 8, 0, 12, 1, 2, 11, 10,
            ],
            counter: 0,
        }
    }
    pub fn into_vec(&self) -> Vec<i32> {
        let mut row_vec = vec![];
        for i in 0..self.value_1.len() {
            let v = decode(self.value_1[i], self.value_2[i], self.value_3[i]) as usize;
            let t = self.table[v];
            row_vec.push(t);
        }
        row_vec
    }
}

#[cfg(any(feature = "py", feature = "rs"))]
impl SafeBoard {
    pub fn new(v: Vec<Vec<i32>>) -> SafeBoard {
        let mut safe_board = vec![];
        for row in v {
            safe_board.push(SafeBoardRow::new(row));
        }
        SafeBoard {
            value: safe_board,
            counter: 0,
        }
    }
    pub fn into_vec_vec(&self) -> Vec<Vec<i32>> {
        let mut board_vec = vec![];
        for row in &self.value {
            let mut row_vec = vec![];
            for i in 0..self.get_column() {
                let v = decode(row.value_1[i], row.value_2[i], row.value_3[i]) as usize;
                let t = row.table[v];
                row_vec.push(t);
            }
            board_vec.push(row_vec);
        }
        board_vec
    }
    pub fn set(&mut self, board: Vec<Vec<i32>>) {
        let mut safe_board = vec![];
        for row in board {
            safe_board.push(SafeBoardRow::new(row));
        }
        self.value = safe_board;
    }
}

#[cfg(any(feature = "py", feature = "rs"))]
impl std::ops::Index<usize> for SafeBoardRow {
    type Output = i32;
    fn index(&self, index: usize) -> &Self::Output {
        let t = decode(
            self.value_1[index],
            self.value_2[index],
            self.value_3[index],
        ) as usize;
        &self.table[t]
    }
}

// impl std::ops::IndexMut<usize> for SafeBoardRow {
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         let t = decode(
//             self.value_1[index],
//             self.value_2[index],
//             self.value_3[index],
//         ) as usize;
//         let t = self.table[t];
//         let (a, b, c) = encode(t, &mut self.rng);
//             self.value_1[index] = a;
//             self.value_2[index] = b;
//             self.value_3[index] = c;
//         &mut self.value[index]
//     }
// }

// impl<'a> IntoIterator for &'a SafeBoardRow {
//     type Item = i32;
//     type IntoIter = std::vec::IntoIter<Self::Item>;
//     fn into_iter(self) -> Self::IntoIter {
//         self.value_1.clone().into_iter().map()
//     }
// }

#[cfg(any(feature = "py", feature = "rs"))]
impl Iterator for SafeBoardRow {
    type Item = i32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < self.value_1.len() {
            let t = decode(
                self.value_1[self.counter],
                self.value_2[self.counter],
                self.value_3[self.counter],
            );
            self.counter += 1;
            Some(t)
        } else {
            None
        }
    }
}

#[cfg(any(feature = "py", feature = "rs"))]
impl ExactSizeIterator for SafeBoardRow {
    fn len(&self) -> usize {
        self.value_1.len()
    }
}

#[cfg(any(feature = "py", feature = "rs"))]
impl Iterator for SafeBoard {
    type Item = SafeBoardRow;
    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < self.value.len() {
            let t = self.value[self.counter].clone();
            self.counter += 1;
            Some(t)
        } else {
            None
        }
    }
}

#[cfg(any(feature = "py", feature = "rs"))]
impl std::ops::Index<usize> for SafeBoard {
    type Output = SafeBoardRow;
    fn index(&self, index: usize) -> &Self::Output {
        &self.value[index]
    }
}

// impl std::ops::IndexMut<usize> for SafeBoard {
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         &mut self.value[index]
//     }
// }

// impl<'a> IntoIterator for &'a SafeBoard {
//     type Item = SafeBoardRow;
//     type IntoIter = std::vec::IntoIter<Self::Item>;
//     fn into_iter(self) -> Self::IntoIter {
//         self.value.clone().into_iter()
//     }
// }

pub trait BoardSize {
    fn get_row(&self) -> usize;
    fn get_column(&self) -> usize;
}

impl BoardSize for Vec<Vec<i32>> {
    fn get_row(&self) -> usize {
        self.len()
    }
    fn get_column(&self) -> usize {
        self[0].len()
    }
}

impl BoardSize for &Vec<Vec<i32>> {
    fn get_row(&self) -> usize {
        self.len()
    }
    fn get_column(&self) -> usize {
        self[0].len()
    }
}

#[cfg(any(feature = "py", feature = "rs"))]
impl BoardSize for SafeBoard {
    fn get_row(&self) -> usize {
        self.value.len()
    }
    fn get_column(&self) -> usize {
        self.value[0].len()
    }
}

#[cfg(any(feature = "py", feature = "rs"))]
impl BoardSize for &SafeBoard {
    fn get_row(&self) -> usize {
        self.value.len()
    }
    fn get_column(&self) -> usize {
        self.value[0].len()
    }
}




