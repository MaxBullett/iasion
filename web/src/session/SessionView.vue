<script setup lang="ts">
// The walking skeleton's only screen: who is signed in, and the database behind it.
import { onMounted, ref } from 'vue'
import { createApiClient } from '../api/client'
import { revisionLabel } from './revision'
import { loadSession, type SessionState } from './session'

const state = ref<SessionState>({ kind: 'loading' })

onMounted(async () => {
  state.value = await loadSession(createApiClient())
})
</script>

<template>
  <main>
    <h1>Iasion</h1>
    <p v-if="state.kind === 'loading'" role="status">Loading…</p>
    <template v-else-if="state.kind === 'signed-in'">
      <p>Signed in as {{ state.email }}</p>
      <p v-if="state.firstSetupPending">No household exists yet. First setup comes next.</p>
      <dl>
        <dt>Schema version</dt>
        <dd>{{ state.schemaVersion ?? 'none' }}</dd>
      </dl>
      <footer>
        <p>Version {{ revisionLabel(state.revision) }}</p>
      </footer>
    </template>
    <p v-else-if="state.kind === 'refused'" role="alert">{{ state.title }}</p>
    <p v-else role="alert">The server cannot be reached.</p>
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
