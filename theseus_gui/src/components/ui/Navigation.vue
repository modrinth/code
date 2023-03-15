<script setup>
import { SearchIcon, BookIcon, ClientIcon, PlusIcon, SettingsIcon, Button } from 'omorphia'
import { useInstances } from '@/store/state'
import { RouterLink } from 'vue-router'
import UserSection from '@/components/ui/UserSection.vue'
import Instance from '@/components/ui/Instance.vue'
const instances = useInstances()

instances.fetchInstances()
</script>

<template>
  <div class="nav-container">
    <UserSection />
    <div class="pages-list">
      <RouterLink to="/" class="btn"> <ClientIcon />Home</RouterLink>
      <RouterLink to="/browse" class="btn"> <SearchIcon />Browse</RouterLink>
      <RouterLink to="/library" class="btn"> <BookIcon />Library</RouterLink>
    </div>
    <div class="instance-list">
      <p>Instances</p>
      <Instance v-for="instance in instances.instances" display="list" :instance="instance" />
    </div>
    <Button color="raised">
      <PlusIcon />
      Create Instance
    </Button>

    <div id="settings">
      <RouterLink to="/settings" class="btn"><SettingsIcon /> Settings</RouterLink>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.nav-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 210px;
  height: 100vh;
  box-shadow: var(--shadow-inset-sm), var(--shadow-floating);
}

.dark-mode {
  .nav-container {
    background: var(--color-bg);
  }
}
.pages-list {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: flex-start;
  width: 100%;

  a {
    display: flex;
    align-items: flex-start;
    width: 80%;
    margin: 0.3rem auto;
    text-align: left;
    font-size: 0.9rem;
    font-weight: 400;
    background: inherit;
    transition: all ease-in-out 0.1s;
    color: var(--color-primary);

    &:hover {
      background-color: var(--color-button-bg);
      box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
      text-decoration: none;
    }

    svg {
      margin-right: 0.25rem;
    }
  }
}

.instance-list {
  display: flex;
  flex-direction: column;
  justify-content: center;
  width: 70%;
  margin: 0.4rem;

  & > p {
    color: var(--color-base);
    margin: 1rem 0;
    font-size: 1.2rem;
    line-height: 13px;
    font-weight: 500;
    text-transform: uppercase;
  }
}

#settings {
  display: flex;
  align-items: center;
  margin: auto 1rem 0.5rem 1rem;
  width: 150px;

  a {
    display: flex;
    justify-content: space-evenly;
    align-items: center;
    width: 80%;
    margin: auto;
    background: inherit;
    color: var(--color-primary);

    &:hover {
      text-decoration: none;
    }
  }
}
</style>
