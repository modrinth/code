<template>
  <table class="mod-table">
    <thead>
      <th style="width: 1fr">Mod Name</th>
      <th style="width: 3fr">Mod Description</th>
      <th style="width: 1px">
        <Button color="primary">
          <PlusIcon />
          Add
        </Button>
      </th>
    </thead>
    <tr v-for="mod in mods" :key="mod.name">
      <td class="name">
        <Avatar :src="mod.icon"/>
        <span class="text">
          {{ mod.name }}
          <span class="version">
            {{ mod.version }}
          </span>
        </span>
      </td>
      <td>{{ mod.description }}</td>
      <td>
        <div class="manage">
          <Button color="primary" iconOnly>
            <TrashIcon />
          </Button>
          <Checkbox />
        </div>
      </td>
    </tr>
  </table>
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
    name: "Mod Name",
    icon: "https://cdn.modrinth.com/data/P7dR8mSH/icon.png",
    version: "Mod Version",
    author: "Mod Author",
    description: "Mod Description"
  },
  {
    name: "Mod Name",
    icon: "https://cdn.modrinth.com/data/P7dR8mSH/icon.png",
    version: "Mod Version",
    author: "Mod Author",
    description: "Mod Description"
  },
  {
    name: "Mod Name",
    icon: "https://cdn.modrinth.com/data/P7dR8mSH/icon.png",
    version: "Mod Version",
    author: "Mod Author",
    description: "Mod Description"
  },
  {
    name: "Mod Name",
    icon: "https://cdn.modrinth.com/data/P7dR8mSH/icon.png",
    version: "Mod Version",
    author: "Mod Author",
    description: "Mod Description"
  }
]
</script>

<style scoped lang="scss">
.mod-table {
  width: 100%;
  border-collapse: collapse;
  border-spacing: 0;
  border-radius: var(--radius-lg);
  overflow: hidden;
  margin: 1rem;
  table-layout: unset;
  align-items: stretch;

  thead {
    color: var(--color-contrast);
    font-weight: bolder;
    background-color: var(--color-accent-contrast);
  }

  td, th {
    padding: 0.75rem;
  }

  th {
    text-align: left;
  }

  tr {
    &:nth-child(even) {
      background-color: var(--color-raised-bg);
    }
  }
}
.name {
  display: flex;
  align-items: center;
  gap: 1rem;
  color: var(--color-contrast);

  .text {
    gap: .25rem;
    .version {
      color: var(--color-base);
    }
  }
}

.manage {
  gap: 1rem;
  display: flex;
  flex-direction: row;
  justify-content: center;
}
</style>