<script setup lang="ts">
import { EveProcessModel } from '../models/EveProcessModel';
import {ref, watch} from "vue";
import {listen} from "@tauri-apps/api/event";
import router from "../router";


const eve_processes = ref<Array<EveProcessModel>>([]);

listen<number[]>('processes', (event) => {
  eve_processes.value = event.payload.map(pid => new EveProcessModel(pid, ''));
});


watch(eve_processes, (newVal) => {
  if (newVal.length === 0) {
    router.push('/eveProcess/'); 
  }
});

const activeTab = ref(0); // Índice de la pestaña activa

function onTabChange(newValue: number) {
  activeTab.value = newValue;
}
</script>

<template>
  <div class="container-eve-process">
    <Tabs :value="activeTab" @update:value="onTabChange">
      <TabList>
        <Tab v-for="eve_process in eve_processes" :key=eve_process.pid  :value="eve_process.pid">
          <router-link class="container-eve-process" :to="`/eveProcess/${eve_process.pid}`">
              <Avatar class="margin-right" image='/eve_icon.png' shape="circle" />
              <span class="font-bold whitespace-nowrap">PID: {{ eve_process.pid }}</span>
          </router-link>
        </Tab>
      </TabList>
    </Tabs>
  </div>
</template>



<style scoped>

.container-eve-process {
  display: flex;
  flex-direction: row;
  align-items: center;
}

div >>> .p-tab {
  padding: 0.08rem 1.25rem; /* Ajusta el padding para cada tab */
  display: flex;
  flex-direction: row;
  align-items: center;
}

.margin-right {
  margin-right: 0.5rem;
}

</style>
