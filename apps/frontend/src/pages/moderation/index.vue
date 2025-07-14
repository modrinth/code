<template>
  <div class="flex flex-row justify-between gap-3">
    <div class="iconified-input">
      <SearchIcon aria-hidden="true" class="text-lg" />
      <input
        v-model="query"
        class="h-[40px]"
        autocomplete="off"
        spellcheck="false"
        type="text"
        :placeholder="`Search...`"
        @input="updateSearchResults()"
      />
      <Button v-if="query" class="r-btn" @click="() => (query = '')">
        <XIcon />
      </Button>
    </div>
    <div class="flex flex-row justify-end gap-2">
      <DropdownSelect
        v-slot="{ selected }"
        v-model="currentFilterType"
        class="!w-[250px] flex-grow md:flex-grow-0"
        name="Filter by"
        :options="filterTypes as unknown[]"
        @change="updateSearchResults()"
      >
        <span class="flex flex-row gap-2 align-middle font-semibold text-secondary"
          ><FilterIcon class="size-4" />{{ selected }}</span
        >
      </DropdownSelect>
      <DropdownSelect
        v-slot="{ selected }"
        v-model="currentSortType"
        class="!w-[150px] flex-grow md:flex-grow-0"
        name="Sort by"
        :options="sortTypes as unknown[]"
        @change="updateSearchResults()"
      >
        <span class="flex flex-row gap-2 align-middle font-semibold text-secondary"
          ><SortAscIcon class="size-4" v-if="selected === 'Oldest'" />
          <SortDescIcon class="size-4" v-else />{{ selected }}</span
        >
      </DropdownSelect>
      <ButtonStyled color="orange">
        <button class="!h-[40px]"><ScaleIcon class="size-4" /> Moderate</button>
      </ButtonStyled>
    </div>
  </div>
</template>
<script setup lang="ts">
import { DropdownSelect, Button, ButtonStyled } from "@modrinth/ui";
import {
  XIcon,
  SearchIcon,
  SortAscIcon,
  SortDescIcon,
  FilterIcon,
  ScaleIcon,
} from "@modrinth/assets";
import { useLocalStorage } from "@vueuse/core";

const query = useLocalStorage("moderation-query", "");
const currentFilterType = useLocalStorage("moderation-current-filter-type", () => "All");
const filterTypes: readonly string[] = readonly([
  "All projects",
  "Modpacks",
  "Mods",
  "Resource Packs",
  "Data Packs",
  "Plugins",
  "Shaders",
]);

const currentSortType = useLocalStorage("moderation-current-sort-type", () => "Oldest");
const sortTypes: readonly string[] = readonly(["Oldest", "Newest"]);

function updateSearchResults() {}
</script>
