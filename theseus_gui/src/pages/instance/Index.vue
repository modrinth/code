<template>
  <div class="instance-container">
    <div class="side-cards">
      <Card class="instance-card">
        <Avatar size="lg" :src="getInstance(instances).img" />
        <div class="instance-info">
          <h2 class="name">{{ getInstance(instances).name }}</h2>
          Fabric {{ getInstance(instances).version }}
        </div>
        <span class="button-group">
          <Button color="primary" class="instance-button">
            <PlayIcon />
            Play
          </Button>
          <Button class="instance-button" icon-only>
            <OpenFolderIcon />
          </Button>
        </span>
      </Card>
      <div class="pages-list">
        <RouterLink :to="`/instance/${$route.params.id}/`" class="btn">
          <BoxIcon />
          Mods
        </RouterLink>
        <RouterLink :to="`/instance/${$route.params.id}/options`" class="btn">
          <SettingsIcon />
          Options
        </RouterLink>
        <RouterLink :to="`/instance/${$route.params.id}/logs`" class="btn">
          <FileIcon />
          Logs
        </RouterLink>
      </div>
    </div>
    <div class="content">
      <Promotion />
      <router-view />
    </div>
  </div>
</template>
<script setup>
import { BoxIcon, SettingsIcon, FileIcon, Button, Avatar, Card, Promotion } from 'omorphia'
import { PlayIcon, OpenFolderIcon } from '@/assets/icons'
import { useInstances } from '@/store/state'

const instances = useInstances()
instances.fetchInstances()
</script>
<script>
export default {
  methods: {
    getInstance(instances) {
      return instances.instances.find((i) => i.id === parseInt(this.$route.params.id))
    },
  },
}
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
  min-height: calc(100% - 2rem);
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
