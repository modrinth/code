<template>
  <div
    class="experimental-styles-within relative mx-auto mb-6 flex min-h-screen w-full max-w-[1280px] flex-col px-6"
  >
    <h1>Moderation</h1>
    <NavTabs :links="moderationLinks" class="mb-4" />
    <NuxtPage />
  </div>
</template>

<script setup lang="ts">
import { defineMessages, useVIntl } from "@vintl/vintl";
import NavTabs from "@/components/ui/NavTabs.vue";

definePageMeta({
  middleware: "auth",
});

const { formatMessage } = useVIntl();
const auth = await useAuth();

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
</script>
