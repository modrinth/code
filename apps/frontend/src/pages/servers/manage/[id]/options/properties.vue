<template>
  <div>
    <div v-if="properties && status === 'success'">
      <section class="card">
        <div class="flex flex-col gap-6">
          <h2 class="text-3xl font-bold">General</h2>
          <div class="h-[2px] w-full bg-divider"></div>
          <div v-for="(property, index) in liveProperties" :key="index">
            <div class="mb-4 flex justify-between">
              <label :for="index as unknown as string" class="block text-lg font-semibold">{{
                index
                  .toString()
                  .split("-")
                  .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
                  .join(" ")
              }}</label>
              <div v-if="typeof property === 'boolean'">
                <Checkbox id="property.id" :model-value="property" />
              </div>
              <div v-else-if="typeof property === 'number'">
                <input
                  :id="index as unknown as string"
                  v-model.number="liveProperties[index]"
                  type="number"
                  class="w-full rounded border p-2"
                />
              </div>
              <div v-else-if="typeof property === 'object'">
                <textarea
                  :id="index as unknown as string"
                  :value="JSON.stringify(property, null, 2)"
                  class="w-full rounded border p-2"
                ></textarea>
              </div>
              <div v-else>
                <input
                  :id="index as unknown as string"
                  :value="property"
                  type="text"
                  class="w-full rounded border p-2"
                />
              </div>
            </div>
            <div class="h-[2px] w-full bg-divider"></div>
          </div>
        </div>
        <button type="submit" class="btn btn-primary mt-4" @click="() => saveProperties()">
          Save
        </button>
      </section>
    </div>
    <PyroLoading v-else-if="status === 'pending'" />
    <PyroError
      v-else-if="status === 'error'"
      title="Failed to load"
      message="Failed to load server properties"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import Checkbox from "~/components/ui/Checkbox.vue";
import PyroLoading from "~/components/ui/servers/PyroLoading.vue";
import PyroError from "~/components/ui/servers/PyroError.vue";

const route = useNativeRoute();
const serverId = route.params.id as string;
const auth = await useAuth();

const changedPropertiesState = ref({});

const { data: properties, status } = await useAsyncData("serverProps", async () => {
  const data = await usePyroFetch<string>(
    auth.value.token,
    `servers/${serverId}/config/ServerProperties`,
  );
  return data;
});

const liveProperties = ref(JSON.parse(JSON.stringify(properties.value)));

watch(
  liveProperties,
  (newProperties) => {
    changedPropertiesState.value = {};
    const changed = [];
    for (const key in newProperties) {
      // @ts-ignore https://typescript.tv/errors/#ts7053
      if (newProperties[key] !== properties.value[key]) {
        changed.push(key);
      }
    }
    // @ts-ignore
    for (const key of changed) {
      // @ts-ignore
      changedPropertiesState.value[key] = newProperties[key];
    }
  },
  { deep: true },
);

const saveProperties = async () => {
  await usePyroFetch(
    auth.value.token,
    `servers/${serverId}/config/server`,
    0,
    "PUT",
    "application/json",
    changedPropertiesState.value,
  );
  refreshNuxtData("serverProps");
};
</script>
