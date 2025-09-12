<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "@/stores/app";
import { Button } from "@/components/ui";

const appStore = useAppStore();
const greetMsg = ref("");
const name = ref("");

async function greet() {
  appStore.setLoading(true);
  try {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg.value = await invoke("greet", { name: name.value });
  } finally {
    appStore.setLoading(false);
  }
}
</script>

<template>
  <main class="min-h-screen bg-background text-foreground p-8">
    <div class="max-w-4xl mx-auto">
      <h1 class="text-4xl font-bold text-center mb-8">{{ appStore.title }}</h1>
      
      <div class="flex justify-center items-center gap-8 mb-8">
        <a href="https://vite.dev" target="_blank" class="hover:opacity-80 transition-opacity">
          <img src="/vite.svg" class="w-24 h-24" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank" class="hover:opacity-80 transition-opacity">
          <img src="/tauri.svg" class="w-24 h-24" alt="Tauri logo" />
        </a>
        <a href="https://vuejs.org/" target="_blank" class="hover:opacity-80 transition-opacity">
          <img src="./assets/vue.svg" class="w-24 h-24" alt="Vue logo" />
        </a>
      </div>
      
      <p class="text-center text-muted-foreground mb-8">
        Built with Tauri + Vue 3 + TypeScript + Tailwind CSS 4 + Pinia + shadcn-vue
      </p>

      <div class="max-w-md mx-auto">
        <form @submit.prevent="greet" class="flex gap-2 mb-4">
          <input 
            id="greet-input" 
            v-model="name" 
            placeholder="Enter a name..." 
            class="flex-1 px-3 py-2 border border-input bg-background rounded-md focus:outline-none focus:ring-2 focus:ring-ring"
            :disabled="appStore.isLoading"
          />
          <Button 
            type="submit" 
            :disabled="appStore.isLoading"
          >
            {{ appStore.isLoading ? 'Loading...' : 'Greet' }}
          </Button>
        </form>
        
        <p v-if="greetMsg" class="text-center p-4 bg-muted rounded-md">
          {{ greetMsg }}
        </p>
      </div>
    </div>
  </main>
</template>