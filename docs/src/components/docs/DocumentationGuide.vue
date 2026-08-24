<script setup>
import { onBeforeUnmount, onMounted, ref } from 'vue'
import CodeBlock from '../CodeBlock.vue'
import EarlyAccess from '../EarlyAccess.vue'

const sections = [
  { id: 'install', title: 'Install' },
  { id: 'quick-start', title: 'Quick start' },
  { id: 'commands', title: 'Commands' },
  { id: 'how-it-works', title: 'How it works' },
  { id: 'updates', title: 'Updates' },
]

const commands = [
  ['ssh-clipboard', 'First run: setup TUI. After that: the live monitor dashboard.'],
  ['ssh-clipboard setup', 'Add, verify, or repair peers. Re-runs installation where needed.'],
  ['ssh-clipboard monitor', 'Watch clipboard values and peer health in a Ratatui dashboard. --plain streams readable lines, --json streams NDJSON.'],
  ['ssh-clipboard status', 'Daemon and connection status. --json for automation.'],
  ['ssh-clipboard update', 'Install the latest stable release. --check compares versions without installing.'],
  ['ssh-clipboard service', 'Manage the per-user background service: install, start, stop, restart.'],
].map(([cmd, desc]) => ({ cmd, desc }))

const activeId = ref('install')
const docLink = 'text-mint border-b border-mint/35 transition-colors hover:border-mint'
let observer

onMounted(() => {
  observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) activeId.value = entry.target.id
      }
    },
    { rootMargin: '-20% 0px -70% 0px' }
  )
  for (const section of sections) {
    const element = document.getElementById(section.id)
    if (element) observer.observe(element)
  }
})

onBeforeUnmount(() => observer?.disconnect())
</script>

<template>
  <div
    class="mx-auto mt-28 grid w-[min(90rem,calc(100vw-7rem))] grid-cols-[13rem_minmax(0,1fr)] gap-14 max-md:w-[calc(100vw-2.5rem)] max-[1150px]:grid-cols-[minmax(0,1fr)]"
  >
    <aside
      class="sticky top-[6.5rem] mt-[5.25rem] flex flex-col gap-1.5 self-start text-[0.95rem] max-[1150px]:hidden"
      aria-label="On this page"
    >
      <a
        v-for="section in sections"
        :key="section.id"
        :href="'#' + section.id"
        class="w-fit whitespace-pre py-1 transition-colors"
        :class="activeId === section.id ? 'text-mint' : 'text-dim hover:text-bright'"
      >{{ (activeId === section.id ? '▌ ' : '  ') + section.title }}</a>
    </aside>

    <div class="mx-auto w-full min-w-0 max-w-[46rem]">
      <section id="install" class="pb-6 pt-20">
        <h2 class="mb-6 text-[1.4rem] font-semibold text-bright">
          <span class="text-mint">#</span> Install
        </h2>
        <p class="mb-4 max-w-[42rem]">
          The npm package installs a native Rust binary and a per-user service
          (<code>launchd</code> on macOS, <code>systemd</code> on Linux).
        </p>
        <CodeBlock code="$ npm i -g ssh-clipboard
$ ssh-clipboard" />
        <p class="mb-4 max-w-[42rem]">You'll need:</p>
        <ul class="mb-4 ml-6 list-disc space-y-1.5">
          <li>macOS or Linux (Wayland or X11), on arm64 or x64</li>
          <li>Node ≥ 18 for <code>npm install</code>; the daemon has no Node dependency</li>
          <li>
            <a :class="docLink" href="https://tailscale.com/kb/1193/tailscale-ssh">Tailscale SSH</a>
            is recommended but passwordless <code>~/.ssh</code> keys work too
          </li>
        </ul>
        <h3 class="mb-3 mt-8 text-[1rem] font-semibold text-bright">Headless Linux</h3>
        <p class="mb-4 max-w-[42rem]">
          A server without Wayland or X11 has no native clipboard. Setup detects that condition
          and offers an opt-in, local-only Xvfb display after the Xvfb package is installed. It
          configures <code>DISPLAY=:99</code>, starts both per-user services in order, and keeps
          that choice through updates.
          <a
            :class="docLink"
            href="https://github.com/standardagents/ssh-clipboard/blob/main/docs/headless-linux.md"
          >Read the headless Linux guide.</a>
        </p>
      </section>

      <section id="quick-start" class="pb-6 pt-20">
        <h2 class="mb-6 text-[1.4rem] font-semibold text-bright">
          <span class="text-mint">#</span> Quick start
        </h2>
        <p class="mb-4 max-w-[42rem]">
          Run <code>ssh-clipboard</code> with no arguments. The first-run TUI lists compatible
          online Tailscale machines, or accepts any passwordless SSH destination. For each peer it:
        </p>
        <ol class="mb-4 ml-6 list-decimal space-y-1.5">
          <li>verifies the connection,</li>
          <li>installs the right binary over SSH,</li>
          <li>starts the per-user service on both ends.</li>
        </ol>
        <p class="mb-4 max-w-[42rem]">
          Copy on one machine, paste on another. After setup it behaves like one clipboard.
        </p>
        <CodeBlock code="$ ssh-clipboard status
