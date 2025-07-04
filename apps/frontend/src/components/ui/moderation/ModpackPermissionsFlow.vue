<template>
  <div>
    <h2 v-if="modPackData" class="m-0 mb-2 text-lg font-extrabold">
      Modpack permissions ({{ Math.min(modPackData.length, currentIndex + 1) }} /
      {{ modPackData.length }})
    </h2>

    <div v-if="!modPackData">Loading data...</div>

    <div v-else-if="modPackData.length === 0">
      <p>All permissions obtained. You may skip this step!</p>
    </div>

    <div v-else-if="!modPackData[currentIndex]">
      <p>All permission checks complete!</p>
    </div>

    <div v-else>
      <div v-if="modPackData[currentIndex].type === 'unknown'">
        <p>What is the approval type of {{ modPackData[currentIndex].file_name }}?</p>
        <div class="input-group">
          <ButtonStyled
            v-for="(option, index) in fileApprovalTypes"
            :key="index"
            :color="modPackData[currentIndex].status === option.id ? 'brand' : 'standard'"
            @click="setStatus(currentIndex, option.id)"
          >
            <button>
              {{ option.name }}
            </button>
          </ButtonStyled>
        </div>
        <div v-if="modPackData[currentIndex].status !== 'unidentified'" class="flex flex-col gap-1">
          <label for="proof">
            <span class="label__title">Proof</span>
          </label>
          <input
            id="proof"
            v-model="(modPackData[currentIndex] as UnknownModpackItem).proof"
            type="text"
            autocomplete="off"
            placeholder="Enter proof of status..."
            @input="persistAll()"
          />
          <label for="link">
            <span class="label__title">Link</span>
          </label>
          <input
            id="link"
            v-model="(modPackData[currentIndex] as UnknownModpackItem).url"
            type="text"
            autocomplete="off"
            placeholder="Enter link of project..."
            @input="persistAll()"
          />
          <label for="title">
            <span class="label__title">Title</span>
          </label>
          <input
            id="title"
            v-model="(modPackData[currentIndex] as UnknownModpackItem).title"
            type="text"
            autocomplete="off"
            placeholder="Enter title of project..."
            @input="persistAll()"
          />
        </div>
      </div>

      <div v-else-if="modPackData[currentIndex].type === 'flame'">
        <p>
          What is the approval type of {{ modPackData[currentIndex].title }} (<a
            :href="modPackData[currentIndex].url"
            target="_blank"
            class="text-link"
            >{{ modPackData[currentIndex].url }}</a
          >)?
        </p>
        <div class="input-group">
          <ButtonStyled
            v-for="(option, index) in fileApprovalTypes"
            :key="index"
            :color="modPackData[currentIndex].status === option.id ? 'brand' : 'standard'"
            @click="setStatus(currentIndex, option.id)"
          >
            <button>
              {{ option.name }}
            </button>
          </ButtonStyled>
        </div>
      </div>

      <div
        v-if="
          ['unidentified', 'no', 'with-attribution'].includes(
            modPackData[currentIndex].status || '',
          )
        "
      >
        <p v-if="modPackData[currentIndex].status === 'unidentified'">
          Does this project provide identification and permission for
          <strong>{{ modPackData[currentIndex].file_name }}</strong
          >?
        </p>
        <p v-else-if="modPackData[currentIndex].status === 'with-attribution'">
          Does this project provide attribution for
          <strong>{{ modPackData[currentIndex].file_name }}</strong
          >?
        </p>
        <p v-else>
          Does this project provide proof of permission for
          <strong>{{ modPackData[currentIndex].file_name }}</strong
          >?
        </p>
        <div class="input-group">
          <ButtonStyled
            v-for="(option, index) in filePermissionTypes"
            :key="index"
            :color="modPackData[currentIndex].approved === option.id ? 'brand' : 'standard'"
            @click="setApproval(currentIndex, option.id)"
          >
            <button>
              {{ option.name }}
            </button>
          </ButtonStyled>
        </div>
      </div>
    </div>

    <div class="mt-4 flex gap-2">
      <ButtonStyled>
        <button :disabled="currentIndex <= 0" @click="goToPrevious">
          <LeftArrowIcon aria-hidden="true" />
          Previous
        </button>
      </ButtonStyled>
      <ButtonStyled v-if="modPackData && currentIndex < modPackData.length" color="blue">
        <button :disabled="!canGoNext" @click="goToNext">
          <RightArrowIcon aria-hidden="true" />
          {{ currentIndex + 1 >= modPackData.length ? "Complete" : "Next" }}
        </button>
      </ButtonStyled>
    </div>
  </div>
