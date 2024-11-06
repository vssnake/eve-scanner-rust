<script setup lang="ts">
import { ref, watch} from 'vue';
import { useRoute } from 'vue-router';
import {UiManager} from "../features/eveui/UiManager.ts";

const route = useRoute(); 
const id = ref<string>(route.params.id as string);



watch(() => route.params.id as string, (newId) => {
  id.value = newId;
});

const ui_manager = new UiManager();



const startTracker = () => {
  const pid = Number(id.value);
  ui_manager.startTracker(pid);
};

const stopTracker = () => {
  const pid = Number(id.value);
  ui_manager.stopTracker(pid);
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