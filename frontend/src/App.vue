<template>
  <div>
    <header class="border-b bg-white">
      <div class="mx-auto max-w-5xl px-4 py-3 flex items-center justify-between">
        <h1 class="text-lg font-semibold text-gray-900">
          PriceCrowd
        </h1>
        <nav class="flex gap-4 text-sm">
          <RouterLink to="/" class="text-gray-700 hover:text-black">Главная</RouterLink>
          <RouterLink to="/products" class="text-gray-700 hover:text-black">Товары</RouterLink>
          <RouterLink to="/categories" class="text-gray-700 hover:text-black">Категории</RouterLink>
          <RouterLink to="/stores" class="text-gray-700 hover:text-black">Магазины</RouterLink>
          <RouterLink to="/about" class="text-gray-700 hover:text-black">О проекте</RouterLink>
        </nav>
      </div>
    </header>

    <main class="mx-auto max-w-5xl px-4 py-8">
      <RouterView />
    </main>

    <footer class="mx-auto max-w-5xl px-4 py-8 text-xs text-gray-500">
      Vue 3 + Vite + Tailwind
    </footer>
  </div>
  
</template>

<script setup lang="ts">
// No-op for now
</script>
