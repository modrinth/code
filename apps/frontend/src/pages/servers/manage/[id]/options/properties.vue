<template>
  <section class="card">
    <h2 class="text-3xl font-bold">{{ formatMessage(messages.title) }}</h2>
    <div
      v-for="(property, index) in properties"
      :key="index"
      class="mb-4 flex justify-between gap-4"
    >
      <label :for="property.id" class="block text-lg font-semibold">{{ property.name }}</label>
      <div v-if="property.type === 'boolean'">
        <Checkbox id="property.id" :model-value="property.value" />
      </div>
      <div v-else-if="property.type === 'number'">
        <input
          :id="property.id"
          v-model="property.value"
          type="number"
          class="w-full rounded border p-2"
        />
      </div>
      <div v-else>
        <input
          :id="property.id"
          v-model="property.value"
          type="text"
          class="w-full rounded border p-2"
        />
      </div>
    </div>
    <button type="submit" class="btn btn-primary">Save</button>
  </section>
</template>

<script setup lang="ts">
import Checkbox from "~/components/ui/Checkbox.vue";

const { formatMessage } = useVIntl();
const messages = defineMessages({
  title: {
    id: "server.options.props.title",
    defaultMessage: "Server Properties",
  },
});

const properties = ref([
  { id: "spawn-protection", name: "Spawn Protection", type: "number", value: 4 },
  { id: "max-tick-time", name: "Max Tick Time", type: "number", value: 4 },
  { id: "query.port", name: "Query Port", type: "number", value: 25566 },
  { id: "gamemode", name: "Gamemode", type: "string", value: "survival" },
  { id: "sync-chunk-writes", name: "Sync Chunk Writes", type: "boolean", value: true },
  { id: "force-gamemode", name: "Force Gamemode", type: "boolean", value: false },
]);
</script>
