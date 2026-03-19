import { defineConfig, loadEnv } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, '.', '')
  return {
    plugins: [svelte(), tailwindcss()],
    server: {
      port: Number(env.VITE_PORT) || 3333,
      proxy: {
        '/api': 'http://localhost:9999',
      },
    },
  }
})
