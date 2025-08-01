<template>
  <div
    class="experimental-styles-within relative mx-auto mb-6 flex min-h-screen w-full max-w-[1280px] flex-col px-6"
  >
    <h1>Moderation</h1>
    <NavTabs :links="moderationLinks" class="mb-4 hidden sm:flex" />
    <div class="mb-4 sm:hidden">
      <Chips
        v-model="selectedChip"
        :items="mobileNavOptions"
        :never-empty="true"
        @change="navigateToPage"
      />
    </div>
    <NuxtPage />
  </div>
</template>

<script setup lang="ts">
import { defineMessages, useVIntl } from "@vintl/vintl";
import { Chips } from "@modrinth/ui";
import NavTabs from "@/components/ui/NavTabs.vue";

definePageMeta({
  middleware: "auth",
});

const { formatMessage } = useVIntl();
const route = useRoute();
const router = useRouter();

const messages = defineMessages({
  projectsTitle: {
    id: "moderation.page.projects",
    defaultMessage: "Projects",
  },
  technicalReviewTitle: {
    id: "moderation.page.technicalReview",
    defaultMessage: "Technical Review",
  },
  reportsTitle: {
    id: "moderation.page.reports",
    defaultMessage: "Reports",
  },
});

const moderationLinks = [
  { label: formatMessage(messages.projectsTitle), href: "/moderation" },
  { label: formatMessage(messages.technicalReviewTitle), href: "/moderation/technical-review" },
  { label: formatMessage(messages.reportsTitle), href: "/moderation/reports" },
];

const mobileNavOptions = [
  formatMessage(messages.projectsTitle),
  formatMessage(messages.technicalReviewTitle),
  formatMessage(messages.reportsTitle),
];

const selectedChip = computed({
  get() {
    const path = route.path;
    if (path === "/moderation/technical-review") {
      return formatMessage(messages.technicalReviewTitle);
    } else if (path.startsWith("/moderation/reports/")) {
      return formatMessage(messages.reportsTitle);
    } else {
      return formatMessage(messages.projectsTitle);
    }
  },
  set(value: string) {
    navigateToPage(value);
  },
});

function navigateToPage(selectedOption: string) {
  if (selectedOption === formatMessage(messages.technicalReviewTitle)) {
    router.push("/moderation/technical-review");
  } else if (selectedOption === formatMessage(messages.reportsTitle)) {
    router.push("/moderation/reports");
  } else {
    router.push("/moderation");
  }
}
</script>
