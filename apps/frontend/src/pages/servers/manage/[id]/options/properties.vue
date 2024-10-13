<template>
  <div class="relative h-full w-full overflow-y-auto">
    <div
      v-if="propsData"
      class="flex h-full w-full flex-col justify-between gap-6 overflow-y-auto px-4"
    >
      <div class="card flex flex-col gap-4">
        <label for="username-field" class="flex flex-col gap-2">
          <span class="text-lg font-bold text-contrast">Server Properties</span>
          <span> Edit the minecraft server properties file. </span>
        </label>
        <div class="flex flex-col gap-4 rounded-xl bg-table-alternateRow p-4">
          <div
            v-for="(property, index) in liveProperties"
            :key="index"
            class="mb-2 flex items-center justify-between pb-2"
          >
            <label :for="index.toString()" class="flex items-center">
              {{
                index
                  .toString()
                  .split("-")
                  .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
                  .join(" ")
              }}
              <span v-if="overrides[index] && overrides[index].info" class="ml-2">
                <EyeIcon v-tooltip="overrides[index].info" />
              </span>
            </label>
            <div v-if="overrides[index] && overrides[index].type === 'dropdown'">
              <DropdownSelect
                v-model="liveProperties[index]"
                :name="
                  index
                    .toString()
                    .split('-')
                    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
                    .join(' ')
                "
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
        </div>
      </div>
    </div>
    <UiServersPyroLoading v-else />
    <div class="absolute bottom-[2.5%] left-[2.5%] z-10 w-[95%]">
      <UiServersSaveBanner
        v-if="hasUnsavedChanges"
        :is-updating="isUpdating"
        restart
        :save="saveProperties"
        :reset="resetProperties"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { DropdownSelect } from "@modrinth/ui";
import { EyeIcon } from "@modrinth/assets";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

const tags = useTags();

const isUpdating = ref(false);

const changedPropertiesState = ref({});
const hasUnsavedChanges = computed(() => JSON.stringify(changedPropertiesState.value) !== "{}");

const data = computed(() => props.server.general);
const { data: propsData } = await useAsyncData(
  "ServerProperties",
  async () => await props.server.general?.fetchConfigFile("ServerProperties"),
);

const getDifficultyOptions = () => {
  const pre113Versions = tags.value.gameVersions
    .filter((v) => {
      const versionNumbers = v.version.split(".").map(Number);
      return versionNumbers[0] === 1 && versionNumbers[1] < 13;
    })
    .map((v) => v.version);
  if (data.value?.mc_version && pre113Versions.includes(data.value.mc_version)) {
    return ["0", "1", "2", "3"];
  } else {
    return ["peaceful", "easy", "normal", "hard"];
  }
};

const overrides: { [key: string]: { type: string; options?: string[]; info?: string } } = {
  difficulty: {
    type: "dropdown",
    options: getDifficultyOptions(),
  },
  gamemode: {
    type: "dropdown",
    options: ["survival", "creative", "adventure", "spectator"],
  },
};

const liveProperties = ref(JSON.parse(JSON.stringify(propsData.value)));

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
    await props.server.fs?.updateFile("server.properties", constructServerProperties());
    await new Promise((resolve) => setTimeout(resolve, 500));
    changedPropertiesState.value = {};
    await props.server.refresh();
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

const resetProperties = async () => {
  liveProperties.value = JSON.parse(JSON.stringify(propsData.value));
  await new Promise((resolve) => setTimeout(resolve, 200));
  changedPropertiesState.value = {};
};
</script>
