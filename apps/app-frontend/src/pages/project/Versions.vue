<template>
  <div>
    <ProjectPageVersions
      :loaders="loaders"
      :game-versions="gameVersions"
      :versions="versions"
      :project="project"
      :version-link="(version) => `/project/${project.id}/version/${version.id}`"
    >
      <template #actions="{ version }">
        <ButtonStyled circular type="transparent">
          <button
            v-tooltip="`Install`"
            :class="{
              'group-hover:!bg-brand group-hover:[&>svg]:!text-brand-inverted':
                !installed || version.id !== installedVersion,
            }"
            :disabled="installing || (installed && version.id === installedVersion)"
            @click.stop="() => install(version.id)"
          >
            <DownloadIcon v-if="!installed" />
            <SwapIcon v-else-if="installed && version.id !== installedVersion" />
            <CheckIcon v-else />
          </button>
        </ButtonStyled>
        <ButtonStyled circular type="transparent">
          <OverflowMenu
            v-if="false"
            class="group-hover:!bg-button-bg"
            :options="[
              {
                id: 'install-elsewhere',
                action: () => {},
                shown: false && !!instance,
                color: 'primary',
                hoverFilled: true,
              },
              {
                id: 'open-in-browser',
                link: `https://modrinth.com/${project.project_type}/${project.slug}/version/${version.id}`,
              },
            ]"
            aria-label="More options"
          >
            <MoreVerticalIcon aria-hidden="true" />
            <template #install-elsewhere>
              <DownloadIcon aria-hidden="true" />
              Add to another instance
            </template>
            <template #open-in-browser> <ExternalIcon /> Open in browser </template>
          </OverflowMenu>
          <a
            v-else
            v-tooltip="`Open in browser`"
            class="group-hover:!bg-button-bg"
            :href="`https://modrinth.com/${project.project_type}/${project.slug}/version/${version.id}`"
            target="_blank"
          >
            <ExternalIcon />
          </a>
        </ButtonStyled>
      </template>
    </ProjectPageVersions>
  </div>
</template>

<script setup>
import { ProjectPageVersions, ButtonStyled, OverflowMenu } from '@modrinth/ui'
import { CheckIcon, DownloadIcon, ExternalIcon, MoreVerticalIcon } from '@modrinth/assets'
import { ref } from 'vue'
import { SwapIcon } from '@/assets/icons/index.js'
import { get_game_versions, get_loaders } from '@/helpers/tags.js'
import { handleError } from '@/store/notifications.js'

defineProps({
  project: {
    type: Object,
    default: () => {},
  },
  versions: {
    type: Array,
    required: true,
  },
  install: {
    type: Function,
    required: true,
  },
  installed: {
    type: Boolean,
    default: null,
  },
  installing: {
    type: Boolean,
    default: false,
  },
  instance: {
    type: Object,
    default: null,
  },
  installedVersion: {
    type: String,
    default: null,
  },
})

const [loaders, gameVersions] = await Promise.all([
  get_loaders().catch(handleError).then(ref),
  get_game_versions().catch(handleError).then(ref),
])
</script>

<style scoped lang="scss">
.filter-header {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.table-row {
  grid-template-columns: min-content 1fr 1fr 1.5fr;
}

.manage {
  display: flex;
  gap: 0.5rem;
  flex-grow: 1;

  .multiselect {
    flex-grow: 1;
  }
}

.card-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: var(--color-raised-bg);
}

.mod-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  overflow: hidden;
  margin-top: 0.5rem;
}

.text-combo {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.select {
  width: 100% !important;
  max-width: 20rem;
}

.version-link {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  text-wrap: wrap;

  .version-badge {
    display: flex;
    flex-wrap: wrap;

    .channel-indicator {
      margin-right: 0.5rem;
    }
  }
}

.stacked-text {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  align-items: flex-start;
}

.download-cell {
  width: 4rem;
  padding: 1rem;
}

.filter-checkbox {
  :deep(.checkbox) {
    border: none;
  }
}
</style>
