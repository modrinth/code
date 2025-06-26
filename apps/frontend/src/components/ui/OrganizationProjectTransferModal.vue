<template>
  <div>
    <Modal ref="modalOpen" header="Transfer Projects">
      <div class="universal-modal items">
        <div class="table">
          <div class="table-head table-row">
            <div class="check-cell table-cell">
              <Checkbox
                :model-value="selectedProjects.length === props.projects.length"
                @update:model-value="toggleSelectedProjects()"
              />
            </div>
            <div class="table-cell">Icon</div>
            <div class="table-cell">Name</div>
            <div class="table-cell">ID</div>
            <div class="table-cell">Type</div>
            <div class="table-cell" />
          </div>
          <div v-for="project in props.projects" :key="`project-${project.id}`" class="table-row">
            <div class="check-cell table-cell">
              <Checkbox
                :disabled="(project.permissions & EDIT_DETAILS) === EDIT_DETAILS"
                :model-value="selectedProjects.includes(project)"
                @update:model-value="
                  selectedProjects.includes(project)
                    ? (selectedProjects = selectedProjects.filter((it) => it !== project))
                    : selectedProjects.push(project)
                "
              />
            </div>
            <div class="table-cell">
              <nuxt-link tabindex="-1" :to="`/project/${project.slug ? project.slug : project.id}`">
                <Avatar
                  :src="project.icon_url"
                  aria-hidden="true"
                  :alt="'Icon for ' + project.name"
                  no-shadow
                />
              </nuxt-link>
            </div>

            <div class="table-cell">
              <span class="project-title">
                <nuxt-link
                  class="hover-link wrap-as-needed"
                  :to="`/project/${project.slug ? project.slug : project.id}`"
                >
                  {{ project.title }}
                </nuxt-link>
              </span>
            </div>

            <div class="table-cell">
              <CopyCode :text="project.id" />
            </div>

            <div class="table-cell">
              <BoxIcon />
              <span>{{
                formatProjectType(
                  $getProjectTypeForDisplay(
                    project.project_types?.[0] ?? "project",
                    project.loaders,
                  ),
                )
              }}</span>
            </div>

            <div class="table-cell">
              <nuxt-link
                class="btn icon-only"
                :to="`/project/${project.slug ? project.slug : project.id}/settings`"
              >
                <SettingsIcon />
              </nuxt-link>
            </div>
          </div>
        </div>
        <div class="push-right input-group">
          <Button @click="$refs.modalOpen?.hide()">
            <XIcon />
            Cancel
          </Button>
          <Button :disabled="!selectedProjects?.length" color="primary" @click="onSubmitHandler()">
            <TransferIcon />
            <span>
              Transfer
              <span>
                {{
                  selectedProjects.length === props.projects.length
                    ? "All"
                    : selectedProjects.length
                }}
              </span>
              <span>
                {{ " " }}
                {{ selectedProjects.length === 1 ? "project" : "projects" }}
              </span>
            </span>
          </Button>
        </div>
      </div>
    </Modal>
    <Button @click="$refs.modalOpen?.show()">
      <TransferIcon />
      <span>Transfer projects</span>
    </Button>
  </div>
</template>

<script setup>
import { BoxIcon, SettingsIcon, TransferIcon, XIcon } from "@modrinth/assets";
import { Button, Modal, Checkbox, CopyCode, Avatar } from "@modrinth/ui";
import { formatProjectType } from "@modrinth/utils";

const modalOpen = ref(null);

const props = defineProps({
  projects: {
    type: Array,
    required: true,
  },
});

// define emit for submission
const emit = defineEmits(["submit"]);

const selectedProjects = ref([]);

const toggleSelectedProjects = () => {
  if (selectedProjects.value.length === props.projects.length) {
    selectedProjects.value = [];
  } else {
    selectedProjects.value = props.projects;
  }
};

const onSubmitHandler = () => {
  if (selectedProjects.value.length === 0) {
    return;
  }
  emit("submit", selectedProjects.value);
  selectedProjects.value = [];
  modalOpen.value?.hide();
};
</script>

<style lang="scss" scoped>
.table {
  display: grid;
  border-radius: var(--radius-md);
  overflow: hidden;
  margin-top: var(--gap-md);
  border: 1px solid var(--color-button-bg);
  background-color: var(--color-raised-bg);

  .table-row {
    grid-template-columns: 2.75rem 3.75rem 2fr 1fr 1fr 3.5rem;
  }

  .table-cell {
    display: flex;
    align-items: center;
    gap: var(--gap-xs);
    padding: var(--gap-md);
    padding-left: 0;
  }

  .check-cell {
    padding-left: var(--gap-md);
  }

  @media screen and (max-width: 750px) {
    display: flex;
    flex-direction: column;

    .table-row {
      display: grid;
      grid-template: "checkbox icon name type settings" "checkbox icon id type settings";
      grid-template-columns:
        min-content min-content minmax(min-content, 2fr)
        minmax(min-content, 1fr) min-content;

      :nth-child(1) {
        grid-area: checkbox;
      }

      :nth-child(2) {
        grid-area: icon;
      }

      :nth-child(3) {
        grid-area: name;
      }

      :nth-child(4) {
        grid-area: id;
        padding-top: 0;
      }

      :nth-child(5) {
        grid-area: type;
      }

      :nth-child(6) {
        grid-area: settings;
      }
    }

    .table-head {
      grid-template: "checkbox settings";
      grid-template-columns: min-content minmax(min-content, 1fr);

      :nth-child(2),
      :nth-child(3),
      :nth-child(4),
      :nth-child(5) {
        display: none;
      }
    }
  }

  @media screen and (max-width: 560px) {
    .table-row {
      display: grid;
      grid-template: "checkbox icon name settings" "checkbox icon id settings" "checkbox icon type settings" "checkbox icon status settings";
      grid-template-columns: min-content min-content minmax(min-content, 1fr) min-content;

      :nth-child(5) {
        padding-top: 0;
      }
    }

    .table-head {
      grid-template: "checkbox settings";
      grid-template-columns: min-content minmax(min-content, 1fr);
    }
  }
}

.items {
  display: flex;
  flex-direction: column;
  gap: var(--gap-md);
  padding: var(--gap-md);
}
</style>
