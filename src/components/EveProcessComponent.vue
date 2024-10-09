<script setup lang="ts">
import { EveProcessModel } from '../models/EveProcessModel';
import {ref, watch} from "vue";
import {listen} from "@tauri-apps/api/event";
import router from "../router";


const eve_processes = ref<Array<EveProcessModel>>([]);

//eve_processes.value = [new EveProcessModel(1, 'Eve Process 1'), new EveProcessModel(2, 'Eve Process 2')];
listen<number[]>('processes', (event) => {
  //console.log('Received processes', event.payload);
  eve_processes.value = event.payload.map(pid => new EveProcessModel(pid, ''));
});


// Watch para detectar si el array está vacío
watch(eve_processes, (newVal) => {
  if (newVal.length === 0) {
    router.push('/eveProcess/'); // Aquí puedes personalizar la ruta
  }
});
</script>

<template>
  <div class="container-eve-process">
    
    <Tabs>
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