</template>

<script setup lang="ts">
import { LeftArrowIcon, RightArrowIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";
import { ref, computed, watch, onMounted } from "vue";
import { useLocalStorage } from "@vueuse/core";

interface ApprovalType {
  id: "yes" | "no" | "with-attribution" | "unidentified";
  name: string;
}

interface PermissionType {
  id: "yes" | "no";
  name: string;
}

interface BaseModpackItem {
  sha1: string;
  file_name: string;
  type: "unknown" | "flame";
  status: ApprovalType["id"] | null;
  approved: PermissionType["id"] | null;
}

interface UnknownModpackItem extends BaseModpackItem {
  type: "unknown";
  proof: string;
  url: string;
  title: string;
}

interface FlameModpackItem extends BaseModpackItem {
  type: "flame";
  id: string;
  title: string;
  url: string;
}

type ModpackItem = UnknownModpackItem | FlameModpackItem;

interface ModpackResponse {
  unknown_files?: Record<string, string>;
  flame_files?: Record<
    string,
    {
      file_name: string;
      id: string;
      title?: string;
      url?: string;
    }
  >;
}

interface Judgements {
  [sha1: string]: {
    type: "flame" | "unknown";
    status: string;
    id?: string;
    link?: string | null;
    title?: string | null;
    proof?: string | null;
  };
}

const props = defineProps<{
  projectId: string;
}>();

const emit = defineEmits<{
  complete: [judgements: Judgements];
}>();

const persistedModPackData = useLocalStorage<ModpackItem[] | null>(
  `modpack-permissions-${props.projectId}`,
  null,
  {
    serializer: {
      read: (v: any) => (v ? JSON.parse(v) : null),
      write: (v: any) => JSON.stringify(v),
    },
  },
);

const persistedIndex = useLocalStorage<number>(`modpack-permissions-index-${props.projectId}`, 0);

const modPackData = ref<ModpackItem[] | null>(null);
const currentIndex = ref(0);

const fileApprovalTypes: ApprovalType[] = [
  { id: "yes", name: "Yes, permission given" },
  { id: "no", name: "No permission given" },
  { id: "with-attribution", name: "With attribution only" },
  { id: "unidentified", name: "Cannot identify project" },
];

const filePermissionTypes: PermissionType[] = [
  { id: "yes", name: "Yes" },
  { id: "no", name: "No" },
];

function persistAll() {
  persistedModPackData.value = modPackData.value;
  persistedIndex.value = currentIndex.value;
}

watch(
  modPackData,
  (newValue) => {
    persistedModPackData.value = newValue;
  },
  { deep: true },
);

watch(currentIndex, (newValue) => {
  persistedIndex.value = newValue;
});

function loadPersistedData(): void {
  if (persistedModPackData.value) {
    modPackData.value = persistedModPackData.value;
  }
  currentIndex.value = persistedIndex.value;
}

function clearPersistedData(): void {
  persistedModPackData.value = null;
  persistedIndex.value = 0;
}

async function fetchModPackData(): Promise<void> {
  try {
    const data = (await useBaseFetch(`moderation/project/${props.projectId}`, {
      internal: true,
    })) as ModpackResponse;
    const sortedData: ModpackItem[] = [
      ...Object.entries(data.unknown_files || {})
        .map(
          ([sha1, fileName]): UnknownModpackItem => ({
            sha1,
            file_name: fileName,
            type: "unknown",
            status: null,
            approved: null,
            proof: "",
            url: "",
            title: "",
          }),
        )
        .sort((a, b) => a.file_name.localeCompare(b.file_name)),
      ...Object.entries(data.flame_files || {})
        .map(
          ([sha1, info]): FlameModpackItem => ({
            sha1,
            file_name: info.file_name,
            type: "flame",
            status: null,
            approved: null,
            id: info.id,
            title: info.title || info.file_name,
            url: info.url || `https://www.curseforge.com/minecraft/mc-mods/${info.id}`,
          }),
        )
        .sort((a, b) => a.file_name.localeCompare(b.file_name)),
    ];

    if (modPackData.value) {
      const existingMap = new Map(modPackData.value.map((item) => [item.sha1, item]));

      sortedData.forEach((item) => {
        const existing = existingMap.get(item.sha1);
        if (existing) {
          Object.assign(item, {
            status: existing.status,
            approved: existing.approved,
            ...(item.type === "unknown" && {
              proof: (existing as UnknownModpackItem).proof || "",
              url: (existing as UnknownModpackItem).url || "",
              title: (existing as UnknownModpackItem).title || "",
            }),
            ...(item.type === "flame" && {
              url: (existing as FlameModpackItem).url || item.url,
              title: (existing as FlameModpackItem).title || item.title,
            }),
          });
        }
      });
    }

    modPackData.value = sortedData;
    persistAll();
  } catch (error) {
    console.error("Failed to fetch modpack data:", error);
    modPackData.value = [];
    persistAll();
  }
}

function goToPrevious(): void {
  if (currentIndex.value > 0) {
    currentIndex.value--;
    persistAll();
  }
}

function goToNext(): void {
  if (modPackData.value && currentIndex.value < modPackData.value.length) {
    currentIndex.value++;

    if (currentIndex.value >= modPackData.value.length) {
      emit("complete", getJudgements());
      clearPersistedData();
    } else {
      persistAll();
    }
  }
}

function setStatus(index: number, status: ApprovalType["id"]): void {
  if (modPackData.value && modPackData.value[index]) {
    modPackData.value[index].status = status;
    modPackData.value[index].approved = null;
    persistAll();
  }
}

function setApproval(index: number, approved: PermissionType["id"]): void {
  if (modPackData.value && modPackData.value[index]) {
    modPackData.value[index].approved = approved;
    persistAll();
  }
}

const canGoNext = computed(() => {
  if (!modPackData.value || !modPackData.value[currentIndex.value]) return false;
  const current = modPackData.value[currentIndex.value];
  return current.status !== null;
});

function getJudgements(): Judgements {
  if (!modPackData.value) return {};

  const judgements: Judgements = {};

  modPackData.value.forEach((item) => {
    if (item.status && item.status !== "unidentified") {
      if (item.type === "flame") {
        judgements[item.sha1] = {
          type: "flame",
          id: item.id,
          status: item.status,
          link: item.url,
          title: item.title,
        };
      } else if (item.type === "unknown") {
        judgements[item.sha1] = {
          type: "unknown",
          status: item.status,
          proof: item.proof || null,
          link: item.url || null,
          title: item.title || null,
        };
      }
    }
  });

  return judgements;
}

onMounted(() => {
  loadPersistedData();
  if (!modPackData.value) {
    fetchModPackData();
  }
});

watch(
  () => props.projectId,
  () => {
    clearPersistedData();
    loadPersistedData();
    if (!modPackData.value) {
      fetchModPackData();
    }
  },
);
</script>

<style scoped>
.input-group {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.5rem;
  margin-bottom: 0.5rem;
}

.modpack-buttons {
  margin-top: 1rem;
}
</style>
