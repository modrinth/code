<template>
  <div class="relative h-full w-full">
    <div v-if="data" class="flex h-full w-full flex-col gap-4 px-4">
      <div class="rounded-xl border-solid border-rose-500 bg-rose-500/20 p-4">
        These settings are meant for advanced users. Changing these settings may break your server.
      </div>

      <div class="gap-2">
        <div class="card flex flex-col gap-4">
          <label for="username-field" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">Startup Command</span>
            <span> This is the command that starts your server. </span>
          </label>
          <textarea
            v-model="invocation"
            class="min-h-[200px] w-full font-[family-name:var(--mono-font)]"
          />
        </div>

        <div class="card flex justify-between gap-2">
          <div class="flex flex-col gap-4">
            <label for="username-field" class="flex flex-col gap-2">
              <span class="text-lg font-bold text-contrast">Java Version</span>
              <span> The version of Java that your server will run on. </span>
            </label>
            <DropdownSelect
              v-model="jdkVersion"
              name="version"
              :options="['Java 21', 'Java 17', 'Java 11', 'Java 8']"
              placeholder="Java Version"
            />
          </div>
          <div class="flex flex-col gap-4">
            <label for="username-field" class="flex flex-col gap-2">
              <span class="text-lg font-bold text-contrast">Runtime</span>
              <span> The runtime that your server will run on. </span>
            </label>
            <DropdownSelect
              v-model="jdkBuild"
              name="runtime"
              :options="['Corretto', 'Temurin', 'GraalVM']"
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
        :server="props.server"
        :is-updating="isUpdating"
        :save="saveStartup"
        :reset="resetStartup"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { DropdownSelect } from "@modrinth/ui";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

const app = useNuxtApp();

const data = computed(() => props.server.general);

const startupSettings = computed(() => props.server.startup);

const jdkVersionMap = [
  { value: "lts8", label: "Java 8" },
  { value: "lts11", label: "Java 11" },
  { value: "lts17", label: "Java 17" },
  { value: "lts21", label: "Java 21" },
];

const jdkBuildMap = [
  { value: "corretto", label: "Corretto" },
  { value: "temurin", label: "Temurin" },
  { value: "graal", label: "GraalVM" },
];

const invocation = ref(startupSettings.value?.invocation);
const jdkVersion = ref(
  jdkVersionMap.find((v) => v.value === startupSettings.value?.jdk_version)?.label || "",
);
const jdkBuild = ref(
  jdkBuildMap.find((v) => v.value === startupSettings.value?.jdk_build)?.label || "",
);
const isUpdating = ref(false);
const hasUnsavedChanges = computed(
  () =>
    invocation.value !== startupSettings.value?.invocation ||
    jdkVersion.value !==
      (jdkVersionMap.find((v) => v.value === startupSettings.value?.jdk_version)?.label || "") ||
    jdkBuild.value !==
      jdkBuildMap.find((v) => v.value === startupSettings.value?.jdk_build || "")?.label,
);

const saveStartup = async () => {
  try {
    isUpdating.value = true;
    const invocationValue = invocation.value ?? "";
    const jdkVersionKey = jdkVersionMap.find((v) => v.label === jdkVersion.value)?.value;
    const jdkBuildKey = jdkBuildMap.find((v) => v.label === jdkBuild.value)?.value;
    await props.server.startup?.update(invocationValue, jdkVersionKey as any, jdkBuildKey as any);
    await new Promise((resolve) => setTimeout(resolve, 500));
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "success",
      title: "Server settings updated",
      text: "Your server settings were successfully changed.",
    });
    await props.server.refresh();
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

const resetStartup = () => {
  invocation.value = startupSettings.value?.invocation;
  jdkVersion.value =
    jdkVersionMap.find((v) => v.value === startupSettings.value?.jdk_version)?.label || "";
  jdkBuild.value =
    jdkBuildMap.find((v) => v.value === startupSettings.value?.jdk_build)?.label || "";
};
</script>
