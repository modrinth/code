<template>
  <div class="relative h-full w-full">
    <div v-if="data" class="flex h-full w-full flex-col gap-6 px-8 py-4">
      <h2 class="m-0 text-3xl font-bold">Startup</h2>
      <div class="rounded-xl border-solid border-rose-500 bg-rose-500/20 p-4">
        These settings are meant for advanced users. Changing these settings may break your server.
      </div>

      <div class="gap-2">
        <div class="card flex flex-col gap-4">
          <label for="username-field" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">Startup Command</span>
            <span> This is the command that starts your server. </span>
          </label>
          <textarea class="min-h-[200px] w-full font-[family-name:var(--mono-font)]" />
        </div>

        <div class="gap-2">
          <div class="card flex flex-col gap-4">
            <label for="username-field" class="flex flex-col gap-2">
              <span class="text-lg font-bold text-contrast">Java Version</span>
              <span> The version of Java that your server will run on. </span>
            </label>
            <DropdownSelect
              name="version"
              :options="['Java 8', 'Java 11', 'Java 16', 'Java 17', 'Java 21', 'Java 22']"
              placeholder="Java Version"
            />
          </div>
        </div>

        <!-- runtime -->
        <div class="gap-2">
          <div class="card flex flex-col gap-4">
            <label for="username-field" class="flex flex-col gap-2">
              <span class="text-lg font-bold text-contrast">Runtime</span>
              <span> The runtime that your server will run on. </span>
            </label>
            <DropdownSelect
              name="runtime"
              :options="['OpenJDK', 'Corretto', 'OpenJ9', 'Adoptium', 'GraalVM']"
              placeholder="Runtime"
            />
          </div>
        </div>
      </div>
    </div>
    <UiServersPyroLoading v-else />
    <div class="absolute bottom-[2.5%] left-[2.5%] z-10 w-[95%]">
      <UiServersSaveBanner
        v-if="hasUnsavedChanges"
        :is-updating="isUpdating"
        :save="saveStartup"
        :reset="resetStartup"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { DropdownSelect } from "@modrinth/ui";
const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const data = computed(() => serverStore.serverData[serverId]);

const isUpdating = ref(false);
const hasUnsavedChanges = ref(false);

const saveStartup = async () => {
  try {
    isUpdating.value = true;
    await new Promise((resolve) => setTimeout(resolve, 500));
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "success",
      title: "Server settings updated",
      text: "Your server settings were successfully changed.",
    });
    await serverStore.fetchServerData(serverId);
  } catch (error) {
    console.error(error);
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "error",
      title: "Failed to update server arguments",
      text: "Please try again later.",
    });
  } finally {
    isUpdating.value = false;
  }
};

const resetStartup = () => {};
</script>
