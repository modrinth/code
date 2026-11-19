<template>
	<slot />
</template>

<script setup lang="ts">
import { computed, provide, shallowReactive } from 'vue'
import {
	matchedRouteKey,
	type RouteLocationNormalizedLoaded,
	routeLocationKey,
	routerViewLocationKey,
	START_LOCATION,
} from 'vue-router'

const props = defineProps<{
	route: RouteLocationNormalizedLoaded
}>()

const scopedRoute = {} as RouteLocationNormalizedLoaded
for (const key in START_LOCATION) {
	Object.defineProperty(scopedRoute, key, {
		get: () => props.route[key as keyof RouteLocationNormalizedLoaded],
		enumerable: true,
	})
}

const route = computed(() => props.route)

provide(routeLocationKey, shallowReactive(scopedRoute))
provide(routerViewLocationKey, route)
provide(
	matchedRouteKey,
	computed(() => route.value.matched.find((record) => record.components)),
)
</script>
