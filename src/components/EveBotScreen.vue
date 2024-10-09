<script setup lang="ts">
import {defineComponent, ref, watch} from 'vue';
import { useRoute } from 'vue-router';
import {listen} from "@tauri-apps/api/event";
import {EveProcessModel} from "../models/EveProcessModel.ts";
import {invoke} from "@tauri-apps/api/core";

const route = useRoute(); // Accedemos a la instancia de la ruta
const id = ref(route.params.id); // Extraemos el parámetro "id" de la ruta

watch(() => route.params.id, (newId) => {
  id.value = newId;
});

listen<any>('eve_ui_status', (event) => {
  console.log('Received status', event.payload);

});

const startTracker = () => {
  const pid = id.value as number;
  invoke('start_tracker', { pid } )
      .then((response) => {
        console.log("Respuesta desde Rust:", response);
      })
      .catch((error) => {
        console.error("Error invocando el comando en Rust:", error);
      });
};

const stopTracker = () => {
  const pid = id.value as number;
  invoke('stop_tracker', { pid } )
      .then((response) => {
        console.log("Respuesta desde Rust:", response);
      })
      .catch((error) => {
        console.error("Error invocando el comando en Rust:", error);
      });
};
</script>

<template>
  <div>
    <h1>Process ID: {{ id || 'No Id' }}</h1>
    <button v-if="id" @click="startTracker">Start</button>
    <button v-if="id" @click="stopTracker">Stop</button>
  </div>
</template>

<style scoped>

</style>