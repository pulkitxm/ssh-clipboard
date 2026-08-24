<script setup>
const features = [
  ['native', 'Writes the real system clipboard: NSPasteboard on macOS, Wayland or X11 on Linux.'],
  ['private', 'Persistent peer-to-peer SSH. No relay, no account, no open port, no new encryption key.'],
  ['faithful', 'Ships every clipboard representation: rich text, images, files.'],
  ['invisible', 'Raycast and other clipboard managers see ordinary clipboard writes. Nothing to configure.'],
  ['fast', 'Sends raw bytes over persistent connections through deduplicated, newest-value queues.'],
  ['self-updating', 'Daemons gossip verified versions and converge on the latest stable release. No central coordinator.'],
].map(([name, text]) => ({ name, text }))
</script>

<template>
  <section
    class="mx-auto grid w-[min(90rem,calc(100vw-7rem))] grid-cols-[repeat(auto-fit,minmax(16rem,1fr))] gap-x-16 gap-y-16 pb-8 pt-24 max-md:w-[calc(100vw-2.5rem)] lg:grid-cols-3"
  >
    <div v-for="feature in features" :key="feature.name">
      <h3 class="mb-2 text-[0.95rem] font-semibold text-bright">
        <span class="text-mint">*</span> {{ feature.name }}
      </h3>
      <p class="pl-4 text-[0.85rem] leading-[1.7] text-dim">{{ feature.text }}</p>
    </div>
  </section>
</template>
