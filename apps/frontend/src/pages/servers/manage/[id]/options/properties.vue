<template>
  <div class="relative h-full w-full select-none overflow-y-auto">
    <div v-if="server.fs?.error" class="flex w-full flex-col items-center justify-center gap-4 p-4">
      <div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
        <div class="flex flex-col items-center text-center">
          <div class="flex flex-col items-center gap-4">
            <div class="grid place-content-center rounded-full bg-bg-orange p-4">
              <IssuesIcon class="size-12 text-orange" />
            </div>
            <h1 class="m-0 mb-2 w-fit text-4xl font-bold">Failed to load properties</h1>
          </div>
          <p class="text-lg text-secondary">
            We couldn't access your server's properties. Here's what we know:
            <span class="break-all font-mono">{{ JSON.stringify(server.fs.error) }}</span>
          </p>
          <ButtonStyled size="large" color="brand" @click="() => server.refresh(['fs'])">
            <button class="mt-6 !w-full">Retry</button>
          </ButtonStyled>
        </div>
      </div>
    </div>

    <div
      v-else-if="propsData && status === 'success'"
      class="flex h-full w-full flex-col justify-between gap-6 overflow-y-auto"
    >
      <div class="card flex flex-col gap-4">
        <div class="flex flex-col gap-2">
          <h2 class="m-0 text-lg font-bold text-contrast">Server properties</h2>
          <div class="m-0">
            Edit the Minecraft server properties file. If you're unsure about a specific property,
            the
            <NuxtLink
              class="goto-link !inline-block"
              to="https://minecraft.wiki/w/Server.properties"
              external
            >
              Minecraft Wiki
            </NuxtLink>
            has more detailed information.
          </div>
        </div>
        <div class="flex flex-col gap-4 rounded-2xl bg-table-alternateRow p-4">
          <div class="relative w-full text-sm">
            <label for="search-server-properties" class="sr-only">Search server properties</label>
            <SearchIcon
              class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2"
              aria-hidden="true"
            />
            <input
              id="search-server-properties"
              v-model="searchInput"
              class="w-full pl-9"
              type="search"
              name="search"
              autocomplete="off"
              placeholder="Search server properties..."
            />
          </div>
          <div
            v-for="(property, index) in filteredProperties"
            :key="index"
            class="flex flex-row flex-wrap items-center justify-between py-2"
          >
            <div class="flex items-center">
              <span :id="`property-label-${index}`">{{ formatPropertyName(index) }}</span>
              <span v-if="overrides[index] && overrides[index].info" class="ml-2">
                <EyeIcon v-tooltip="overrides[index].info" />
              </span>
            </div>
            <div
              v-if="overrides[index] && overrides[index].type === 'dropdown'"
              class="mt-2 flex w-full sm:w-[320px] sm:justify-end"
            >
              <UiServersTeleportDropdownMenu
                :id="`server-property-${index}`"
                v-model="liveProperties[index]"
                :name="formatPropertyName(index)"
                :options="overrides[index].options || []"
                :aria-labelledby="`property-label-${index}`"
                placeholder="Select..."
              />
            </div>
            <div v-else-if="typeof property === 'boolean'" class="flex justify-end">
              <input
                :id="`server-property-${index}`"
                v-model="liveProperties[index]"
                class="switch stylized-toggle"
                type="checkbox"
                :aria-labelledby="`property-label-${index}`"
              />
            </div>
            <div v-else-if="typeof property === 'number'" class="mt-2 w-full sm:w-[320px]">
              <input
                :id="`server-property-${index}`"
                v-model.number="liveProperties[index]"
                type="number"
                class="w-full border p-2"
                :aria-labelledby="`property-label-${index}`"
              />
            </div>
            <div v-else-if="isComplexProperty(property)" class="mt-2 w-full sm:w-[320px]">
              <textarea
                :id="`server-property-${index}`"
                v-model="liveProperties[index]"
                class="w-full resize-y rounded-xl border p-2"
                :aria-labelledby="`property-label-${index}`"
              ></textarea>
            </div>
            <div v-else class="mt-2 flex w-full justify-end sm:w-[320px]">
              <input
                :id="`server-property-${index}`"
                v-model="liveProperties[index]"
                type="text"
                class="w-full rounded-xl border p-2"
                :aria-labelledby="`property-label-${index}`"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-else class="card flex h-full w-full items-center justify-center">
      <p class="text-contrast">
        The server properties file has not been generated yet. Start up your server to generate it.
      </p>
    </div>

    <UiServersSaveBanner
      :is-visible="hasUnsavedChanges"
      :server="props.server"
      :is-updating="isUpdating"
      restart
      :save="saveProperties"
      :reset="resetProperties"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed, inject } from "vue";
