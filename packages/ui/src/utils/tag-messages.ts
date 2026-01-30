import { capitalizeString } from '@modrinth/utils'

import { defineMessages, type MessageDescriptor, type VIntlFormatters } from '../composables/i18n'

export const loaderMessages = defineMessages({
	babric: {
		id: 'tag.loader.babric',
		defaultMessage: 'Babric',
	},
	'bta-babric': {
		id: 'tag.loader.bta-babric',
		defaultMessage: 'BTA (Babric)',
	},
	bukkit: {
		id: 'tag.loader.bukkit',
		defaultMessage: 'Bukkit',
	},
	bungeecord: {
		id: 'tag.loader.bungeecord',
		defaultMessage: 'BungeeCord',
	},
	canvas: {
		id: 'tag.loader.canvas',
		defaultMessage: 'Canvas',
	},
	datapack: {
		id: 'tag.loader.datapack',
		defaultMessage: 'Data Pack',
	},
	fabric: {
		id: 'tag.loader.fabric',
		defaultMessage: 'Fabric',
	},
	folia: {
		id: 'tag.loader.folia',
		defaultMessage: 'Folia',
	},
	forge: {
		id: 'tag.loader.forge',
		defaultMessage: 'Forge',
	},
	geyser: {
		id: 'tag.loader.geyser',
		defaultMessage: 'Geyser Extension',
	},
	iris: {
		id: 'tag.loader.iris',
		defaultMessage: 'Iris',
	},
	'java-agent': {
		id: 'tag.loader.java-agent',
		defaultMessage: 'Java Agent',
	},
	'legacy-fabric': {
		id: 'tag.loader.legacy-fabric',
		defaultMessage: 'Legacy Fabric',
	},
	liteloader: {
		id: 'tag.loader.liteloader',
		defaultMessage: 'LiteLoader',
	},
	minecraft: {
		id: 'tag.loader.minecraft',
		defaultMessage: 'Resource Pack',
	},
	modloader: {
		id: 'tag.loader.modloader',
		defaultMessage: "Risugami's ModLoader",
	},
	mrpack: {
		id: 'tag.loader.mrpack',
		defaultMessage: 'Modpack',
	},
	neoforge: {
		id: 'tag.loader.neoforge',
		defaultMessage: 'NeoForge',
	},
	nilloader: {
		id: 'tag.loader.nilloader',
		defaultMessage: 'NilLoader',
	},
	optifine: {
		id: 'tag.loader.optifine',
		defaultMessage: 'OptiFine',
	},
	ornithe: {
		id: 'tag.loader.ornithe',
		defaultMessage: 'Ornithe',
	},
	paper: {
		id: 'tag.loader.paper',
		defaultMessage: 'Paper',
	},
	purpur: {
		id: 'tag.loader.purpur',
		defaultMessage: 'Purpur',
	},
	quilt: {
		id: 'tag.loader.quilt',
		defaultMessage: 'Quilt',
	},
	rift: {
		id: 'tag.loader.rift',
		defaultMessage: 'Rift',
	},
	spigot: {
		id: 'tag.loader.spigot',
		defaultMessage: 'Spigot',
	},
	sponge: {
		id: 'tag.loader.sponge',
		defaultMessage: 'Sponge',
	},
	vanilla: {
		id: 'tag.loader.vanilla',
		defaultMessage: 'Vanilla Shader',
	},
	velocity: {
		id: 'tag.loader.velocity',
		defaultMessage: 'Velocity',
	},
	waterfall: {
		id: 'tag.loader.waterfall',
		defaultMessage: 'Waterfall',
	},
})

