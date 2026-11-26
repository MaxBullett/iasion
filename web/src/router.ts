import { createRouter, createWebHistory } from 'vue-router'
import SessionView from './session/SessionView.vue'

export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: '/', component: SessionView }],
})
