import { defineMessages, type MessageDescriptor } from '@modrinth/ui'

import backpack from '@/assets/instance-icons/backpack.png'
import beacon from '@/assets/instance-icons/beacon.png'
import blueShark from '@/assets/instance-icons/blue-shark.png'
import bookshelf from '@/assets/instance-icons/bookshelf.png'
import brownBear from '@/assets/instance-icons/brown-bear.png'
import cake from '@/assets/instance-icons/cake.png'
import campfire from '@/assets/instance-icons/campfire.png'
import chest from '@/assets/instance-icons/chest.png'
import cogwheel from '@/assets/instance-icons/cogwheel.png'
import commandBlock from '@/assets/instance-icons/command-block.png'
import cookingPot from '@/assets/instance-icons/cooking-pot.png'
import couch from '@/assets/instance-icons/couch.png'
import craftingTable from '@/assets/instance-icons/crafting-table.png'
import creeper from '@/assets/instance-icons/creeper.png'
import enchantingTable from '@/assets/instance-icons/enchanting-table.png'
import enderChest from '@/assets/instance-icons/ender-chest.png'
import enderDragon from '@/assets/instance-icons/ender-dragon.png'
import engine from '@/assets/instance-icons/engine.png'
import fabric from '@/assets/instance-icons/fabric.png'
import forge from '@/assets/instance-icons/forge.png'
import furnace from '@/assets/instance-icons/furnace.png'
import gizmo from '@/assets/instance-icons/gizmo.png'
import globe from '@/assets/instance-icons/globe.png'
import grassBlock from '@/assets/instance-icons/grass-block.png'
import lantern from '@/assets/instance-icons/lantern.png'
import moobloom from '@/assets/instance-icons/moobloom.png'
import mrPack from '@/assets/instance-icons/mr-pack.png'
import neoForge from '@/assets/instance-icons/neoforge.png'
import orb from '@/assets/instance-icons/orb.png'
import oxygenDistributor from '@/assets/instance-icons/oxygen-distributor.png'
import pancakes from '@/assets/instance-icons/pancakes.png'
import pickaxe from '@/assets/instance-icons/pickaxe.png'
import pokeBall from '@/assets/instance-icons/poke-ball.png'
import quilt from '@/assets/instance-icons/quilt.png'
import redstoneBlock from '@/assets/instance-icons/redstone-block.png'
import sculkSensor from '@/assets/instance-icons/sculk-sensor.png'
import skeleton from '@/assets/instance-icons/skeleton.png'
import skillet from '@/assets/instance-icons/skillet.png'
import slimeBlock from '@/assets/instance-icons/slime-block.png'
import spaceHelmet from '@/assets/instance-icons/space-helmet.png'
import stickyPiston from '@/assets/instance-icons/sticky-piston.png'
import sword from '@/assets/instance-icons/sword.png'
import terminal from '@/assets/instance-icons/terminal.png'
import tinyPotato from '@/assets/instance-icons/tiny-potato.png'
import tire from '@/assets/instance-icons/tire.png'
import tnt from '@/assets/instance-icons/tnt.png'
import wrench from '@/assets/instance-icons/wrench.png'
import wrenchRinth from '@/assets/instance-icons/wrench-rinth.png'
import zombie from '@/assets/instance-icons/zombie.png'

