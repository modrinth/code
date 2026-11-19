<template>
	<slot />
</template>

<script setup lang="ts">
import { provideRouteScope, useRouteScope } from '@modrinth/ui'
import { computed, provide, shallowReactive, toRef } from 'vue'
import {
	matchedRouteKey,
	routeLocationKey,
	type RouteLocationNormalizedLoaded,
	routerViewLocationKey,
	START_LOCATION,
} from 'vue-router'

const props = defineProps<{
	route: RouteLocationNormalizedLoaded
}>()

const route = toRef(props, 'route')

const scopedRoute = {} as RouteLocationNormalizedLoaded
for (const key in START_LOCATION) {
	Object.defineProperty(scopedRoute, key, {
		get: () => route.value[key as keyof RouteLocationNormalizedLoaded],
		enumerable: true,
	})
}

provide(routeLocationKey, shallowReactive(scopedRoute))
provide(routerViewLocationKey, route)
provide(
	matchedRouteKey,
	computed(() => route.value.matched.find((record) => record.components)),
)
provideRouteScope(useRouteScope())
</script>