import { EyeIcon, SearchIcon, IssuesIcon } from "@modrinth/assets";
import Fuse from "fuse.js";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const tags = useTags();

const isUpdating = ref(false);

const searchInput = ref("");

const data = computed(() => props.server.general);
const modulesLoaded = inject<Promise<void>>("modulesLoaded");
const { data: propsData, status } = await useAsyncData("ServerProperties", async () => {
  await modulesLoaded;
  const rawProps = await props.server.fs?.downloadFile("server.properties");
  if (!rawProps) return null;

  const properties: Record<string, any> = {};
  const lines = rawProps.split("\n");

  for (const line of lines) {
    if (line.startsWith("#") || !line.includes("=")) continue;
    const [key, ...valueParts] = line.split("=");
    let value = valueParts.join("=");

    if (value.toLowerCase() === "true" || value.toLowerCase() === "false") {
      value = value.toLowerCase() === "true";
    } else if (!isNaN(value as any) && value !== "") {
      value = Number(value);
    }

    properties[key.trim()] = value;
  }

  return properties;
});

const liveProperties = ref<Record<string, any>>({});
const originalProperties = ref<Record<string, any>>({});

watch(
  propsData,
  (newPropsData) => {
    if (newPropsData) {
      liveProperties.value = JSON.parse(JSON.stringify(newPropsData));
      originalProperties.value = JSON.parse(JSON.stringify(newPropsData));
    }
  },
  { immediate: true },
);

const hasUnsavedChanges = computed(() => {
  return Object.keys(liveProperties.value).some(
    (key) =>
      JSON.stringify(liveProperties.value[key]) !== JSON.stringify(originalProperties.value[key]),
  );
});

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

const fuse = computed(() => {
  if (!liveProperties.value) return null;

  const propertiesToFuse = Object.entries(liveProperties.value).map(([key, value]) => ({
    key,
    value: String(value),
  }));

  return new Fuse(propertiesToFuse, {
    keys: ["key", "value"],
    threshold: 0.2,
  });
});

const filteredProperties = computed(() => {
  if (!searchInput.value?.trim()) {
    return liveProperties.value;
  }

  const results = fuse.value?.search(searchInput.value) ?? [];

  return Object.fromEntries(results.map(({ item }) => [item.key, liveProperties.value[item.key]]));
});

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
    originalProperties.value = JSON.parse(JSON.stringify(liveProperties.value));
    await props.server.refresh();
    addNotification({
      group: "serverOptions",
      type: "success",
      title: "Server properties updated",
      text: "Your server properties were successfully changed.",
    });
  } catch (error) {
    console.error("Error updating server properties:", error);
    addNotification({
      group: "serverOptions",
      type: "error",
      title: "Failed to update server properties",
      text: "An error occurred while attempting to update your server properties.",
    });
  } finally {
    isUpdating.value = false;
  }
};

const resetProperties = async () => {
  liveProperties.value = JSON.parse(JSON.stringify(originalProperties.value));
  await new Promise((resolve) => setTimeout(resolve, 200));
};

const formatPropertyName = (propertyName: string): string => {
  return propertyName
    .split(/[-.]/)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
};

const isComplexProperty = (property: any): boolean => {
  return (
    typeof property === "object" ||
    (typeof property === "string" &&
      (property.includes(",") ||
        property.includes("{") ||
        property.includes("}") ||
        property.includes("[") ||
        property.includes("]") ||
        property.length > 30))
  );
};
</script>

<style scoped>
.stylized-toggle:checked::after {
  background: var(--color-accent-contrast) !important;
}
</style>