const names = defineMessages({
	yellow: {
		id: 'instance.icon-editor.background.yellow',
		defaultMessage: 'Yellow',
	},
	green: {
		id: 'instance.icon-editor.background.green',
		defaultMessage: 'Green',
	},
	lime: {
		id: 'instance.icon-editor.background.lime',
		defaultMessage: 'Lime',
	},
	darkGreen: {
		id: 'instance.icon-editor.background.dark-green',
		defaultMessage: 'Dark green',
	},
	purple: {
		id: 'instance.icon-editor.background.purple',
		defaultMessage: 'Purple',
	},
	blue: {
		id: 'instance.icon-editor.background.blue',
		defaultMessage: 'Blue',
	},
	orange: {
		id: 'instance.icon-editor.background.orange',
		defaultMessage: 'Orange',
	},
	red: {
		id: 'instance.icon-editor.background.red',
		defaultMessage: 'Red',
	},
	rose: {
		id: 'instance.icon-editor.background.rose',
		defaultMessage: 'Rose',
	},
	pink: {
		id: 'instance.icon-editor.background.pink',
		defaultMessage: 'Pink',
	},
	indigo: {
		id: 'instance.icon-editor.background.indigo',
		defaultMessage: 'Indigo',
	},
	lavender: {
		id: 'instance.icon-editor.background.lavender',
		defaultMessage: 'Lavender',
	},
	lightGray: {
		id: 'instance.icon-editor.background.light-gray',
		defaultMessage: 'Light gray',
	},
	gray: {
		id: 'instance.icon-editor.background.gray',
		defaultMessage: 'Gray',
	},
	darkGray: {
		id: 'instance.icon-editor.background.dark-gray',
		defaultMessage: 'Dark gray',
	},
	backpack: { id: 'instance.icon-editor.symbol.backpack', defaultMessage: 'Backpack' },
	beacon: { id: 'instance.icon-editor.symbol.beacon', defaultMessage: 'Beacon' },
	blueShark: { id: 'instance.icon-editor.symbol.blue-shark', defaultMessage: 'Blue Shark' },
	bookshelf: { id: 'instance.icon-editor.symbol.bookshelf', defaultMessage: 'Bookshelf' },
	brownBear: { id: 'instance.icon-editor.symbol.brown-bear', defaultMessage: 'Brown Bear' },
	cake: { id: 'instance.icon-editor.symbol.cake', defaultMessage: 'Cake' },
	campfire: { id: 'instance.icon-editor.symbol.campfire', defaultMessage: 'Campfire' },
	chest: { id: 'instance.icon-editor.symbol.chest', defaultMessage: 'Chest' },
	cogwheel: { id: 'instance.icon-editor.symbol.cogwheel', defaultMessage: 'Cogwheel' },
	commandBlock: {
		id: 'instance.icon-editor.symbol.command-block',
		defaultMessage: 'Command Block',
	},
	cookingPot: {
		id: 'instance.icon-editor.symbol.cooking-pot',
		defaultMessage: 'Cooking Pot',
	},
	couch: { id: 'instance.icon-editor.symbol.couch', defaultMessage: 'Couch' },
	craftingTable: {
		id: 'instance.icon-editor.symbol.crafting-table',
		defaultMessage: 'Crafting Table',
	},
	creeper: { id: 'instance.icon-editor.symbol.creeper', defaultMessage: 'Creeper' },
	enchantingTable: {
		id: 'instance.icon-editor.symbol.enchanting-table',
		defaultMessage: 'Enchanting Table',
	},
	enderChest: {
		id: 'instance.icon-editor.symbol.ender-chest',
		defaultMessage: 'Ender Chest',
	},
	enderDragon: {
		id: 'instance.icon-editor.symbol.ender-dragon',
		defaultMessage: 'Ender Dragon',
	},
	engine: { id: 'instance.icon-editor.symbol.engine', defaultMessage: 'Engine' },
	furnace: { id: 'instance.icon-editor.symbol.furnace', defaultMessage: 'Furnace' },
	gizmo: { id: 'instance.icon-editor.symbol.gizmo', defaultMessage: 'Gizmo' },
	globe: { id: 'instance.icon-editor.symbol.globe', defaultMessage: 'Globe' },
	grassBlock: {
		id: 'instance.icon-editor.symbol.grass-block',
		defaultMessage: 'Grass Block',
	},
	lantern: { id: 'instance.icon-editor.symbol.lantern', defaultMessage: 'Lantern' },
	moobloom: { id: 'instance.icon-editor.symbol.moobloom', defaultMessage: 'Moobloom' },
	mrPack: { id: 'instance.icon-editor.symbol.mr-pack', defaultMessage: 'Mr Pack' },
	orb: { id: 'instance.icon-editor.symbol.orb', defaultMessage: 'Orb' },
	oxygenDistributor: {
		id: 'instance.icon-editor.symbol.oxygen-distributor',
		defaultMessage: 'Oxygen Distributor',
	},
	pancakes: { id: 'instance.icon-editor.symbol.pancakes', defaultMessage: 'Pancakes' },
	pickaxe: { id: 'instance.icon-editor.symbol.pickaxe', defaultMessage: 'Pickaxe' },
	pokeBall: { id: 'instance.icon-editor.symbol.poke-ball', defaultMessage: 'Poke Ball' },
	redstoneBlock: {
		id: 'instance.icon-editor.symbol.redstone-block',
		defaultMessage: 'Redstone Block',
	},
	sculkSensor: {
		id: 'instance.icon-editor.symbol.sculk-sensor',
		defaultMessage: 'Sculk Sensor',
	},
	skeleton: { id: 'instance.icon-editor.symbol.skeleton', defaultMessage: 'Skeleton' },
	skillet: { id: 'instance.icon-editor.symbol.skillet', defaultMessage: 'Skillet' },
	slimeBlock: {
		id: 'instance.icon-editor.symbol.slime-block',
		defaultMessage: 'Slime Block',
	},
	spaceHelmet: {
		id: 'instance.icon-editor.symbol.space-helmet',
		defaultMessage: 'Space Helmet',
	},
	stickyPiston: {
		id: 'instance.icon-editor.symbol.sticky-piston',
		defaultMessage: 'Sticky Piston',
	},
	sword: { id: 'instance.icon-editor.symbol.sword', defaultMessage: 'Sword' },
	tnt: { id: 'instance.icon-editor.symbol.tnt', defaultMessage: 'TNT' },
	terminal: { id: 'instance.icon-editor.symbol.terminal', defaultMessage: 'Terminal' },
	tinyPotato: {
		id: 'instance.icon-editor.symbol.tiny-potato',
		defaultMessage: 'Tiny Potato',
	},
	tire: { id: 'instance.icon-editor.symbol.tire', defaultMessage: 'Tire' },
	wrench: { id: 'instance.icon-editor.symbol.create-wrench', defaultMessage: 'Wrench' },
	wrenchRinth: {
		id: 'instance.icon-editor.symbol.wrenth-rinth',
		defaultMessage: 'Modrinth Wrench',
	},
	zombie: { id: 'instance.icon-editor.symbol.zombie', defaultMessage: 'Zombie' },
	fabric: { id: 'instance.icon-editor.symbol.fabric', defaultMessage: 'Fabric' },
	forge: { id: 'instance.icon-editor.symbol.forge', defaultMessage: 'Forge' },
	neoForge: { id: 'instance.icon-editor.symbol.neoforge', defaultMessage: 'NeoForge' },
	quilt: { id: 'instance.icon-editor.symbol.quilt', defaultMessage: 'Quilt' },
})

