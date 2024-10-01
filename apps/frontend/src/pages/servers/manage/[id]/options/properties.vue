<template>
  <div class="relative h-full w-full">
    <div
      v-if="data && status == 'success'"
      class="flex h-full w-full flex-col justify-between gap-6 overflow-y-auto p-8"
    >
      <h2 class="text-3xl font-bold">server.properties</h2>
      <div
        v-for="(property, index) in liveProperties"
        :key="index"
        class="mb-2 flex items-center justify-between border-x-0 border-b-2 border-t-0 border-solid border-bg-raised pb-2"
      >
        <label :for="index.toString()" class="block">
          {{
            index
              .toString()
              .split("-")
              .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
              .join(" ")
          }}
        </label>
        <div v-if="overrides[index] && overrides[index].type === 'dropdown'">
          <DropdownSelect
            v-model="liveProperties[index]"
            :name="property.id"
            :options="overrides[index].options"
            placeholder="Select..."
          />
        </div>
        <div v-else-if="typeof property === 'boolean'">
          <input
            id="property.id"
            v-model="liveProperties[index]"
            class="switch stylized-toggle"
            type="checkbox"
          />
        </div>
        <div v-else-if="typeof property === 'number'" class="w-[320px]">
          <input
            :id="index.toString()"
            v-model.number="liveProperties[index]"
            type="number"
            class="w-full border p-2"
          />
        </div>
        <div
          v-else-if="
            typeof property === 'object' ||
            property.includes(',') ||
            property.includes('{') ||
            property.includes('}') ||
            property.includes('[') ||
            property.includes(']') ||
            property.length > 30
          "
          class="w-[320px]"
        >
          <textarea
            :id="index.toString()"
            :value="JSON.stringify(property, null, 2)"
            class="w-full rounded-xl border p-2"
          ></textarea>
        </div>
        <div v-else class="w-[320px]">
          <input
            :id="index.toString()"
            :value="property"
            type="text"
            class="w-full rounded-xl border p-2"
          />
        </div>
      </div>
      <div class="mt-10"></div>
    </div>
    <div v-else-if="status === 'error'" class="mt-12 flex w-full items-center justify-center">
      <div class="flex flex-col items-center gap-4">
        <h2 class="text-3xl font-bold">Config not available</h2>
        <div class="text-center text-lg">
          We couldn't find a config, make sure you have started your server at least once. <br />
          If this issue persists, please contact support.
        </div>
      </div>
    </div>
    <UiServersPyroLoading v-else />
    <div class="absolute bottom-[2.5%] left-[2.5%] z-10 w-[95%]">
      <UiServersSaveBanner
        v-if="hasUnsavedChanges"
        :is-updating="isUpdating"
        :save="saveProperties"
        :reset="resetProperties"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { DropdownSelect } from "@modrinth/ui";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const auth = ref<any>(null);
const fetchAuth = async () => {
  try {
    const apiInfo = await serverStore.getFileApiInfo(serverId);
    auth.value = apiInfo;
  } catch (error) {
    console.error("Error fetching file api info:", error);
    auth.value = null;
  }
};

onMounted(async () => {
  await fetchAuth();
});

const isUpdating = ref(false);

const changedPropertiesState = ref({});

const { data, status } = await useAsyncData(
  "data",
  async () => await serverStore.fetchConfigFile(serverId, "ServerProperties"),
);

const overrides: { [key: string]: { type: string; options: string[] } } = {
  difficulty: {
    type: "dropdown",
    options: ["peaceful", "easy", "normal", "hard"],
  },
  gamemode: {
    type: "dropdown",
    options: ["survival", "creative", "adventure", "spectator"],
  },
};

const liveProperties = ref(JSON.parse(JSON.stringify(data.value)));

const hasUnsavedChanges = computed(() => JSON.stringify(changedPropertiesState.value) !== "{}");

watch(
  liveProperties,
  (newProperties) => {
    changedPropertiesState.value = {};
    const changed = [];
    for (const key in newProperties) {
      // @ts-ignore https://typescript.tv/errors/#ts7053
      if (newProperties[key] !== data.value[key]) {
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

const constructServerProperties = (): string => {
  const properties = liveProperties.value;

  let fileContent = `#Minecraft server properties\n#${new Date().toUTCString()}\n`;

  for (const [key, value] of Object.entries(properties)) {
    if (typeof value === "object") {
      fileContent += `${key}=${JSON.stringify(value)}\n`;
    } else if (typeof value === "boolean") {
      fileContent += `${key}=${value ? "true" : "false"}\n`;
    } else {
      fileContent += `${key}=${value}\n`;
    }
  }

  return fileContent;
};

const saveProperties = async () => {
  try {
    isUpdating.value = true;
    await serverStore.updateFile(auth.value, "server.properties", constructServerProperties());
    await new Promise((resolve) => setTimeout(resolve, 500));
    changedPropertiesState.value = {};
    await refreshNuxtData("data");
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "success",
      title: "Server settings updated",
      text: "Your server settings were successfully changed.",
    });
  } catch (error) {
    console.error("Error updating server settings:", error);
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "error",
      title: "Failed to update server settings",
      text: "An error occurred while attempting to update your server settings.",
    });
  } finally {
    isUpdating.value = false;
  }
};

const resetProperties = () => {
  liveProperties.value = JSON.parse(JSON.stringify(data.value));
};
</script>
