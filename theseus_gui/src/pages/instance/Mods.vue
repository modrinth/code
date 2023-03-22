<template>
  <div class="table-container">
    <div class="table-row table-head">
      <div class="table-cell table-text"> Mod Name </div>
      <div class="table-cell table-text"> Mod Description </div>
      <div class="table-cell manage">
        <Button color="primary">
          <PlusIcon />
          Add
        </Button>
      </div>
    </div>
    <div class="table-row" v-for="mod in mods" :key="mod.name">
      <div class="table-cell table-text">
        <span class="mod-text">
          <Avatar :src="mod.icon"/>
          {{ mod.name }}
          <span class="version">
            {{ mod.version }}
          </span>
        </span>
      </div>
      <div class="table-cell table-text"> {{ mod.description }} </div>
      <div class="table-cell manage">
        <Button color="danger" iconOnly>
          <TrashIcon />
        </Button>
        <input
          type="checkbox"
          class="switch stylized-toggle"
          :checked="true"
        />
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: "Mods"
}
</script>
<script setup>
import { Avatar, Button, TrashIcon, PlusIcon, Checkbox } from 'omorphia'
const mods = [
  {
    name: "Fabric API",
    icon: "https://cdn.modrinth.com/data/P7dR8mSH/icon.png",
    version: "0.76.0+1.19.4",
    author: "modmuss50",
    description: "Lightweight and modular API providing common hooks and intercompatibility measures utilized by mods using the Fabric toolchain."
  },
  {
    name: "Spirit",
    icon: "https://cdn.modrinth.com/data/b1LdOZlE/465598dc5d89f67fb8f8de6def21240fa35e3a54.png",
    version: "2.2.4",
    author: "CodexAdrian",
    description: "Create your own configurable mob spawner!"
  },
  {
    name: "Botarium",
    icon: "https://cdn.modrinth.com/data/2u6LRnMa/98b286b0d541ad4f9409e0af3df82ad09403f179.gif",
    version: "2.0.5",
    author: "CodexAdrian",
    description: "A crossplatform API for devs that makes transfer and storage of items, fluids and energy easier, as well as some other helpful things"
  },
  {
    name: "Tempad",
    icon: "https://cdn.modrinth.com/data/gKNwt7xu/icon.gif",
    version: "2.2.4",
    author: "Mod Author",
    description: "Create a portal to anywhere from anywhere"
  },
  {
    name: "Sodium",
    icon: "https://cdn.modrinth.com/data/AANobbMI/icon.png",
    version: "0.4.10",
    description: "Modern rendering engine and client-side optimization mod for Minecraft"
  }
]
</script>

<style scoped lang="scss">
.table-container {
  display: grid;
  grid-template-rows: repeat(auto-fill, auto);
  width: 100%;
  border-radius: var(--radius-md);
  overflow: hidden;
}

.table-row {
  display: grid;
  grid-template-columns: 2fr 3fr 1fr;
}

.table-head {
  .table-cell {
    background-color: var(--color-accent-contrast);
  }
}

.table-cell {
  padding: 1rem;
  height: 100%;
  align-items: center;
  vertical-align: center;
  display: flex;
}

.table-text {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: fade;
}

.manage {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.mod-text {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--color-contrast);

  .version {
    color: var(--color-base);
  }
}

.table-row:nth-child(even) .table-cell {
  background-color: var(--color-raised-bg);
}
</style>