export interface SymbolOption {
	id: string
	name: MessageDescriptor
	asset: string
	category: 'loader' | 'modded' | 'vanilla'
	excludeFromRandomization?: boolean
}

export const backgroundOptions = [
	{
		id: 'rose',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#D62E63',
			bottom_color: '#F95C62',
		},
		name: names.rose,
	},
	{
		id: 'orange',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#FF8D29',
			bottom_color: '#FFB452',
		},
		name: names.orange,
	},
	{
		id: 'yellow',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#FFC629',
			bottom_color: '#FFEE53',
		},
		name: names.yellow,
	},
	{
		id: 'lime',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#6FDA1D',
			bottom_color: '#CBFF50',
		},
		name: names.lime,
	},
	{
		id: 'green',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#0B9F21',
			bottom_color: '#4FD24B',
		},
		name: names.green,
	},
	// {
	// 	id: 'dark_green',
	// 	background: {
	// 		type: 'linear-top-down-gradient',
	// 		top_color: '#084C13',
	// 		bottom_color: '#327735',
	// 	},
	// 	name: names.darkGreen,
	// },
	// {
	// 	id: 'indigo',
	// 	background: {
	// 		type: 'linear-top-down-gradient',
	// 		top_color: '#3F00D3',
	// 		bottom_color: '#2659FE',
	// 	},
	// 	name: names.indigo,
	// },
	{
		id: 'purple',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#4739FF',
			bottom_color: '#6670FF',
		},
		name: names.purple,
	},
	{
		id: 'blue',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#227EFF',
			bottom_color: '#5EC1FF',
		},
		name: names.blue,
	},
	{
		id: 'lavender',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#C056FD',
			bottom_color: '#B889FF',
		},
		name: names.lavender,
	},
	{
		id: 'pink',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#F640C0',
			bottom_color: '#FF7BF1',
		},
		name: names.pink,
	},

	// {
	// 	id: 'red',
	// 	background: {
	// 		type: 'linear-top-down-gradient',
	// 		top_color: '#F6111C',
	// 		bottom_color: '#F94548',
	// 	},
	// 	name: names.red,
	// },
	{
		id: 'light_gray',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#AEAEAE',
			bottom_color: '#D9D9D9',
		},
		name: names.lightGray,
	},
	{
		id: 'gray',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#373C4C',
			bottom_color: '#4C4F58',
		},
		name: names.gray,
	},
	{
		id: 'dark_gray',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#1B1D29',
			bottom_color: '#252731',
		},
		name: names.darkGray,
	},
] as const

