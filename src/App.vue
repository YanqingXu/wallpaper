<script setup lang="ts">
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import {
  AlertTriangle,
  CheckCircle2,
  FolderOpen,
  ImageIcon,
  RefreshCw,
  Shuffle,
} from '@lucide/vue';
import { computed, onMounted, ref } from 'vue';

type WallpaperSource = 'bundled' | 'user';

interface WallpaperItem {
  id: string;
  label: string;
  path: string;
  source: WallpaperSource;
}

const wallpapers = ref<WallpaperItem[]>([]);
const current = ref<WallpaperItem | null>(null);
const status = ref('正在加载壁纸');
const statusKind = ref<'info' | 'success' | 'error'>('info');
const busy = ref(false);

const bundledCount = computed(
  () => wallpapers.value.filter((item) => item.source === 'bundled').length,
);
const userCount = computed(
  () => wallpapers.value.filter((item) => item.source === 'user').length,
);

onMounted(() => {
  void refreshWallpapers();
});

function imageSrc(path: string) {
  return convertFileSrc(path);
}

function sourceLabel(source: WallpaperSource) {
  return source === 'bundled' ? '内置' : '本地';
}

function setStatus(message: string, kind: 'info' | 'success' | 'error' = 'info') {
  status.value = message;
  statusKind.value = kind;
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

async function refreshWallpapers() {
  try {
    wallpapers.value = await invoke<WallpaperItem[]>('list_wallpapers');
    setStatus(`可用壁纸 ${wallpapers.value.length} 张`, 'info');
  } catch (error) {
    setStatus(errorMessage(error), 'error');
  }
}

async function chooseLocalWallpaper() {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [
      {
        name: '图片',
        extensions: ['png', 'jpg', 'jpeg', 'bmp'],
      },
    ],
  });

  const path = Array.isArray(selected) ? selected[0] : selected;
  if (!path) {
    setStatus('未选择图片', 'info');
    return;
  }

  await runAction(async () => {
    wallpapers.value = await invoke<WallpaperItem[]>('add_user_wallpaper', { path });
    const added = wallpapers.value.find((item) => item.path === path);
    setStatus(`已加入 ${added?.label ?? '本地图片'}`, 'success');
  });
}

async function setRandomWallpaper() {
  await runAction(async () => {
    const selected = await invoke<WallpaperItem>('set_random_wallpaper');
    current.value = selected;
    setStatus(`已切换到 ${selected.label}`, 'success');
  });
}

async function setSpecificWallpaper(item: WallpaperItem) {
  await runAction(async () => {
    await invoke('set_wallpaper', { path: item.path });
    current.value = item;
    setStatus(`已设置 ${item.label}`, 'success');
  });
}

async function runAction(action: () => Promise<void>) {
  if (busy.value) {
    return;
  }

  busy.value = true;
  try {
    await action();
  } catch (error) {
    setStatus(errorMessage(error), 'error');
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <main class="app-shell">
    <header class="topbar">
      <div>
        <p class="eyebrow">Wallpaper Switcher</p>
        <h1>桌面壁纸切换器</h1>
      </div>

      <div class="stats" aria-label="壁纸数量">
        <span>{{ bundledCount }} 内置</span>
        <span>{{ userCount }} 本地</span>
      </div>
    </header>

    <section class="controls" aria-label="壁纸操作">
      <button class="primary-action" :disabled="busy" @click="setRandomWallpaper">
        <Shuffle :size="20" aria-hidden="true" />
        <span>随机切换</span>
      </button>

      <button class="secondary-action" :disabled="busy" @click="chooseLocalWallpaper">
        <FolderOpen :size="20" aria-hidden="true" />
        <span>选择图片</span>
      </button>

      <button class="icon-action" :disabled="busy" title="刷新" aria-label="刷新" @click="refreshWallpapers">
        <RefreshCw :size="20" aria-hidden="true" />
      </button>
    </section>

    <section class="status-line" :class="`status-${statusKind}`" aria-live="polite">
      <CheckCircle2 v-if="statusKind === 'success'" :size="18" aria-hidden="true" />
      <AlertTriangle v-else-if="statusKind === 'error'" :size="18" aria-hidden="true" />
      <ImageIcon v-else :size="18" aria-hidden="true" />
      <span>{{ status }}</span>
    </section>

    <section class="current-strip" v-if="current">
      <span>当前</span>
      <strong>{{ current.label }}</strong>
    </section>

    <section class="wallpaper-grid" aria-label="壁纸列表">
      <button
        v-for="item in wallpapers"
        :key="item.id"
        class="wallpaper-tile"
        :class="{ active: current?.path === item.path }"
        :disabled="busy"
        @click="setSpecificWallpaper(item)"
      >
        <img :src="imageSrc(item.path)" :alt="item.label" loading="lazy" />
        <span class="tile-meta">
          <span class="tile-title">{{ item.label }}</span>
          <span class="source-badge">{{ sourceLabel(item.source) }}</span>
        </span>
      </button>
    </section>
  </main>
</template>
