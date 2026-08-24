export const COLORS = {
  line: '#262626',
  lineBright: '#3d3d3d',
  bright: '#f2f2f2',
  dim: '#8b8b8b',
  faint: '#595959',
  mint: '#4ee585',
  tuiBg: '#13151b',
  accent: '#a78bfa',
  cyan: '#22d3ee',
  green: '#34d399',
  yellow: '#fbbf24',
  muted: '#64748b',
  soft: '#cbd5e1',
  tuiPanel: '#3d3d3d',
  chipBg: '#0f1620',
  bg: '#000000',
}

export const MONO = "ui-monospace, 'SF Mono', 'JetBrains Mono', Menlo, Consolas, monospace"
export const LOOP = 16000
export const clamp01 = (value) => Math.max(0, Math.min(1, value))
export const seg = (time, start, end) => clamp01((time - start) / (end - start))
export const outQuint = (progress) => 1 - (1 - progress) ** 5
export const inCubic = (progress) => progress ** 3
export const trap = (progress) =>
  progress <= 0 || progress >= 1 ? 0 : Math.min(1, progress / 0.12, (1 - progress) / 0.18)

export const TIMELINE = {
  sel: [0.04, 0.105],
  flash: [0.115, 0.132],
  deskAClose: [0.132, 0.162],
  flyA: [0.168, 0.205],
  sentTotal: 0.267,
  fanA: [0.215, 0.265],
  recvA: [0.265, 0.32],
  deskB: [0.285, 0.32, 0.445, 0.475],
  deskC: [0.495, 0.53, 0.716, 0.746],
  hlC: [0.676, 0.706],
  flyB: [0.75, 0.79],
  recvTotal: 0.795,
  fanB: [0.805, 0.855],
  recvB: [0.855, 0.91],
  deskD: [0.875, 0.908],
  macPtr: [0.9, 0.945],
  codeFill: 0.948,
  fade: [0.955, 0.99],
}

export const IMAGE_PREVIEW = '<Apple PNG past…'
export const TEXT_PREVIEW = '01923091283'
export const ACTIVITY_ROWS = [
  { at: 0.205, time: '41.204', flow: '◆ copied here', fc: COLORS.accent, pv: IMAGE_PREVIEW, size: '8.8 MiB', fm: '6' },
  { at: 0.218, time: '41.530', flow: '→ fedora', fc: COLORS.cyan, pv: IMAGE_PREVIEW, size: '8.8 MiB', fm: '6' },
  { at: 0.224, time: '41.530', flow: '→ debian', fc: COLORS.cyan, pv: IMAGE_PREVIEW, size: '8.8 MiB', fm: '6' },
  { at: 0.23, time: '41.530', flow: '→ mini', fc: COLORS.cyan, pv: IMAGE_PREVIEW, size: '8.8 MiB', fm: '6' },
  { at: 0.792, time: '47.214', flow: '← fedora', fc: COLORS.green, pv: TEXT_PREVIEW, size: '11 B', fm: '1' },
  { at: 0.808, time: '47.215', flow: '→ debian', fc: COLORS.cyan, pv: TEXT_PREVIEW, size: '11 B', fm: '1' },
  { at: 0.814, time: '47.215', flow: '→ mini', fc: COLORS.cyan, pv: TEXT_PREVIEW, size: '11 B', fm: '1' },
  { at: 0.82, time: '47.215', flow: '→ macbook', fc: COLORS.cyan, pv: TEXT_PREVIEW, size: '11 B', fm: '1' },
]

export const SCENE_W = 440
export const SCENE_H = 292
export const CAPTURE = { x: 252, y: 102, w: 116, h: 90 }
export const STAT_H = 48

function curve(p0, p1, vertical) {
  const c1 = vertical
    ? { x: p0.x, y: p0.y + (p1.y - p0.y) * 0.45 }
    : { x: p0.x + (p1.x - p0.x) * 0.45, y: p0.y }
  const c2 = vertical
    ? { x: p1.x, y: p1.y - (p1.y - p0.y) * 0.45 }
    : { x: p1.x - (p1.x - p0.x) * 0.45, y: p1.y }
  return {
    at(time) {
      const remaining = 1 - time
      return {
        x:
          remaining ** 3 * p0.x +
          3 * remaining * remaining * time * c1.x +
          3 * remaining * time * time * c2.x +
          time ** 3 * p1.x,
        y:
          remaining ** 3 * p0.y +
          3 * remaining * remaining * time * c1.y +
          3 * remaining * time * time * c2.y +
          time ** 3 * p1.y,
      }
    },
    p0,
    c1,
    c2,
    p1,
  }
}

