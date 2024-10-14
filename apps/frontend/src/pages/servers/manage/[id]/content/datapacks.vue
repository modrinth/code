<template>
  <div class="flex h-full w-full flex-col">
    <div class="card flex items-center justify-between gap-2 px-3">
      <h2 class="m-0 text-2xl font-bold text-contrast">Datapacks</h2>
      <ButtonStyled color="brand">
        <button v-tooltip="'Add a mod'">
          <PlusIcon />
          Add Datapack
        </button>
      </ButtonStyled>
    </div>
    soon tm
  </div>
</template>

<script setup lang="ts">
import { PlusIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";
</script>
