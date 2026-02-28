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
	'adventure-mode': {
		id: 'tag.category.adventure-mode',
		defaultMessage: 'Adventure Mode',
	},
	anarchy: {
		id: 'tag.category.anarchy',
		defaultMessage: 'Anarchy',
	},
	atmosphere: {
		id: 'tag.category.atmosphere',
		defaultMessage: 'Atmosphere',
	},
	audio: {
		id: 'tag.category.audio',
		defaultMessage: 'Audio',
	},
	'battle-royale': {
		id: 'tag.category.battle-royale',
		defaultMessage: 'Battle Royale',
	},
	bedwars: {
		id: 'tag.category.bedwars',
		defaultMessage: 'Bed Wars',
	},
	blocks: {
		id: 'tag.category.blocks',
		defaultMessage: 'Blocks',
	},
	bloom: {
		id: 'tag.category.bloom',
		defaultMessage: 'Bloom',
	},
	bosses: {
		id: 'tag.category.bosses',
		defaultMessage: 'Bosses',
	},
	cartoon: {
		id: 'tag.category.cartoon',
		defaultMessage: 'Cartoon',
	},
	challenging: {
		id: 'tag.category.challenging',
		defaultMessage: 'Challenging',
	},
	classes: {
		id: 'tag.category.classes',
		defaultMessage: 'Classes',
	},
	'colored-lighting': {
		id: 'tag.category.colored-lighting',
		defaultMessage: 'Colored Lighting',
	},
	combat: {
		id: 'tag.category.combat',
		defaultMessage: 'Combat',
	},
	competitive: {
		id: 'tag.category.competitive',
		defaultMessage: 'Competitive',
	},
	'core-shaders': {
		id: 'tag.category.core-shaders',
		defaultMessage: 'Core Shaders',
	},
	'creative-mode': {
		id: 'tag.category.creative-mode',
		defaultMessage: 'Creative Mode',
	},
	'creator-community': {
		id: 'tag.category.creator-community',
		defaultMessage: 'Creator Community',
	},
	crossplay: {
		id: 'tag.category.crossplay',
		defaultMessage: 'Crossplay',
	},
	cursed: {
		id: 'tag.category.cursed',
		defaultMessage: 'Cursed',
	},
	'custom-content': {
		id: 'tag.category.custom-content',
		defaultMessage: 'Custom Content',
	},
	decoration: {
		id: 'tag.category.decoration',
		defaultMessage: 'Decoration',
	},
	dungeons: {
		id: 'tag.category.dungeons',
		defaultMessage: 'Dungeons',
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
	factions: {
		id: 'tag.category.factions',
		defaultMessage: 'Factions',
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
	gens: {
		id: 'tag.category.gens',
		defaultMessage: 'Gens',
	},
	gui: {
		id: 'tag.category.gui',
		defaultMessage: 'GUI',
	},
	'hardcore-mode': {
		id: 'tag.category.hardcore-mode',
		defaultMessage: 'Hardcore Mode',
	},
	high: {
		id: 'tag.category.high',
		defaultMessage: 'High',
	},
	items: {
		id: 'tag.category.items',
		defaultMessage: 'Items',
	},
	'keep-inventory': {
		id: 'tag.category.keep-inventory',
		defaultMessage: 'Keep Inventory',
	},
	'kitchen-sink': {
		id: 'tag.category.kitchen-sink',
		defaultMessage: 'Kitchen Sink',
	},
	kitpvp: {
		id: 'tag.category.kitpvp',
		defaultMessage: 'Kit PVP',
	},
	library: {
		id: 'tag.category.library',
		defaultMessage: 'Library',
	},
	lifesteal: {
		id: 'tag.category.lifesteal',
		defaultMessage: 'Lifesteal',
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
	media: {
		id: 'tag.category.media',
		defaultMessage: 'Media',
	},
	medium: {
		id: 'tag.category.medium',
		defaultMessage: 'Medium',
	},
	microgames: {
		id: 'tag.category.microgames',
		defaultMessage: 'Microgames',
	},
	minigame: {
		id: 'tag.category.minigame',
		defaultMessage: 'Minigame',
	},
	minigames: {
		id: 'tag.category.minigames',
		defaultMessage: 'Minigames',
	},
	mmo: {
		id: 'tag.category.mmo',
		defaultMessage: 'MMO',
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
	network: {
		id: 'tag.category.network',
		defaultMessage: 'Network',
	},
	'offline-mode': {
		id: 'tag.category.offline-mode',
		defaultMessage: 'Offline Mode',
	},
	oneblock: {
		id: 'tag.category.oneblock',
		defaultMessage: 'One Block',
	},
	op: {
		id: 'tag.category.op',
		defaultMessage: 'OP',
	},
	optimization: {
		id: 'tag.category.optimization',
		defaultMessage: 'Optimization',
	},
	parkour: {
		id: 'tag.category.parkour',
		defaultMessage: 'Parkour',
	},
	'path-tracing': {
		id: 'tag.category.path-tracing',
		defaultMessage: 'Path Tracing',
	},
	pbr: {
		id: 'tag.category.pbr',
		defaultMessage: 'PBR',
	},
	'personal-worlds': {
		id: 'tag.category.personal-worlds',
		defaultMessage: 'Personal Worlds',
	},
	plots: {
		id: 'tag.category.plots',
		defaultMessage: 'Plots',
	},
	pokemon: {
		id: 'tag.category.pokemon',
		defaultMessage: 'Pokemon',
	},
	potato: {
		id: 'tag.category.potato',
		defaultMessage: 'Potato',
	},
	prison: {
		id: 'tag.category.prison',
		defaultMessage: 'Prison',
	},
	pve: {
		id: 'tag.category.pve',
		defaultMessage: 'PVE',
	},
	pvp: {
		id: 'tag.category.pvp',
		defaultMessage: 'PVP',
	},
	questing: {
		id: 'tag.category.questing',
		defaultMessage: 'Questing',
	},
	quests: {
		id: 'tag.category.quests',
		defaultMessage: 'Quests',
	},
	racing: {
		id: 'tag.category.racing',
		defaultMessage: 'Racing',
	},
	realistic: {
		id: 'tag.category.realistic',
		defaultMessage: 'Realistic',
	},
	'recording-smp': {
		id: 'tag.category.recording-smp',
		defaultMessage: 'Recording SMP',
	},
	reflections: {
		id: 'tag.category.reflections',
		defaultMessage: 'Reflections',
	},
	roleplay: {
		id: 'tag.category.roleplay',
		defaultMessage: 'Roleplay',
	},
	rpg: {
		id: 'tag.category.rpg',
		defaultMessage: 'RPG',
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
	skyblock: {
		id: 'tag.category.skyblock',
		defaultMessage: 'Skyblock',
	},
	smp: {
		id: 'tag.category.smp',
		defaultMessage: 'SMP',
	},
	social: {
		id: 'tag.category.social',
		defaultMessage: 'Social',
	},
	storage: {
		id: 'tag.category.storage',
		defaultMessage: 'Storage',
	},
	'survival-mode': {
		id: 'tag.category.survival-mode',
		defaultMessage: 'Survival Mode',
	},
	teams: {
		id: 'tag.category.teams',
		defaultMessage: 'Teams',
	},
	technical: {
		id: 'tag.category.technical',
		defaultMessage: 'Technical',
	},
	technology: {
		id: 'tag.category.technology',
		defaultMessage: 'Technology',
	},
	themed: {
		id: 'tag.category.themed',
		defaultMessage: 'Themed',
	},
	towns: {
		id: 'tag.category.towns',
		defaultMessage: 'Towns',
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
	whitelisted: {
		id: 'tag.category.whitelisted',
		defaultMessage: 'Whitelisted',
	},
	'world-resets': {
		id: 'tag.category.world-resets',
		defaultMessage: 'World Resets',
	},
	worldgen: {
		id: 'tag.category.worldgen',
		defaultMessage: 'World Generation',
	},
})

export const DEFAULT_MOD_LOADERS: string[] = ['fabric', 'forge', 'neoforge']
export const DEFAULT_SHADER_LOADERS: string[] = ['iris', 'optifine', 'vanilla']

const DEFAULT_LOADER_NAMES = new Set([...DEFAULT_MOD_LOADERS, ...DEFAULT_SHADER_LOADERS])

// sort by:
// 1. categories, alphabetically
// 2. default loaders, alphabetically
// 3. other loaders, alphabetically
export function sortTagsForDisplay(tags: string[]): string[] {
	const isLoader = (tag: string) => getTagMessage(tag, 'loader') !== undefined
	const loaders = tags.filter(isLoader)
	const categories = tags.filter((tag) => !isLoader(tag))
	categories.sort((a, b) => a.localeCompare(b))
	loaders.sort((a, b) => {
		const aDefault = DEFAULT_LOADER_NAMES.has(a)
		const bDefault = DEFAULT_LOADER_NAMES.has(b)
		if (aDefault !== bDefault) return aDefault ? -1 : 1
		return a.localeCompare(b)
	})
	return [...categories, ...loaders]
}

export const categoryHeaderMessages = defineMessages({
	resolutions: {
		id: 'header.category.resolutions',
		defaultMessage: 'Resolutions',
	},
	categories: {
		id: 'header.category.category',
		defaultMessage: 'Category',
	},
	features: {
		id: 'header.category.feature',
		defaultMessage: 'Feature',
	},
	'performance impact': {
		id: 'header.category.performance-impact',
		defaultMessage: 'Performance impact',
	},
	minecraft_server_community: {
		id: 'header.category.minecraft-server-community',
		defaultMessage: 'Community',
	},
	minecraft_server_features: {
		id: 'header.category.minecraft-server-features',
		defaultMessage: 'Features',
	},
	minecraft_server_gameplay: {
		id: 'header.category.minecraft-server-gameplay',
		defaultMessage: 'Gameplay',
	},
	minecraft_server_meta: {
		id: 'header.category.minecraft-server-meta',
		defaultMessage: 'Meta',
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

export function getCategoryHeaderMessage(header: string): MessageDescriptor | undefined {
	return categoryHeaderMessages[header]
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
	return formatTag(formatter, category, 'loader')
}

export function formatCategoryHeader(formatter: VIntlFormatters['formatMessage'], header: string) {
	const message = getCategoryHeaderMessage(header)
	return message ? formatter(message) : capitalizeString(header)
}
