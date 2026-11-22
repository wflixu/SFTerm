import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueJsx from '@vitejs/plugin-vue-jsx'
import vueDevTools from 'vite-plugin-vue-devtools'

// https://vite.dev/config/
export default defineConfig({
  // Tauri 的 devUrl 固定是 5173；端口被占用时直接失败，
  // 避免 Vite 悄悄换到 5174 而 Tauri 仍加载 5173 上的旧实例。
  server: {
    port: 5173,
    strictPort: true,
  },
  plugins: [
    vue(),
    vueJsx(),
    vueDevTools(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    },
  },
})
