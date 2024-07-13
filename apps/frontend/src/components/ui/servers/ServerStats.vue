<template>
  <div data-pyro-server-stats class="flex flex-row items-center gap-6">
    <div
      class="relative min-h-[230px] w-full overflow-hidden rounded-2xl bg-[var(--color-raised-bg)] p-8"
    >
      <div class="flex flex-row items-center gap-2">
        <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">
          {{ (Math.round(cpuPercent * 100) / 100).toFixed(2) }}%
        </h2>
        <ChevronRightIcon />
      </div>
      <h3>CPU Usage</h3>

      <img
        src="https://media.discordapp.net/attachments/1155367404948439081/1254644496486891684/magicpattern-svg-chart-1718988459205_1.png?ex=667a3e3e&is=6678ecbe&hm=ffd0591b51e5da05238563c28f7075942801dfc2d69660d8e206006763d040d1&=&format=webp&quality=lossless&width=820&height=200"
        alt=""
        class="absolute bottom-0 left-0 right-0 w-full"
      />

      <CPUIcon />
    </div>

    <div
      class="relative min-h-[230px] w-full overflow-hidden rounded-2xl bg-[var(--color-raised-bg)] p-8"
    >
      <div class="flex flex-row items-center gap-2">
        <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">
          {{ Math.floor((ramUsageBytes / ramTotalBytes) * 100) }}%
        </h2>
        <ChevronRightIcon />
      </div>
      <h3>Memory Usage</h3>

      <img
        src="https://media.discordapp.net/attachments/1155367404948439081/1254644527671676950/magicpattern-svg-chart-1718988637752_1.png?ex=667a3e45&is=6678ecc5&hm=ee30b5eda2612da25329c9d07f3b5625c0c79f4f93c1cab8fc607b485e1089f2&=&format=webp&quality=lossless&width=820&height=440"
        alt=""
        class="absolute bottom-0 left-0 right-0 w-full"
      />

      <DBIcon />
    </div>

    <div
      class="relative min-h-[230px] w-full overflow-hidden rounded-2xl bg-[var(--color-raised-bg)] p-8"
    >
      <div class="flex flex-row items-center gap-2">
        <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">
          {{ (Math.round((storageUsageBytes / 1e9) * 100) / 100).toFixed(2) }} GB
        </h2>
        <!-- make mb when not decimal -->
        <ChevronRightIcon />
      </div>
      <h3>Storage Usage</h3>

      <div class="flex flex-col gap-2 pt-3">
        <div class="flex flex-row items-center gap-2 text-sm">
          <FolderOpenIcon />
          <p>World</p>
        </div>
        <div class="flex flex-row items-center gap-2 text-sm">
          <FileTextIcon />
          <p>Server Properties</p>
        </div>
        <div class="flex flex-row items-center gap-2 text-sm">
          <FileTextIcon />

          <p>Paper Configuration</p>
        </div>
      </div>

      <FolderOpenIcon />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Stats as StatsType } from '~/types/servers'

import { FileTextIcon, FolderOpenIcon, ChevronRightIcon, CPUIcon, DBIcon } from "@modrinth/assets";

type Stats = Omit<
  StatsType,
  | 'cpu_percent'
  | 'ram_usage_bytes'
  | 'ram_total_bytes'
  | 'storage_usage_bytes'
  | 'storage_total_bytes'
> & {
  cpuPercent: number
  ramUsageBytes: number
  ramTotalBytes: number
  storageUsageBytes: number
  storageTotalBytes: number
}

defineProps<Stats>()
</script>
