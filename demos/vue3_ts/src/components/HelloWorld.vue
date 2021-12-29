<template>
  <div class="hello">
    <button @click="get_board">无猜埋雷</button>
    <p>{{ board }}</p>
    <button @click="cal_3BV">计算3BV</button>
    <p>经过计算，上图的3BV等于{{ bbbv }}</p>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, reactive, toRef } from "vue";

export default defineComponent({
  name: "ms_toollib demo of vue3 with ts",
  setup() {
    let board = reactive([[0]]);
    let bbbv = ref(0);
    const get_board = async () => {
      while(board.length > 0) {
        board.pop();
      }
      const ms = await import("ms-toollib");
      const rows = 16;
      const columns = 30;
      let b = JSON.parse(ms.laymine_solvable(rows, columns, 99, 0, 0, 100, 381, 10000, 40))[0];
      for(let i = 0; i < rows; i++) {
        board.push([])
        for(let j = 0; j < columns; j++) {
          board[i].push(b[i][j])
        }
      }
    };
    const cal_3BV = async () => {
      const ms = await import("ms-toollib");
      bbbv.value = ms.cal3BV(JSON.stringify(board))
    };

    return {
      board,
      get_board,
      bbbv,
      cal_3BV
    };
  },
});
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
h3 {
  margin: 40px 0 0;
}
ul {
  list-style-type: none;
  padding: 0;
}
li {
  display: inline-block;
  margin: 0 10px;
}
a {
  color: #42b983;
}
</style>