export const symbolOptions = [
	/////////////////////////
	// loaders
	/////////////////////////

	{
		id: 'fabric',
		name: names.fabric,
		asset: fabric,
		category: 'loader',
		excludeFromRandomization: true,
	},
	{
		id: 'forge',
		name: names.forge,
		asset: forge,
		category: 'loader',
		excludeFromRandomization: true,
	},
	{
		id: 'neoforge',
		name: names.neoForge,
		asset: neoForge,
		category: 'loader',
		excludeFromRandomization: true,
	},
	{
		id: 'quilt',
		name: names.quilt,
		asset: quilt,
		category: 'loader',
		excludeFromRandomization: true,
	},

	// Cobblemon: Poké Ball
	{ id: 'poke_ball', name: names.pokeBall, asset: pokeBall, category: 'modded' },

	// Origins: Orb of Origins
	{ id: 'orb', name: names.orb, asset: orb, category: 'modded' },

	// Farmer's Delight: Cooking Pot, Skillet
	{ id: 'cooking_pot', name: names.cookingPot, asset: cookingPot, category: 'modded' },
	{ id: 'skillet', name: names.skillet, asset: skillet, category: 'modded' },

	// Supplementaries: Globe, Pancakes
	{ id: 'globe', name: names.globe, asset: globe, category: 'modded' },
	{ id: 'pancakes', name: names.pancakes, asset: pancakes, category: 'modded' },

	// Sophisticated Backpacks: Backpack
	{ id: 'backpack', name: names.backpack, asset: backpack, category: 'modded' },

	// Chipped: Chair
	{ id: 'couch', name: names.couch, asset: couch, category: 'modded' },

	// Botania: Tiny Potato
	{ id: 'tiny_potato', name: names.tinyPotato, asset: tinyPotato, category: 'modded' },

	// Blåhaj: Blue Shark
	{ id: 'blue_shark', name: names.blueShark, asset: blueShark, category: 'modded' },

	// Other modded symbols: Brown Bear, Moobloom
	{ id: 'brown_bear', name: names.brownBear, asset: brownBear, category: 'modded' },
	{ id: 'moobloom', name: names.moobloom, asset: moobloom, category: 'modded' },

	// Create: Wrench, Cogwheel
	{ id: 'create_wrench', name: names.wrench, asset: wrench, category: 'modded' },
	{ id: 'cogwheel', name: names.cogwheel, asset: cogwheel, category: 'modded' },

	// Create Aeronautics: Engine, Tire
	{ id: 'engine', name: names.engine, asset: engine, category: 'modded' },
	{ id: 'tire', name: names.tire, asset: tire, category: 'modded' },

	// Ad Astra: Oxygen Distributor, Space Helmet
	{
		id: 'oxygen_distributor',
		name: names.oxygenDistributor,
		asset: oxygenDistributor,
		category: 'modded',
	},
	{ id: 'space_helmet', name: names.spaceHelmet, asset: spaceHelmet, category: 'modded' },

	// Miscellaneous: Gizmo, Terminal
	{ id: 'gizmo', name: names.gizmo, asset: gizmo, category: 'modded' },
	{ id: 'terminal', name: names.terminal, asset: terminal, category: 'modded' },

	// Miscellaneous: Modrinth Wrench, Mr Pack
	{ id: 'wrenth_rinth', name: names.wrenchRinth, asset: wrenchRinth, category: 'modded' },
	{ id: 'mr_pack', name: names.mrPack, asset: mrPack, category: 'modded' },

	/////////////////////////
	// vanilla ones
	/////////////////////////

	{ id: 'grass_block', name: names.grassBlock, asset: grassBlock, category: 'vanilla' },

	{ id: 'crafting_table', name: names.craftingTable, asset: craftingTable, category: 'vanilla' },
	{ id: 'furnace', name: names.furnace, asset: furnace, category: 'vanilla' },
	{ id: 'chest', name: names.chest, asset: chest, category: 'vanilla' },

	{ id: 'bookshelf', name: names.bookshelf, asset: bookshelf, category: 'vanilla' },
	{
		id: 'redstone_block',
		name: names.redstoneBlock,
		asset: redstoneBlock,
		category: 'vanilla',
	},
	{
		id: 'sticky_piston',
		name: names.stickyPiston,
		asset: stickyPiston,
		category: 'vanilla',
	},
	{ id: 'slime_block', name: names.slimeBlock, asset: slimeBlock, category: 'vanilla' },
	{ id: 'cake', name: names.cake, asset: cake, category: 'vanilla' },
	{ id: 'campfire', name: names.campfire, asset: campfire, category: 'vanilla' },
	{ id: 'pickaxe', name: names.pickaxe, asset: pickaxe, category: 'vanilla' },
	{ id: 'sword', name: names.sword, asset: sword, category: 'vanilla' },
	{ id: 'zombie', name: names.zombie, asset: zombie, category: 'vanilla' },
	{ id: 'creeper', name: names.creeper, asset: creeper, category: 'vanilla' },
	{ id: 'skeleton', name: names.skeleton, asset: skeleton, category: 'vanilla' },
	{ id: 'ender_dragon', name: names.enderDragon, asset: enderDragon, category: 'vanilla' },
	{ id: 'ender_chest', name: names.enderChest, asset: enderChest, category: 'vanilla' },
	{ id: 'sculk_sensor', name: names.sculkSensor, asset: sculkSensor, category: 'vanilla' },
	{ id: 'beacon', name: names.beacon, asset: beacon, category: 'vanilla' },
	{
		id: 'enchanting_table',
		name: names.enchantingTable,
		asset: enchantingTable,
		category: 'vanilla',
	},
	{ id: 'lantern', name: names.lantern, asset: lantern, category: 'vanilla' },
	{ id: 'tnt', name: names.tnt, asset: tnt, category: 'vanilla' },
	{ id: 'command_block', name: names.commandBlock, asset: commandBlock, category: 'vanilla' },
] as const satisfies readonly SymbolOption[]

