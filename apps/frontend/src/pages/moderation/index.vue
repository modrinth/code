<template> <NavRow :links="moderationLinks" /> </template>

<script lang="ts" setup>
 import { defineMessages, useVintl } from "@vintl/vintl"; import NavRow from
"@/components/ui/NavRow.vue"; const { formatMessage } = useVintl(); const messages =
defineMessages({}); const moderationLinks = [{ label: }];
</script>

