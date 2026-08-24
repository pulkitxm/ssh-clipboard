<script setup>
import InstallPill from '../InstallPill.vue'
import MeshDemo from '../MeshDemo.vue'
</script>

<template>
  <section class="relative overflow-x-clip px-5 pb-24 pt-32 text-left max-xl:text-center max-md:pt-16">
    <div
      class="mx-auto grid w-[min(90rem,calc(100vw-7rem))] grid-cols-[minmax(21rem,27rem)_minmax(0,1fr)] items-center gap-16 max-xl:grid-cols-1 max-md:gap-8 max-md:w-[calc(100vw-2.5rem)]"
    >
      <div>
        <h1
          class="mb-8 text-[clamp(2.8rem,5.2vw,4.1rem)] font-semibold leading-[1.12] tracking-[-0.03em] text-bright"
        >
          <span>copy/paste,</span><br />
          <span class="text-mint">everywhere.</span>
        </h1>
        <p class="mb-10 max-w-[38rem] text-[1.05rem] leading-[1.85] text-dim max-xl:mx-auto">
          Peer-to-peer clipboard sync over SSH.
          Copy anything on one machine, paste on another. Open source, and
          <span class="text-mint">Written in Rust ackchyually.</span>
        </p>
        <InstallPill />
      </div>
      <div class="relative isolate min-w-0">
        <div
          class="dither pointer-events-none absolute -inset-x-48 -inset-y-28 -z-10 text-mint opacity-[0.13] [mask-image:radial-gradient(70%_78%_at_50%_46%,black,transparent_82%)] max-md:-inset-x-3"
        ></div>
        <MeshDemo />
      </div>
    </div>
  </section>
</template>