function monitorHeight(fontSize) {
  return Math.round(14 * fontSize * 1.62 + 108)
}

export function meshLayout(width) {
  const nodeWidth = 118
  const nodeHeight = 96
  const wide = width >= 620
  const nodes = {}
  let mon
  let fontSize
  let height
  let sceneScale
  if (wide) {
    const monWidth = Math.max(300, Math.min(470, width - 2 * nodeWidth - 4 * 26))
    fontSize = Math.max(9, Math.min(12, monWidth / 40))
    const monHeight = monitorHeight(fontSize)
    const sceneHeight = Math.max(monHeight + 56, 476)
    height = sceneHeight + STAT_H
    mon = { x: (width - monWidth) / 2, y: (sceneHeight - monHeight) / 2 + 14, w: monWidth, h: monHeight }
    nodes.macbook = { x: 8, y: sceneHeight - nodeHeight - 6, w: nodeWidth, h: nodeHeight }
    const right = width - nodeWidth - 8
    const gap = (sceneHeight - 3 * nodeHeight) / 4
    nodes.fedora = { x: right, y: gap, w: nodeWidth, h: nodeHeight }
    nodes.debian = { x: right, y: gap * 2 + nodeHeight, w: nodeWidth, h: nodeHeight }
    nodes.mini = { x: right, y: gap * 3 + nodeHeight * 2, w: nodeWidth, h: nodeHeight }
    sceneScale = Math.min(1, (width * 0.46) / SCENE_W)
  } else {
    const monWidth = Math.min(440, width - 8)
    fontSize = Math.max(8.5, Math.min(11, monWidth / 40))
    const monHeight = monitorHeight(fontSize)
    const gap = 46
    height = nodeHeight + gap + monHeight + gap + nodeHeight + 16 + STAT_H
    mon = { x: (width - monWidth) / 2, y: nodeHeight + gap + 8, w: monWidth, h: monHeight }
    nodes.macbook = { x: (width - nodeWidth) / 2, y: 8, w: nodeWidth, h: nodeHeight }
    const bottomWidth = Math.min(nodeWidth, (width - 32) / 3)
    const bottomY = mon.y + monHeight + gap
    const bottomGap = (width - 3 * bottomWidth) / 4
    nodes.fedora = { x: bottomGap, y: bottomY, w: bottomWidth, h: nodeHeight }
    nodes.mini = { x: bottomGap * 2 + bottomWidth, y: bottomY, w: bottomWidth, h: nodeHeight }
    nodes.debian = { x: bottomGap * 3 + bottomWidth * 2, y: bottomY, w: bottomWidth, h: nodeHeight }
    sceneScale = Math.min(1, (width - 16) / SCENE_W)
  }
  const centerX = (rect) => rect.x + rect.w / 2
  const centerY = (rect) => rect.y + rect.h / 2
  const wires = {}
  let deskARect
  let deskBRect
  if (wide) {
    wires.macbook = curve({ x: nodes.macbook.x + nodeWidth, y: centerY(nodes.macbook) }, { x: mon.x, y: mon.y + mon.h * 0.72 }, false)
    wires.fedora = curve({ x: mon.x + mon.w, y: mon.y + mon.h * 0.24 }, { x: nodes.fedora.x, y: centerY(nodes.fedora) }, false)
    wires.debian = curve({ x: mon.x + mon.w, y: mon.y + mon.h * 0.5 }, { x: nodes.debian.x, y: centerY(nodes.debian) }, false)
    wires.mini = curve({ x: mon.x + mon.w, y: mon.y + mon.h * 0.76 }, { x: nodes.mini.x, y: centerY(nodes.mini) }, false)
    deskARect = { x: 8, y: 8 }
    deskBRect = { x: width - SCENE_W * sceneScale - 8, y: 8 }
  } else {
    wires.macbook = curve({ x: centerX(nodes.macbook), y: nodes.macbook.y + nodes.macbook.h }, { x: centerX(mon), y: mon.y }, true)
    for (const name of ['fedora', 'mini', 'debian']) {
      wires[name] = curve({ x: centerX(nodes[name]), y: mon.y + mon.h }, { x: centerX(nodes[name]), y: nodes[name].y }, true)
    }
    const x = (width - SCENE_W * sceneScale) / 2
    const y = mon.y + (mon.h - SCENE_H * sceneScale) / 2
    deskARect = { x, y }
    deskBRect = { x, y }
  }
  return { w: width, H: height, fs: fontSize, mon, nodes, wires, wide, sceneScale, deskARect, deskBRect }
}
