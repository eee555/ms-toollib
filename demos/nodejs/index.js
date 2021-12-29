// 安装：npm install ms-toollib@1.2.7-node --save

const ms = require('ms-toollib');

const a = [[0, 0, 1, -1, 2, -1, 1, 0], [0, 0, 2, 2, 3, 1, 1, 0], [0, 1, 2, -1, 1, 0, 0, 0],
[0, 1, -1, 3, 2, 0, 0, 0], [2, 3, 3, -1, 2, 1, 1, 0], [-1, -1, 2, 2, 3, -1, 1, 0],
[2, 2, 1, 1, -1, 3, 2, 0], [0, 0, 0, 1, 2, -1, 1, 0]];
let bv = ms.cal3BV(JSON.stringify(a));  //计算3BV
console.log(bv);


