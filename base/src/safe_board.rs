use std::cmp::{max, min};

/// 安全局面的行
#[derive(Clone, Debug)]
struct SafeBoardRow {
    value: Vec<i32>,
    /// 迭代器的计数器
    counter: usize,
}

/// 安全局面
#[derive(Clone, Debug)]
struct SafeBoard {
    value: Vec<SafeBoardRow>,
    /// 迭代器的计数器
    counter: usize,
}

impl SafeBoardRow {
    fn new(v: Vec<i32>) -> SafeBoardRow {
        SafeBoardRow {
            value: v,
            counter: 0,
        }
    }
}

impl SafeBoard {
    fn new(v: Vec<Vec<i32>>) -> SafeBoard {
        let mut safe_board = vec![];
        for row in v {
            safe_board.push(SafeBoardRow::new(row));
        }
        SafeBoard {
            value: safe_board,
            counter: 0,
        }
    }
}

impl std::ops::Index<usize> for SafeBoardRow {
    type Output = i32;
    fn index(&self, index: usize) -> &Self::Output {
        &self.value[index]
    }
}

impl std::ops::IndexMut<usize> for SafeBoardRow {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.value[index]
    }
}

impl<'a> IntoIterator for &'a SafeBoardRow {
    type Item = i32;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.value.clone().into_iter()
    }
}

impl Iterator for SafeBoardRow {
    type Item = i32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < self.value.len() {
            let t = self.value[self.counter];
            self.counter += 1;
            Some(t)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for SafeBoardRow {
    fn len(&self) -> usize {
        self.value.len()
    }
}

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

impl std::ops::Index<usize> for SafeBoard {
    type Output = SafeBoardRow;
    fn index(&self, index: usize) -> &Self::Output {
        &self.value[index]
    }
}

impl std::ops::IndexMut<usize> for SafeBoard {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.value[index]
    }
}


impl<'a> IntoIterator for &'a SafeBoard {
    type Item = SafeBoardRow;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.value.clone().into_iter()
    }
}

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

impl BoardSize for SafeBoard {
    fn get_row(&self) -> usize {
        self.value.len()
    }
    fn get_column(&self) -> usize {
        self.value[0].len()
    }
}

impl BoardSize for &SafeBoard {
    fn get_row(&self) -> usize {
        self.value.len()
    }
    fn get_column(&self) -> usize {
        self.value[0].len()
    }
}





