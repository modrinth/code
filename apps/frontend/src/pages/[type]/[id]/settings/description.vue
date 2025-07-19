<template>
  <div>
    <div class="universal-card">
      <div class="markdown-disclaimer">
        <h2>Description</h2>
        <span class="label__description">
          You can type an extended description of your mod here.
          <span class="label__subdescription">
            The description must clearly and honestly describe the purpose and function of the
            project. See section 2.1 of the
            <nuxt-link class="text-link" target="_blank" to="/legal/rules">Content Rules</nuxt-link>
            for the full requirements.
          </span>
        </span>
      </div>
      <MarkdownEditor
        v-model="description"
        :disabled="
          !currentMember ||
          (currentMember.permissions & TeamMemberPermission.EDIT_BODY) !==
            TeamMemberPermission.EDIT_BODY
        "
        :on-image-upload="onUploadHandler"
      />
      <div class="input-group markdown-disclaimer">
        <button
          :disabled="!hasChanges"
          class="iconified-button brand-button"
          type="button"
          @click="saveChanges()"
        >
          <SaveIcon />
          Save changes
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { SaveIcon } from "@modrinth/assets";
import { MarkdownEditor } from "@modrinth/ui";
import { type Project, type TeamMember, TeamMemberPermission } from "@modrinth/utils";
import { computed, ref } from "vue";
import { useImageUpload } from "~/composables/image-upload.ts";

const props = defineProps<{
  project: Project;
  allMembers: TeamMember[];
  currentMember: TeamMember | undefined;
  patchProject: (payload: object, quiet?: boolean) => object;
}>();

const description = ref(props.project.body);

const patchRequestPayload = computed(() => {
  const payload: {
    body?: string;
  } = {};

  if (description.value !== props.project.body) {
    payload.body = description.value;
  }

  return payload;
});

const hasChanges = computed(() => {
  return Object.keys(patchRequestPayload.value).length > 0;
});

function saveChanges() {
  props.patchProject(patchRequestPayload.value);
}

async function onUploadHandler(file: File) {
  const response = await useImageUpload(file, {
    context: "project",
    projectID: props.project.id,
  });

  return response.url;
}
</script>

<style scoped>
.markdown-disclaimer {
  margin-block: 1rem;
}
</style>
