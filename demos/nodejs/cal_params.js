const ms = require('ms-toollib');
const { assert, log } = require('console');

const board = [[0, 0, 1, -1, 2, -1, 1, 0], [0, 0, 2, 2, 3, 1, 1, 0], [0, 1, 2, -1, 1, 0, 0, 0],
[0, 1, -1, 3, 2, 0, 0, 0], [2, 3, 3, -1, 2, 1, 1, 0], [-1, -1, 2, 2, 3, -1, 1, 0],
[2, 2, 1, 1, -1, 3, 2, 0], [0, 0, 0, 1, 2, -1, 1, 0]];
//计算3BV
let bbbv = ms.cal_bbbv(board);
assert(bbbv == 12);

let op = ms.cal_op(board);
assert(op == 3);
// console.log(op);