export const categoryMessages = defineMessages({
	'128x': {
		id: 'tag.category.128x',
		defaultMessage: '128x',
	},
	'16x': {
		id: 'tag.category.16x',
		defaultMessage: '16x',
	},
	'256x': {
		id: 'tag.category.256x',
		defaultMessage: '256x',
	},
	'32x': {
		id: 'tag.category.32x',
		defaultMessage: '32x',
	},
	'48x': {
		id: 'tag.category.48x',
		defaultMessage: '48x',
	},
	'512x+': {
		id: 'tag.category.512x+',
		defaultMessage: '512x or higher',
	},
	'64x': {
		id: 'tag.category.64x',
		defaultMessage: '64x',
	},
	'8x-': {
		id: 'tag.category.8x-',
		defaultMessage: '8x or lower',
	},
	adventure: {
		id: 'tag.category.adventure',
		defaultMessage: 'Adventure',
	},
	atmosphere: {
		id: 'tag.category.atmosphere',
		defaultMessage: 'Atmosphere',
	},
	audio: {
		id: 'tag.category.audio',
		defaultMessage: 'Audio',
	},
	blocks: {
		id: 'tag.category.blocks',
		defaultMessage: 'Blocks',
	},
	bloom: {
		id: 'tag.category.bloom',
		defaultMessage: 'Bloom',
	},
	cartoon: {
		id: 'tag.category.cartoon',
		defaultMessage: 'Cartoon',
	},
	challenging: {
		id: 'tag.category.challenging',
		defaultMessage: 'Challenging',
	},
	'colored-lighting': {
		id: 'tag.category.colored-lighting',
		defaultMessage: 'Colored Lighting',
	},
	combat: {
		id: 'tag.category.combat',
		defaultMessage: 'Combat',
	},
	'core-shaders': {
		id: 'tag.category.core-shaders',
		defaultMessage: 'Core Shaders',
	},
	cursed: {
		id: 'tag.category.cursed',
		defaultMessage: 'Cursed',
	},
	decoration: {
		id: 'tag.category.decoration',
		defaultMessage: 'Decoration',
	},
	economy: {
		id: 'tag.category.economy',
		defaultMessage: 'Economy',
	},
	entities: {
		id: 'tag.category.entities',
		defaultMessage: 'Entities',
	},
	environment: {
		id: 'tag.category.environment',
		defaultMessage: 'Environment',
	},
	equipment: {
		id: 'tag.category.equipment',
		defaultMessage: 'Equipment',
	},
	fantasy: {
		id: 'tag.category.fantasy',
		defaultMessage: 'Fantasy',
	},
	foliage: {
		id: 'tag.category.foliage',
		defaultMessage: 'Foliage',
	},
	fonts: {
		id: 'tag.category.fonts',
		defaultMessage: 'Fonts',
	},
	food: {
		id: 'tag.category.food',
		defaultMessage: 'Food',
	},
	'game-mechanics': {
		id: 'tag.category.game-mechanics',
		defaultMessage: 'Game Mechanics',
	},
	gui: {
		id: 'tag.category.gui',
		defaultMessage: 'GUI',
	},
	high: {
		id: 'tag.category.high',
		defaultMessage: 'High',
	},
	items: {
		id: 'tag.category.items',
		defaultMessage: 'Items',
	},
	'kitchen-sink': {
		id: 'tag.category.kitchen-sink',
		defaultMessage: 'Kitchen Sink',
	},
	library: {
		id: 'tag.category.library',
		defaultMessage: 'Library',
	},
	lightweight: {
		id: 'tag.category.lightweight',
		defaultMessage: 'Lightweight',
	},
	locale: {
		id: 'tag.category.locale',
		defaultMessage: 'Locale',
	},
	low: {
		id: 'tag.category.low',
		defaultMessage: 'Low',
	},
	magic: {
		id: 'tag.category.magic',
		defaultMessage: 'Magic',
	},
	management: {
		id: 'tag.category.management',
		defaultMessage: 'Management',
	},
	medium: {
		id: 'tag.category.medium',
		defaultMessage: 'Medium',
	},
	minigame: {
		id: 'tag.category.minigame',
		defaultMessage: 'Minigame',
	},
	mobs: {
		id: 'tag.category.mobs',
		defaultMessage: 'Mobs',
	},
	modded: {
		id: 'tag.category.modded',
		defaultMessage: 'Modded',
	},
	models: {
		id: 'tag.category.models',
		defaultMessage: 'Models',
	},
	multiplayer: {
		id: 'tag.category.multiplayer',
		defaultMessage: 'Multiplayer',
	},
	optimization: {
		id: 'tag.category.optimization',
		defaultMessage: 'Optimization',
	},
	'path-tracing': {
		id: 'tag.category.path-tracing',
		defaultMessage: 'Path Tracing',
	},
	pbr: {
		id: 'tag.category.pbr',
		defaultMessage: 'PBR',
	},
	potato: {
		id: 'tag.category.potato',
		defaultMessage: 'Potato',
	},
	quests: {
		id: 'tag.category.quests',
		defaultMessage: 'Quests',
	},
	realistic: {
		id: 'tag.category.realistic',
		defaultMessage: 'Realistic',
	},
	reflections: {
		id: 'tag.category.reflections',
		defaultMessage: 'Reflections',
	},
	screenshot: {
		id: 'tag.category.screenshot',
		defaultMessage: 'Screenshot',
	},
	'semi-realistic': {
		id: 'tag.category.semi-realistic',
		defaultMessage: 'Semi Realistic',
	},
	shadows: {
		id: 'tag.category.shadows',
		defaultMessage: 'Shadows',
	},
	simplistic: {
		id: 'tag.category.simplistic',
		defaultMessage: 'Simplistic',
	},
	social: {
		id: 'tag.category.social',
		defaultMessage: 'Social',
	},
	storage: {
		id: 'tag.category.storage',
		defaultMessage: 'Storage',
	},
	technology: {
		id: 'tag.category.technology',
		defaultMessage: 'Technology',
	},
	themed: {
		id: 'tag.category.themed',
		defaultMessage: 'Themed',
	},
	transportation: {
		id: 'tag.category.transportation',
		defaultMessage: 'Transportation',
	},
	tweaks: {
		id: 'tag.category.tweaks',
		defaultMessage: 'Tweaks',
	},
	utility: {
		id: 'tag.category.utility',
		defaultMessage: 'Utility',
	},
	'vanilla-like': {
		id: 'tag.category.vanilla-like',
		defaultMessage: 'Vanilla Like',
	},
	worldgen: {
		id: 'tag.category.worldgen',
		defaultMessage: 'World Generation',
	},
})

export function getTagMessage(
	tag: string,
	enforceType?: 'loader' | 'category',
): MessageDescriptor | undefined {
	if (enforceType === 'loader') {
		return loaderMessages[tag]
	} else if (enforceType === 'category') {
		return categoryMessages[tag]
	} else {
		return loaderMessages[tag] ?? categoryMessages[tag]
	}
}

export function getLoaderMessage(loader: string) {
	return getTagMessage(loader, 'loader')
}

export function getCategoryMessage(category: string) {
	return getTagMessage(category, 'category')
}

export function formatTag(
	formatter: VIntlFormatters['formatMessage'],
	tag: string,
	enforceType?: 'loader' | 'category',
) {
	const message = getTagMessage(tag, enforceType)
	return message ? formatter(message) : capitalizeString(tag)
}

export function formatCategory(formatter: VIntlFormatters['formatMessage'], category: string) {
	return formatTag(formatter, category, 'category')
}

export function formatLoader(formatter: VIntlFormatters['formatMessage'], category: string) {
	return formatTag(formatter, category, 'category')
}
