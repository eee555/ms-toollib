const ms = require('ms-toollib');

let board = ms.laymine(8, 8, 10, 0, 0);
console.log(board);

let board_op = ms.laymine_op(8, 8, 10, 0, 0);
console.log(board_op);

let board_solvable = ms.laymine_solvable(8, 8, 10, 0, 0);
console.log(board_solvable);


