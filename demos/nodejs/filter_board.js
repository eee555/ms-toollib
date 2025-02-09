const ms = require('ms-toollib');
const { assert, log } = require('console');

let num_8 = 0
for (let i = 0; i < 100000; i++) {
    let board = ms.laymine(16, 30, 99, 0, 0);
    let ms_board = new ms.Board(board);
    if (ms_board.cell8 >= 1) {
        console.log(board);
        num_1_8++;
    }
}
console.log(num_8);


