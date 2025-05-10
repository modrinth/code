<script setup lang="ts">
import type {GameInstance} from "@/helpers/types";
import type ContextMenu from "@/components/ui/ContextMenu.vue";
import type {Version} from "@modrinth/utils";
import {getAllProfileScreenshots} from "@/helpers/screenshots.ts";

const props = defineProps<{
  instance: GameInstance
  options: InstanceType<typeof ContextMenu> | null
  offline: boolean
  playing: boolean
  versions: Version[]
  installed: boolean
}>()

const screenshots = await getAllProfileScreenshots(props.instance.path);
</script>

<template>
  <div class="card">
    {{ screenshots }}
  </div>
</template>