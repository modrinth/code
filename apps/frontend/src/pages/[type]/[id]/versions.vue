<template>
  <ConfirmModal
    v-if="currentMember"
    ref="deleteVersionModal"
    :has-to-type="false"
    description="This will remove this version forever (like really forever)."
    proceed-label="Delete"
    title="Are you sure you want to delete this version?"
    @proceed="deleteVersion()"
  />
  <section class="experimental-styles-within overflow-visible">
    <div
      v-if="currentMember && isPermission(currentMember?.permissions, 1 << 0)"
      class="card flex items-center gap-4"
    >
      <FileInput
        :accept="acceptFileFromProjectType(project.project_type)"
        :callback="handleFiles"
        :max-size="524288000"
        aria-label="Upload a version"
        class="btn btn-primary"
        prompt="Upload a version"
      />
      <span class="flex items-center gap-2">
        <InfoIcon aria-hidden="true" /> Click to choose a file or drag one onto this page
      </span>
      <DropArea :accept="acceptFileFromProjectType(project.project_type)" @change="handleFiles" />
    </div>
    <ProjectPageVersions
      :base-id="baseDropdownId"
      :current-member="!!currentMember"
      :game-versions="tags.gameVersions"
      :loaders="tags.loaders"
      :project="project"
      :show-files="flags.showVersionFilesInTable"
      :version-link="
        (version) =>
          `/${project.project_type}/${
            project.slug ? project.slug : project.id
          }/version/${encodeURI(version.displayUrlEnding)}`
      "
      :versions="versions"
    >
      <template #actions="{ version }">
        <ButtonStyled circular type="transparent">
          <a
            v-tooltip="`Download`"
            :href="getPrimaryFile(version).url"
            aria-label="Download"
            class="group-hover:!bg-brand group-hover:[&>svg]:!text-brand-inverted"
            @click="emit('onDownload')"
          >
            <DownloadIcon aria-hidden="true" />
          </a>
        </ButtonStyled>
        <ButtonStyled circular type="transparent">
          <OverflowMenu
            :dropdown-id="`${baseDropdownId}-${version.id}`"
            :options="[
              {
                id: 'download',
                color: 'primary',
                hoverFilled: true,
                link: getPrimaryFile(version).url,
                action: () => {
                  emit('onDownload');
                },
              },
              {
                id: 'new-tab',
                action: () => {},
                link: `/${project.project_type}/${
                  project.slug ? project.slug : project.id
                }/version/${encodeURI(version.displayUrlEnding)}`,
                external: true,
              },
              {
                id: 'copy-link',
                action: () =>
                  copyToClipboard(
                    `https://modrinth.com/${project.project_type}/${
                      project.slug ? project.slug : project.id
                    }/version/${encodeURI(version.displayUrlEnding)}`,
                  ),
              },
              {
                id: 'share',
                action: () => {},
                shown: false,
              },
              {
                id: 'report',
                color: 'red',
                hoverFilled: true,
                action: () => (auth.user ? reportVersion(version.id) : navigateTo('/auth/sign-in')),
                shown: !currentMember,
              },
              { divider: true, shown: currentMember || flags.developerMode },
              {
                id: 'copy-id',
                action: () => {
                  copyToClipboard(version.id);
                },
                shown: currentMember || flags.developerMode,
              },
              { divider: true, shown: currentMember },
              {
                id: 'edit',
                link: `/${project.project_type}/${
                  project.slug ? project.slug : project.id
                }/version/${encodeURI(version.displayUrlEnding)}/edit`,
                shown: currentMember,
              },
              {
                id: 'delete',
                color: 'red',
                hoverFilled: true,
                action: () => {
                  selectedVersion = version.id;
                  deleteVersionModal.show();
                },
                shown: currentMember,
              },
            ]"
            aria-label="More options"
            class="group-hover:!bg-button-bg"
          >
            <MoreVerticalIcon aria-hidden="true" />
            <template #download>
              <DownloadIcon aria-hidden="true" />
              Download
            </template>
            <template #new-tab>
              <ExternalIcon aria-hidden="true" />
              Open in new tab
            </template>
            <template #copy-link>
              <LinkIcon aria-hidden="true" />
              Copy link
            </template>
            <template #share>
              <ShareIcon aria-hidden="true" />
              Share
            </template>
            <template #report>
              <ReportIcon aria-hidden="true" />
              Report
            </template>
            <template #edit>
              <EditIcon aria-hidden="true" />
              Edit
            </template>
            <template #delete>
              <TrashIcon aria-hidden="true" />
              Delete
            </template>
            <template #copy-id>
              <ClipboardCopyIcon aria-hidden="true" />
              Copy ID
            </template>
          </OverflowMenu>
        </ButtonStyled>
      </template>
    </ProjectPageVersions>
  </section>
</template>

<script setup>
import {
  ButtonStyled,
  ConfirmModal,
  FileInput,
  OverflowMenu,
  ProjectPageVersions,
} from "@modrinth/ui";
import {
  ClipboardCopyIcon,
  DownloadIcon,
  EditIcon,
  ExternalIcon,
  InfoIcon,
  LinkIcon,
  MoreVerticalIcon,
  ReportIcon,
  ShareIcon,
  TrashIcon,
} from "@modrinth/assets";
import DropArea from "~/components/ui/DropArea.vue";
import { acceptFileFromProjectType } from "~/helpers/fileUtils.js";

const props = defineProps({
  project: {
    type: Object,
    default() {
      return {};
    },
  },
  versions: {
    type: Array,
    default() {
      return [];
    },
  },
  currentMember: {
    type: Object,
    default() {
      return null;
    },
  },
});

const tags = useTags();
const flags = useFeatureFlags();
const auth = await useAuth();

const deleteVersionModal = ref();
const selectedVersion = ref(null);

const emit = defineEmits(["onDownload", "deleteVersion"]);

const router = useNativeRouter();

const baseDropdownId = useId();

function getPrimaryFile(version) {
  return version.files.find((x) => x.primary) || version.files[0];
}

async function handleFiles(files) {
  await router.push({
    name: "type-id-version-version",
    params: {
      type: props.project.project_type,
      id: props.project.slug ? props.project.slug : props.project.id,
      version: "create",
    },
    state: {
      newPrimaryFile: files[0],
    },
  });
}

async function copyToClipboard(text) {
  await navigator.clipboard.writeText(text);
}

function deleteVersion() {
  emit("deleteVersion", selectedVersion.value);
  selectedVersion.value = null;
}
</script>
