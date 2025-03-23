<script setup lang="ts">
import { ref } from 'vue';
import BaseNav from './components/BaseNav.vue';
import HostView from './HostView.vue';
import KeysView from './KeysView.vue';
import HistoryView from './HistoryView.vue';


const active = ref('hosts');

</script>

<template>
  <main>
    <Splitter style="height: calc(100vh - 36px)">
      <SplitterPanel :size="25" :minSize="10">
        <div class="top">
          <BaseNav v-model="active" />
        </div>
        <div class="bottom">

        </div>
      </SplitterPanel>
      <SplitterPanel  :size="75">
         <template v-if="active === 'hosts'">
          <HostView />
         </template>
          <template v-else-if="active === 'keys'">
            <KeysView />  
          </template>
          <template v-else-if="active === 'history'">
            <HistoryView />
          </template>
      </SplitterPanel>
    </Splitter>
  </main>
</template>

<style scoped>
.top {
  height: 40vh;
  width: 100%;
  border-bottom: 1px solid #e7ebf0;
}
</style>
