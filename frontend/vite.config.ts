import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      // Enable runtime template compilation for inline templates
      vue: 'vue/dist/vue.esm-bundler.js',
    },
  },
});

