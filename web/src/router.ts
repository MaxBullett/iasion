import { createRouter, createWebHistory } from 'vue-router'
import NotFoundView from './NotFoundView.vue'
import SessionView from './session/SessionView.vue'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: SessionView },
    { path: '/:pathMatch(.*)*', component: NotFoundView },
  ],
})
