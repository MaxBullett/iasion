<script setup lang="ts">
// Any path the router does not know. The server serves the app for every path
// outside /api (ADR-0043, decision 12), so this page is the 404 of the app.
import { RouterLink } from 'vue-router'
</script>

<template>
  <main>
    <h1>Page not found</h1>
    <p>There is no page at this address.</p>
    <p><RouterLink to="/">Back to the start</RouterLink></p>
  </main>
</template>

<style scoped>
main {
  max-width: 40rem;
  margin: 0 auto;
  padding: 1rem;
  font-family: system-ui, sans-serif;
}
</style>