running as macbook (node-a1, pasteboard, version 0.2.0)
connected: fedora
connected: macbookserver" />
      </section>

      <section id="commands" class="pb-6 pt-20">
        <h2 class="mb-6 text-[1.4rem] font-semibold text-bright">
          <span class="text-mint">#</span> Commands
        </h2>
        <dl class="my-8 overflow-hidden border border-line">
          <template v-for="(command, index) in commands" :key="command.cmd">
            <dt class="bg-panel px-5 pt-4">
              <code class="border-0 bg-transparent p-0 font-semibold">
                <span :class="index === 0 ? 'text-mint' : 'text-bright'">ssh-clipboard</span><span
                  class="text-mint"
                >{{ command.cmd.slice('ssh-clipboard'.length) }}</span>
              </code>
            </dt>
            <dd class="border-b border-line bg-panel px-5 pb-4 pt-1 text-[0.88rem] text-dim last:border-b-0">
              {{ command.desc }}
            </dd>
          </template>
        </dl>
        <p class="mb-4 max-w-[42rem]">
          Every command has a machine-readable twin: <code>status --json</code> for health checks,
          <code>monitor --json</code> for an NDJSON event stream you can pipe anywhere.
        </p>
      </section>

      <section id="how-it-works" class="pb-6 pt-20">
        <h2 class="mb-6 text-[1.4rem] font-semibold text-bright">
          <span class="text-mint">#</span> How it works
        </h2>
        <pre class="my-8 overflow-x-auto border border-line bg-panel px-6 py-5 text-[0.78rem] leading-[1.45] text-dim">
┌──────────────┐         encrypted SSH         ┌──────────────┐
│    macbook   │ ◀═══════════════════════════▶ │    fedora    │
│  pasteboard  │    persistent · deduplicated  │ wayland/x11  │
└──────────────┘         newest-wins           └──────────────┘</pre>
        <p class="mb-4 max-w-[42rem]">
          A small Rust daemon on each machine watches the system clipboard through native backends.
          On change, it ships the raw bytes of <em>every representation</em> to its peers over
          persistent SSH and writes them back natively.
        </p>
        <p class="mb-4 max-w-[42rem]">
          No relay, cloud account, additional port forwarding required. Values are deduplicated,
          and per-peer queues always deliver the newest value.
        </p>
      </section>

      <section id="updates" class="pb-6 pt-20">
        <h2 class="mb-6 text-[1.4rem] font-semibold text-bright">
          <span class="text-mint">#</span> Updates
        </h2>
        <p class="mb-4 max-w-[42rem]">
          Each daemon checks npm for the latest stable release and tells its peers what it found,
          so any machine that's online can update the whole mesh. There's no central update server.
        </p>
        <p class="mb-4 max-w-[42rem]">
          Before installing anything, a daemon verifies the npm SHA-512 hash, the bundled SHA-256
          manifest, the executable target, and the version the binary actually reports. Updates
          keep the old executable around, swap the new one in atomically, and let launchd or
          systemd restart the daemon.
        </p>
        <CodeBlock code="$ ssh-clipboard update --check
current: 0.2.0
latest:  0.2.0" />
      </section>

      <EarlyAccess />
    </div>
  </div>
</template>
