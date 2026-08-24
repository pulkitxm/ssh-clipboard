<script setup>
import StarButton from '../StarButton.vue'
</script>

<template>
  <header class="sticky top-0 z-10 border-b border-line bg-bg">
    <div class="mx-auto flex w-[min(90rem,calc(100vw-7rem))] items-center justify-between gap-4 py-4 max-md:w-[calc(100vw-2.5rem)] max-md:py-3">
      <a class="whitespace-nowrap font-semibold text-bright" href="#top">
        ssh-clipboard<span class="text-mint">_</span>
      </a>
      <nav class="flex items-center gap-9 text-[0.9rem] max-md:gap-5 max-md:text-[0.8rem]">
        <a class="text-ink transition-colors hover:text-mint max-[560px]:hidden" href="#install">install</a>
        <a class="text-ink transition-colors hover:text-mint max-[560px]:hidden" href="#commands">commands</a>
        <a class="text-ink transition-colors hover:text-mint max-[560px]:hidden" href="#how-it-works">how&nbsp;it&nbsp;works</a>
        <StarButton />
      </nav>
    </div>
  </header>
</template>
