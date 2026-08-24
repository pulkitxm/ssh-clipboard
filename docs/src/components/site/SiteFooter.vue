<script setup>
const link = 'text-mint border-b border-mint/35 transition-colors hover:border-mint'
</script>

<template>
  <footer class="mt-24 border-t border-line/40 pb-24 pt-8 text-[0.85rem] text-dim">
    <div
      class="mx-auto flex w-[min(90rem,calc(100vw-7rem))] flex-wrap items-baseline justify-between gap-4 max-md:w-[calc(100vw-2.5rem)]"
    >
      <p>Built by Standard Agents. Open Source under MIT.</p>
      <p>
        <a :class="link" href="https://github.com/standardagents/ssh-clipboard">GitHub</a> ·
        <a :class="link" href="https://www.npmjs.com/package/ssh-clipboard">npm</a> ·
        <a :class="link" href="https://github.com/standardagents/ssh-clipboard/blob/main/LICENSE">MIT license</a>
      </p>
    </div>
  </footer>
</template>
