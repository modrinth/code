<template>
  <Card class="mod-card">
    <div class="card-row">
      <div class="iconified-input">
        <SearchIcon />
        <input v-model="searchFilter" type="text" placeholder="Search Mods" />
      </div>
      <span class="manage">
        <span class="text-combo">
          Sort By
          <DropdownSelect
            v-model="sortFilter"
            :options="['Name', 'Version', 'Author']"
            default-value="Name"
            class="dropdown"
          />
        </span>
        <Button color="primary">
          <PlusIcon />
          Add Mods
        </Button>
      </span>
    </div>
    <div class="table-container">
      <div class="table-row table-head">
        <div class="table-cell table-text">
          <Button color="success" icon-only>
            <UpdatedIcon />
          </Button>
        </div>
        <div class="table-cell table-text name-cell">Name</div>
        <div class="table-cell table-text">Version</div>
        <div class="table-cell table-text">Author</div>
        <div class="table-cell table-text">Actions</div>
      </div>
      <div v-for="mod in search" :key="mod.name" class="table-row">
        <div class="table-cell table-text">
          <Button v-if="mod.outdated" icon-only>
            <UpdatedIcon />
          </Button>
          <Button v-else disabled icon-only>
            <CheckCircleIcon />
          </Button>
        </div>
        <div class="table-cell table-text name-cell">
          <span class="mod-text">
            <Avatar :src="mod.icon" />
            {{ mod.name }}
          </span>
        </div>
        <div class="table-cell table-text">{{ mod.version }}</div>
        <div class="table-cell table-text">{{ mod.author }}</div>
        <div class="table-cell table-text manage">
          <Button icon-only>
            <TrashIcon />
          </Button>
          <input id="switch-1" type="checkbox" class="switch stylized-toggle" checked />
        </div>
      </div>
    </div>
  </Card>
</template>

<script>
export default {
  name: 'Mods',
  data() {
    return {
      searchFilter: '',
      sortFilter: '',
      mods: [
        {
          name: 'Fabric API',
          icon: 'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',
          version: '0.76.0+1.19.4',
          author: 'modmuss50',
          description:
            'Lightweight and modular API providing common hooks and intercompatibility measures utilized by mods using the Fabric toolchain.',
          outdated: true,
        },
        {
          name: 'Spirit',
          icon: 'https://cdn.modrinth.com/data/b1LdOZlE/465598dc5d89f67fb8f8de6def21240fa35e3a54.png',
          version: '2.2.4',
          author: 'CodexAdrian',
          description: 'Create your own configurable mob spawner!',
          outdated: true,
        },
        {
          name: 'Botarium',
          icon: 'https://cdn.modrinth.com/data/2u6LRnMa/98b286b0d541ad4f9409e0af3df82ad09403f179.gif',
          version: '2.0.5',
          author: 'CodexAdrian',
          description:
            'A crossplatform API for devs that makes transfer and storage of items, fluids and energy easier, as well as some other helpful things',
          outdated: true,
        },
        {
          name: 'Tempad',
          icon: 'https://cdn.modrinth.com/data/gKNwt7xu/icon.gif',
          version: '2.2.4',
          author: 'CodexAdrian',
          description: 'Create a portal to anywhere from anywhere',
          outdated: false,
        },
        {
          name: 'Sodium',
          icon: 'https://cdn.modrinth.com/data/AANobbMI/icon.png',
          version: '0.4.10',
          author: 'jellysquid3',
          description: 'Modern rendering engine and client-side optimization mod for Minecraft',
          outdated: false,
        },
      ],
    }
  },
  computed: {
    search() {
      const filtered = this.mods.filter((mod) => {
        return mod.name.toLowerCase().includes(this.searchFilter.toLowerCase())
      })

      return this.updateSort(filtered, this.sortFilter)
    },
  },
  methods: {
    updateSort(projects, sort) {
      switch (sort) {
        case 'Version':
          return projects.slice().sort((a, b) => {
            if (a.version < b.version) {
              return -1
            }
            if (a.version > b.version) {
              return 1
            }
            return 0
          })
        case 'Author':
          return projects.slice().sort((a, b) => {
            if (a.author < b.author) {
              return -1
            }
            if (a.author > b.author) {
              return 1
            }
            return 0
          })
        default:
          return projects.slice().sort((a, b) => {
            if (a.name < b.name) {
              return -1
            }
            if (a.name > b.name) {
              return 1
            }
            return 0
          })
      }
    },
  },
}
</script>
<script setup>
import {
  Avatar,
  Button,
  TrashIcon,
  PlusIcon,
  Card,
  CheckCircleIcon,
  SearchIcon,
  UpdatedIcon,
  DropdownSelect,
} from 'omorphia'
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
  grid-template-columns: min-content 2fr 1fr 1fr 8rem;
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
  gap: 0.5rem;
}

.mod-text {
  display: flex;
  align-items: center;
  gap: 1rem;
  color: var(--color-contrast);
}

.table-row:nth-child(even) .table-cell {
  background-color: var(--color-bg);
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
}

.text-combo {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.name-cell {
  padding-left: 0;
}

.dropdown {
  width: 7rem !important;
}
</style>
