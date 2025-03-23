<template>
    <div class="xterm-box" ref="boxRef">

    </div>
</template>

<script setup lang="ts">

import { Terminal } from '@xterm/xterm';
import { onMounted, ref } from 'vue';
import { FitAddon } from '@xterm/addon-fit';

const term = new Terminal({
    disableStdin: false,  // 允许输入（默认值，可省略）
    cursorBlink: true,    // 光标闪烁
    cursorStyle: 'block', // 光标样式
    fontSize: 14,
    fontFamily: 'Consolas, monospace'
});
const fitAddon = new FitAddon();
term.loadAddon(fitAddon)
const boxRef = ref<HTMLElement | null>(null);

onMounted(() => {
    term.open(boxRef.value!);
    fitAddon.fit()
    term.writeln('Hello from xterm');
    term.input('hhh', true)
    term.focus();
    term.onData((data) =>{
        console.log("用户输入内容：", data); 
        term.writeln(data)
    })
});

</script>

<style scoped>
.xterm-box {
    height: calc(100vh - 36px);
}
</style>