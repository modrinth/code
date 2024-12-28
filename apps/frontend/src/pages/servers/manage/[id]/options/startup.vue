<template>
  <div class="relative h-full w-full">
    <div v-if="data" class="flex h-full w-full flex-col gap-4">
      <div class="rounded-2xl border-solid border-orange bg-bg-orange p-4 text-contrast">
        These settings are for advanced users. Changing them can break your server.
      </div>

      <div class="gap-2">
        <div class="card flex flex-col gap-4">
          <div class="flex flex-col justify-between gap-4 sm:flex-row">
            <label for="startup-command-field" class="flex flex-col gap-2">
              <span class="text-lg font-bold text-contrast">Startup command</span>
              <span> The command that runs when your server is started. </span>
            </label>
            <ButtonStyled>
              <button
                :disabled="invocation === startupSettings?.original_invocation"
                class="!w-full sm:!w-auto"
                @click="resetToDefault"
              >
                <UpdatedIcon class="h-5 w-5" />
                Restore default command
              </button>
            </ButtonStyled>
          </div>
          <textarea
            id="startup-command-field"
            v-model="invocation"
            class="min-h-[270px] w-full resize-y font-[family-name:var(--mono-font)]"
          />
        </div>

        <div class="card flex flex-col gap-8">
          <div class="flex flex-col gap-4">
            <div class="flex flex-col gap-2">
              <span class="text-lg font-bold text-contrast">Java version</span>
              <span>
                The version of Java that your server will run on. Your server is running Minecraft
                {{ data.mc_version }}. By default, only the Java versions compatible with this
                version of Minecraft are shown. Some mods or modpacks may require a specific Java
                version.
              </span>
            </div>
            <div class="flex items-center gap-2">
              <input
                id="show-all-versions"
                v-model="showAllVersions"
                class="switch stylized-toggle flex-none"
                type="checkbox"
              />
              <label for="show-all-versions" class="text-sm">Show all Java versions</label>
            </div>
            <UiServersTeleportDropdownMenu
              :id="'java-version-field'"
              v-model="jdkVersion"
              name="java-version"
              :options="displayedJavaVersions"
              placeholder="Java Version"
            />
          </div>
          <div class="flex flex-col gap-4">
            <div class="flex flex-col gap-2">
              <span class="text-lg font-bold text-contrast">Runtime</span>
              <span> The Java runtime your server will use. </span>
            </div>
            <UiServersTeleportDropdownMenu
              :id="'runtime-field'"
              v-model="jdkBuild"
              name="runtime"
              :options="['Corretto', 'Temurin', 'GraalVM']"
              placeholder="Runtime"
            />
          </div>
        </div>
      </div>
    </div>
    <UiServersSaveBanner
      :is-visible="!!hasUnsavedChanges"
      :server="props.server"
      :is-updating="isUpdating"
      :save="saveStartup"
      :reset="resetStartup"
    />
  </div>
</template>

<script setup lang="ts">
import { UpdatedIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const data = computed(() => props.server.general);
const startupSettings = computed(() => props.server.startup);
const showAllVersions = ref(false);

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

const compatibleJavaVersions = computed(() => {
  const mcVersion = data.value?.mc_version ?? "";
  if (!mcVersion) return jdkVersionMap.map((v) => v.label);

  const [major, minor] = mcVersion.split(".").map(Number);

  if (major >= 1) {
    if (minor >= 20) return ["Java 21"];
    if (minor >= 18) return ["Java 17", "Java 21"];
    if (minor >= 17) return ["Java 16", "Java 17", "Java 21"];
    if (minor >= 12) return ["Java 8", "Java 11", "Java 17", "Java 21"];
    if (minor >= 6) return ["Java 8", "Java 11"];
  }

  return ["Java 8"];
});

const displayedJavaVersions = computed(() => {
  return showAllVersions.value ? jdkVersionMap.map((v) => v.label) : compatibleJavaVersions.value;
});

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
    addNotification({
      group: "serverOptions",
      type: "success",
      title: "Server settings updated",
      text: "Your server settings were successfully changed.",
    });
    await props.server.refresh();
  } catch (error) {
    console.error(error);
    addNotification({
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

const resetToDefault = () => {
  invocation.value = startupSettings.value?.original_invocation;
};
</script>

<style scoped>
.stylized-toggle:checked::after {
  background: var(--color-accent-contrast) !important;
}
</style>
