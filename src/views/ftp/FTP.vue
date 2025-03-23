<template>
    <div>
        <h1>FTP</h1>
        <Button @click="onTest">test</Button>
    </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
const onTest = () => {
    invoke('ssh_connect').then((res) => {
        console.log(res)
    })
}

</script>

<style scoped></style>