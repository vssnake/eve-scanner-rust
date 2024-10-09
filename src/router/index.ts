﻿import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router';
import EveProcessComponent from '../components/EveProcessComponent.vue';
import EveBotScreen from "../components/EveBotScreen.vue";

const routes: Array<RouteRecordRaw> = [
    {
        path: '/eveProcess/:id?',
        name: 'EveProcess',
        component: EveBotScreen,
    },
];

// Crea una nueva instancia del router
const router = createRouter({
    history: createWebHistory(),
    routes,
});

router.beforeEach((to, from, next) => {
    console.log(`Navegando desde ${from.fullPath} hacia ${to.fullPath}`);
    next();
});

router.beforeResolve((to, from, next) => {
    console.log(`Resolviendo la ruta: ${to.fullPath}`);
    next();
});

router.afterEach((to, from) => {
    console.log(`Navegación completa hacia ${to.fullPath}`);
});

export default router;