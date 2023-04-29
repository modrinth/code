<template>
  <div class="instance-container">
    <div class="side-cards">
      <Card class="instance-card">
        <Avatar size="lg" :src="convertFileSrc(instance.metadata.icon)" />
        <div class="instance-info">
          <h2 class="name">{{ instance.metadata.name }}</h2>
          <span class="metadata">
            {{ instance.metadata.loader }} {{ instance.metadata.game_version }}
          </span>
        </div>
        <span class="button-group">
          <Button
            v-if="playing === true"
            color="danger"
            class="instance-button"
            @click="stopInstance"
            @mouseover="checkProcess"
          >
            <XIcon />
            Stop
          </Button>
          <Button
            v-else-if="playing === false && loading === false"
            color="primary"
            class="instance-button"
            @click="startInstance"
            @mouseover="checkProcess"
          >
            <PlayIcon />
            Play
          </Button>
          <Button v-else-if="loading === true && playing === false" disabled class="instance-button"
            >Loading...</Button
          >
          <Button class="instance-button" icon-only @click="open({ defaultPath: instance.path })">
            <OpenFolderIcon />
          </Button>
        </span>
      </Card>
      <div class="pages-list">
        <RouterLink :to="`/instance/${encodeURIComponent($route.params.id)}/`" class="btn">
          <BoxIcon />
          Mods
        </RouterLink>
        <RouterLink :to="`/instance/${encodeURIComponent($route.params.id)}/options`" class="btn">
          <SettingsIcon />
          Options
        </RouterLink>
        <RouterLink :to="`/instance/${encodeURIComponent($route.params.id)}/logs`" class="btn">
          <FileIcon />
          Logs
        </RouterLink>
      </div>
    </div>
    <div class="content">
      <Promotion />
      <router-view :instance="instance" />
    </div>
  </div>
</template>
<script setup>
import { BoxIcon, SettingsIcon, FileIcon, XIcon, Button, Avatar, Card, Promotion } from 'omorphia'
import { PlayIcon, OpenFolderIcon } from '@/assets/icons'
import { get, run } from '@/helpers/profile'
import {
  get_all_running_profile_paths,
  get_uuids_by_profile_path,
  kill_by_uuid,
} from '@/helpers/process'
import { process_listener } from '@/helpers/events'
import { useRoute } from 'vue-router'
import { shallowRef, ref } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const breadcrumbs = useBreadcrumbs()

breadcrumbs.setName('Instance', instance.value.metadata.name)
breadcrumbs.setContext({
  name: instance.value.metadata.name,
  link: route.path,
})

const route = useRoute()
const instance = shallowRef(await get(route.params.id))
const uuid = ref(null)
const playing = ref(false)
const loading = ref(false)

const startInstance = async () => {
  loading.value = true
  uuid.value = await run(route.params.id)
  loading.value = false
  playing.value = true
}

const checkProcess = async () => {
  const runningPaths = await get_all_running_profile_paths()
  if (runningPaths.includes(instance.value.path)) {
    playing.value = true
    return
  }

  playing.value = false
  uuid.value = null
}

await checkProcess()

const stopInstance = async () => {
  playing.value = false

  try {
    if (!uuid.value) {
      const uuids = await get_uuids_by_profile_path(instance.value.path)
      uuids.forEach(async (u) => await kill_by_uuid(u))
    } else await kill_by_uuid(uuid.value)
  } catch (err) {
    // Theseus currently throws:
    //  "Error launching Minecraft: Minecraft exited with non-zero code 1" error
    // For now, we will catch and just warn
    console.warn(err)
  }
}

await process_listener((e) => {
  if (e.event === 'Finished') playing.value = false
})
</script>

<style scoped lang="scss">
.instance-card {
  background: var(--color-bg);
  display: flex;
  flex-direction: column;
  gap: 1rem;
  width: 15rem;
}

Button {
  width: 100%;
}

.button-group {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
}

.side-cards {
  position: absolute;
  display: flex;
  flex-direction: column;
  padding: 1rem;
  background: var(--color-raised-bg);
  min-height: calc(100% - 3.25rem);
  overflow: hidden;
}

.instance-nav {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  padding: 1rem;
  gap: 0.5rem;
  background: var(--color-raised-bg);
  height: 100%;
}

.name {
  font-size: 1.25rem;
  color: var(--color-contrast);
}

.metadata {
  text-transform: capitalize;
}

.instance-container {
  display: flex;
  flex-direction: row;
  overflow: auto;
  gap: 1rem;
}

.content {
  margin-left: 18rem;
}

.instance-info {
  display: flex;
  flex-direction: column;
  width: 100%;
}

.badge {
  display: flex;
  align-items: center;
  font-weight: bold;
  width: fit-content;
  color: var(--color-orange);
}

.pages-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;

  a {
    font-size: 100%;
    font-weight: 400;
    background: inherit;
    transition: all ease-in-out 0.1s;
    width: 100%;
    color: var(--color-primary);
    padding: var(--gap-md);

    &.router-link-exact-active {
      background: var(--color-button-bg);
    }

    &:hover {
      background-color: var(--color-button-bg);
      color: var(--color-contrast);
      box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
      text-decoration: none;
    }

    svg {
      width: 1.3rem;
      height: 1.3rem;
    }
  }
}

.header-nav {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  padding: 0.5rem;
  gap: 0.5rem;
  background: var(--color-raised-bg);
}

.project-card {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  gap: 1rem;
  background: var(--color-raised-bg);
  width: 20rem;
}

.instance-nav {
  display: flex;
  flex-direction: row;
  align-items: flex-start;
  justify-content: left;
  padding: 1rem;
  gap: 0.5rem;
  background: var(--color-raised-bg);
  height: min-content;
  width: 100%;
}

.instance-button {
  width: fit-content;
}

.actions {
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  gap: 0.5rem;
}

.content {
  width: 100%;
  display: flex;
  flex-direction: column;
  padding: 1rem 1rem 0 0;
  overflow: auto;
}

.stats {
  grid-area: stats;
  display: flex;
  flex-direction: column;
  flex-wrap: wrap;
  gap: var(--gap-md);

  .stat {
    display: flex;
    flex-direction: row;
    align-items: center;
    width: fit-content;
    gap: var(--gap-xs);
    --stat-strong-size: 1.25rem;

    strong {
      font-size: var(--stat-strong-size);
    }

    p {
      margin: 0;
    }

    svg {
      height: var(--stat-strong-size);
      width: var(--stat-strong-size);
    }
  }

  .date {
    margin-top: auto;
  }

  @media screen and (max-width: 750px) {
    flex-direction: row;
    column-gap: var(--gap-md);
    margin-top: var(--gap-xs);
  }

  @media screen and (max-width: 600px) {
    margin-top: 0;

    .stat-label {
      display: none;
    }
  }
}

.card-divider {
  background-color: var(--color-button-bg);
  border: none;
  color: var(--color-button-bg);
  height: 1px;
  margin: var(--gap-xl) 0;
}
</style>
