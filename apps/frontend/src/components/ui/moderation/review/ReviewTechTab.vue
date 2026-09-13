<template>
	<TechReviewPage :projectId="projectV3.id"/>
</template>

<script setup lang="ts">

import {injectProjectPageContext} from "@modrinth/ui";
import {defineAsyncComponent} from "vue";

const TechReviewPage = defineAsyncComponent(() => import('~/pages/moderation/technical-review/[project].vue'))

const { projectV3 } = injectProjectPageContext()

</script>
