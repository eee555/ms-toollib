<template>
    <div>
        <button @click="get_board">无猜埋雷</button>
        <p>{{ board }}</p>
        <button @click="cal_3BV">计算3BV</button>
        <p>经过计算，上图的3BV等于{{ bbbv }}</p>
    </div>
    <input type="file" @change="openFile">
    <div v-for="v of video_info">{{ v }}</div>
</template>

<script setup lang="ts">

import { defineComponent, ref, reactive, toRef, onMounted } from "vue";

let board = ref<number[][]>([[0]]);
let bbbv = ref(0);
let video_info = ref<string[]>([]);

const get_board = async () => {
    while (board.value.length > 0) {
        board.value.pop();
    }
    const ms = await import("ms-toollib");
    const rows = 16;
    const columns = 30;
    let b = ms.laymine_solvable(rows, columns, 99, 0, 0, 10000)[0];

    for (let i = 0; i < rows; i++) {
        board.value.push([])
        for (let j = 0; j < columns; j++) {
            board.value[i].push(b[i][j])
        }
    }
};
const cal_3BV = async () => {
    const ms = await import("ms-toollib");
    bbbv.value = ms.cal_bbbv(board.value)
};

const openFile = async (e: Event) => {
    const ms = await import("ms-toollib");
    const file = (e.target as HTMLInputElement).files?.[0];
    if (file) {
        const reader = new FileReader();
        reader.onload = () => {
            const uint8Array = new Uint8Array(reader.result as ArrayBuffer);
            console.log(uint8Array);
            
            let video = new ms.AvfVideo(uint8Array, "")
            video.parse()
            video.analyse()
            video.current_time = 999999.0
            video_info.value.push(`3BV = ${video.bbbv}`);
            video_info.value.push(`3BV/s = ${video.bbbv_s}`);
            video_info.value.push(`left/s = ${video.left_s}`);
        };
        reader.readAsArrayBuffer(file);
    }

    

}


</script>
