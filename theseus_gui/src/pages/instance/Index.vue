<template>
  <div class="instance-container">
    <div class="instance-nav">
      <div class="instance-gallery-item">
        <p>{{ $route.params.id }}</p>
        <p>{{ getInstance(instances).version }}</p>
        <p>{{ getInstance(instances).name }}</p>
      </div>
      <Button>
        <BoxIcon />
        Mods
      </Button>
      <Button>
        <SettingsIcon />
        Settings
      </Button>
      <Button>
        <FileIcon />
        Logs
      </Button>
    </div>
    <router-view />
  </div>
</template>
<script setup>
import { BoxIcon, SettingsIcon, FileIcon, Button, Avatar } from 'omorphia'
import { useInstances } from '@/store/state'
const instances = useInstances();
instances.fetchInstances();
</script>
<script>
export default {
  methods: {
    getInstance(instances) {
      return instances.instances.find((i) => i.id === parseInt(this.$route.params.id));
    }
  },
}
</script>

<style scoped lang="scss">
.instance-gallery-item {
  width: 9rem;
  height: 9rem;
  background: url('https://avatars1.githubusercontent.com/u/6166773?v=4');
  background-position: center;
  background-size: cover;
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  padding: 0.3rem;
  cursor: pointer;
  transition: all ease-in-out 0.2s;
  border-radius: var(--radius-md);

  &:before {
    content: '';
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    background-image: linear-gradient(to bottom right, rgba(50, 50, 50, 0.9), rgba(0, 0, 0, 0.9));
    opacity: 0.5;
    z-index: 10;
    border-radius: inherit;
  }

  &:hover {
    box-shadow: 0 0 4px 4px rgba(0, 0, 0, 0.5);
  }

  p {
    font-weight: bold;
    font-size: 0.75rem;
    color: #fff;
    z-index: 11;
    margin-bottom: 0.2rem;

    &:nth-child(1) {
      font-weight: normal;
      color: #ddd;
      font-size: 0.6rem;
    }
  }
}

Button {
  width: 100%;
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

.instance-container {
  width: 100%;
  display: flex;
  flex-direction: row;
  overflow: hidden;
}
</style>