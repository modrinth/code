<template>
  <NewModal
    ref="modal"
    :header="cf ? `Installing a CurseForge pack` : `Uploading .zip contents from URL`"
  >
    <form class="flex flex-col gap-4 md:w-[600px]" @submit.prevent="handleSubmit">
      <div class="flex flex-col gap-2">
        <div class="font-bold text-contrast">
          {{ cf ? `How to get the modpack version's URL` : "URL of .zip file" }}
        </div>
        <ol v-if="cf" class="mb-1 mt-0 flex flex-col gap-1 pl-8 leading-normal text-secondary">
          <li>
            <a
              href="https://www.curseforge.com/minecraft/search?page=1&pageSize=40&sortBy=relevancy&class=modpacks"
              class="inline-flex font-semibold text-[#F16436] transition-all hover:underline active:brightness-[--hover-brightness]"
              target="_blank"
              rel="noopener noreferrer"
            >
              Find the CurseForge modpack
              <ExternalIcon class="ml-1 inline size-4" stroke-width="3" />
            </a>
            you'd like to install on your server.
          </li>
          <li>
            On the modpack's page, go to the
            <span class="font-semibold text-primary">"Files"</span> tab, and
            <span class="font-semibold text-primary">select the version</span> of the modpack you
            want to install.
          </li>
          <li>
            <span class="font-semibold text-primary">Copy the URL</span> of the version you want to
            install, and paste it in the box below.
          </li>
        </ol>
        <p v-else class="mb-1 mt-0">Copy and paste the direct download URL of a .zip file.</p>
        <input
          ref="urlInput"
          v-model="url"
          autofocus
          :disabled="submitted"
          type="text"
          data-1p-ignore
          data-lpignore="true"
          data-protonpass-ignore="true"
          required
          :placeholder="
            cf
              ? 'https://www.curseforge.com/minecraft/modpacks/.../files/6412259'
              : 'https://www.example.com/.../modpack-name-1.0.2.zip'
          "
          autocomplete="off"
        />
        <div v-if="submitted && error" class="text-red">{{ error }}</div>
      </div>
      <BackupWarning :backup-link="`/servers/manage/${props.server.serverId}/backups`" />
      <div class="flex justify-start gap-2">
        <ButtonStyled color="brand">
          <button v-tooltip="error" :disabled="submitted || !!error" type="submit">
            <SpinnerIcon v-if="submitted" class="animate-spin" />
            <DownloadIcon v-else class="h-5 w-5" />
            {{ submitted ? "Installing..." : "Install" }}
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button type="button" @click="hide">
            <XIcon class="h-5 w-5" />
            {{ submitted ? "Close" : "Cancel" }}
          </button>
        </ButtonStyled>
      </div>
    </form>
  </NewModal>
</template>

<script setup lang="ts">
import { ExternalIcon, SpinnerIcon, DownloadIcon, XIcon } from "@modrinth/assets";
import { BackupWarning, ButtonStyled, NewModal } from "@modrinth/ui";
import { ref, computed, nextTick } from "vue";
import { handleError, type Server } from "~/composables/pyroServers.ts";

const cf = ref(false);

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const modal = ref<typeof NewModal>();
const urlInput = ref<HTMLInputElement | null>(null);
const url = ref("");
const submitted = ref(false);

const trimmedUrl = computed(() => url.value.trim());

const regex = /https:\/\/(www\.)?curseforge\.com\/minecraft\/modpacks\/[^/]+\/files\/\d+/;

const error = computed(() => {
  if (trimmedUrl.value.length === 0) {
    return "URL is required.";
  }
  if (cf.value && !regex.test(trimmedUrl.value)) {
    return "URL must be a CurseForge modpack version URL.";
  } else if (!cf.value && !trimmedUrl.value.includes("/")) {
    return "URL must be valid.";
  }
  return "";
});

const handleSubmit = async () => {
  submitted.value = true;
  if (!error.value) {
    // hide();
    try {
      const dry = await props.server.fs?.extractFile(trimmedUrl.value, true, true);

      if (!cf.value || dry.modpack_name) {
        await props.server.fs?.extractFile(trimmedUrl.value, true, false, true);
        hide();
      } else {
        submitted.value = false;
        handleError(
          new ServersError(
            "Could not find CurseForge modpack at that URL.",
            undefined,
            undefined,
            undefined,
            {
              context: "Error installing modpack",
              error: `url: ${url.value}`,
              description: "Could not find CurseForge modpack at that URL.",
            },
          ),
        );
      }
    } catch (error) {
      submitted.value = false;
      console.error("Error installing:", error);
      handleError(error);
    }
  }
};

const show = (isCf: boolean) => {
  cf.value = isCf;
  url.value = "";
  submitted.value = false;
  modal.value?.show();
  nextTick(() => {
    setTimeout(() => {
      urlInput.value?.focus();
    }, 100);
  });
};

const hide = () => {
  modal.value?.hide();
};

defineExpose({ show, hide });
</script>