export type BackgroundId = (typeof backgroundOptions)[number]['id']
export type SymbolId = (typeof symbolOptions)[number]['id']

export const RANDOM_CONFIG_BLACKLIST = [
	{ background: 'purple', symbol: 'globe' },
	{ background: 'blue', symbol: 'globe' },
	{ background: 'gray', symbol: 'cogwheel' },
	{ background: 'dark_gray', symbol: 'cogwheel' },
	{ background: 'rose', symbol: 'poke_ball' },
	{ background: 'lime', symbol: 'slime_block' },
	{ background: 'green', symbol: 'slime_block' },
	{ background: 'rose', symbol: 'redstone_block' },
	{ background: 'rose', symbol: 'couch' },
	{ background: 'orange', symbol: 'space_helmet' },
	{ background: 'rose', symbol: 'tnt' },
	{ background: 'yellow', symbol: 'moobloom' },
	{ background: 'green', symbol: 'wrenth_rinth' },
	{ background: 'lime', symbol: 'mr_pack' },
	{ background: 'light_gray', symbol: 'skillet' },
	{ background: 'light_gray', symbol: 'cooking_pot' },
] satisfies readonly { background: BackgroundId; symbol: SymbolId }[]

export const DEFAULT_BACKGROUND_ID = 'purple' satisfies BackgroundId
export const DEFAULT_SYMBOL_ID = 'grass_block' satisfies SymbolId
