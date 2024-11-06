import {createApp} from "vue";
import App from "./App.vue";
import Aura from '@primevue/themes/aura';
import PrimeVue from "primevue/config";
import router from "./router";
import 'reflect-metadata';
const app = createApp(App);

app.use(PrimeVue, {
    theme: {
        preset: Aura
    }
});
app.use(router);

app.mount("#app");
