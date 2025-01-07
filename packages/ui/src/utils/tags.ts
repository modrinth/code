import type { Component } from 'vue'
import { defineMessage, type MessageDescriptor } from '@vintl/vintl'
import {
  BukkitIcon,
  BungeeCordIcon,
  CanvasIcon,
  FabricIcon,
  FoliaIcon,
  ForgeIcon,
  IrisIcon,
  LiteLoaderIcon,
  ModLoaderIcon,
  NeoForgeIcon,
  OptiFineIcon,
  PaperIcon,
  PurpurIcon,
  QuiltIcon,
  RiftIcon,
  SpigotIcon,
  SpongeIcon,
  VanillaIcon,
  VelocityIcon,
  WaterfallIcon,

  AdventureIcon,
  AtmosphereIcon,
  AudioIcon,
  BlocksIcon,
  BloomIcon,
  CartoonIcon,
  ChallengingIcon,
  ColoredLightingIcon,
  CombatIcon,
  CoreShadersIcon,
  CursedIcon,
  DecorationIcon,
  EconomyIcon,
  EntitiesIcon,
  EnvironmentIcon,
  EquipmentIcon,
  FantasyIcon,
  FoliageIcon,
  FontsIcon,
  FoodIcon,
  GameMechanicsIcon,
  GuiIcon,
  HighIcon,
  ItemsIcon,
  KitchenSinkIcon,
  BookIcon,
  LightweightIcon,
  LocaleIcon,
  LowIcon,
  MagicIcon,
  ManagementIcon,
  MediumIcon,
  MinigameIcon,
  MobsIcon,
  ModdedIcon,
  ModelsIcon,
  MultiplayerIcon,
  OptimizationIcon,
  PathTracingIcon,
  PbrIcon,
  PotatoIcon,
  QuestsIcon,
  RealisticIcon,
  ReflectionsIcon,
  ScreenshotIcon,
  SemiRealisticIcon,
  ShadowsIcon,
  SimplisticIcon,
  SocialIcon,
  StorageIcon,
  TechnologyIcon,
  ThemedIcon,
  TransportationIcon,
  TweaksIcon,
  UtilityIcon,
  VanillaLikeIcon,
  WorldgenIcon,
} from '@modrinth/assets'

type TagMetadata = {
  type: 'platform' | 'category'
  message: MessageDescriptor
  icon?: Component
}

const PLATFORM_METADATA: Record<string, TagMetadata> = {
  bukkit: {
    type: 'platform',
    icon: BukkitIcon,
    message: defineMessage({ id: 'tags.platform.bukkit.name', defaultMessage: 'Bukkit' }),
  },
  bungeecord: {
    type: 'platform',
    icon: BungeeCordIcon,
    message: defineMessage({ id: 'tags.platform.bungeecord.name', defaultMessage: 'BungeeCord' }),
  },
  canvas: {
    type: 'platform',
    icon: CanvasIcon,
    message: defineMessage({ id: 'tags.platform.canvas.name', defaultMessage: 'Canvas' }),
  },
  datapack: {
    type: 'platform',
    icon: VanillaIcon,
    message: defineMessage({ id: 'tags.platform.datapack.name', defaultMessage: 'Data Pack' }),
  },
  fabric: {
    type: 'platform',
    icon: FabricIcon,
    message: defineMessage({ id: 'tags.platform.fabric.name', defaultMessage: 'Fabric' }),
  },
  folia: {
    type: 'platform',
    icon: FoliaIcon,
    message: defineMessage({ id: 'tags.platform.folia.name', defaultMessage: 'Folia' }),
  },
  forge: {
    type: 'platform',
    icon: ForgeIcon,
    message: defineMessage({ id: 'tags.platform.forge.name', defaultMessage: 'Forge' }),
  },
  iris: {
    type: 'platform',
    icon: IrisIcon,
    message: defineMessage({ id: 'tags.platform.iris.name', defaultMessage: 'Iris' }),
  },
  liteloader: {
    type: 'platform',
    icon: LiteLoaderIcon,
    message: defineMessage({ id: 'tags.platform.liteloader.name', defaultMessage: 'LiteLoader' }),
  },
  minecraft: {
    type: 'platform',
    icon: VanillaIcon,
    message: defineMessage({ id: 'tags.platform.minecraft.name', defaultMessage: 'Resource Pack' }),
  },
  modloader: {
    type: 'platform',
    icon: ModLoaderIcon,
    message: defineMessage({
      id: 'tags.platform.modloader.name',
      defaultMessage: `Risugami's ModLoader`,
    }),
  },
  neoforge: {
    type: 'platform',
    icon: NeoForgeIcon,
    message: defineMessage({ id: 'tags.platform.neoforge.name', defaultMessage: 'NeoForge' }),
  },
  optifine: {
    type: 'platform',
    icon: OptiFineIcon,
    message: defineMessage({ id: 'tags.platform.optifine.name', defaultMessage: 'OptiFine' }),
  },
  paper: {
    type: 'platform',
    icon: PaperIcon,
    message: defineMessage({ id: 'tags.platform.paper.name', defaultMessage: 'Paper' }),
  },
  purpur: {
    type: 'platform',
    icon: PurpurIcon,
    message: defineMessage({ id: 'tags.platform.purpur.name', defaultMessage: 'Purpur' }),
  },
  quilt: {
    type: 'platform',
    icon: QuiltIcon,
    message: defineMessage({ id: 'tags.platform.quilt.name', defaultMessage: 'Quilt' }),
  },
  rift: {
    type: 'platform',
    icon: RiftIcon,
    message: defineMessage({ id: 'tags.platform.rift.name', defaultMessage: 'Rift' }),
  },
  spigot: {
    type: 'platform',
    icon: SpigotIcon,
    message: defineMessage({ id: 'tags.platform.spigot.name', defaultMessage: 'Spigot' }),
  },
  sponge: {
    type: 'platform',
    icon: SpongeIcon,
    message: defineMessage({ id: 'tags.platform.sponge.name', defaultMessage: 'Sponge' }),
  },
  vanilla: {
    type: 'platform',
    icon: VanillaIcon,
    message: defineMessage({ id: 'tags.platform.vanilla.name', defaultMessage: 'Vanilla' }),
  },
  velocity: {
    type: 'platform',
    icon: VelocityIcon,
    message: defineMessage({ id: 'tags.platform.velocity.name', defaultMessage: 'Velocity' }),
  },
  waterfall: {
    type: 'platform',
    icon: WaterfallIcon,
    message: defineMessage({ id: 'tags.platform.waterfall.name', defaultMessage: 'Waterfall' }),
  },
}

const CATEGORY_METADATA: Record<string, TagMetadata> = {
  adventure: {
    type: 'category',
    icon: AdventureIcon,
    message: defineMessage({ id: 'tags.category.adventure.name', defaultMessage: 'Adventure' }),
  },
  atmosphere: {
    type: 'category',
    icon: AtmosphereIcon,
    message: defineMessage({ id: 'tags.category.atmosphere.name', defaultMessage: 'Atmosphere' }),
  },
  audio: {
    type: 'category',
    icon: AudioIcon,
    message: defineMessage({ id: 'tags.category.audio.name', defaultMessage: 'Audio' }),
  },
  blocks: {
    type: 'category',
    icon: BlocksIcon,
    message: defineMessage({ id: 'tags.category.blocks.name', defaultMessage: 'Blocks' }),
  },
  bloom: {
    type: 'category',
    icon: BloomIcon,
    message: defineMessage({ id: 'tags.category.bloom.name', defaultMessage: 'Bloom' }),
  },
  cartoon: {
    type: 'category',
    icon: CartoonIcon,
    message: defineMessage({ id: 'tags.category.cartoon.name', defaultMessage: 'Cartoon' }),
  },
  challenging: {
    type: 'category',
    icon: ChallengingIcon,
    message: defineMessage({ id: 'tags.category.challenging.name', defaultMessage: 'Challenging' }),
  },
  'colored-lighting': {
    type: 'category',
    icon: ColoredLightingIcon,
    message: defineMessage({
      id: 'tags.category.colored-lighting.name',
      defaultMessage: 'Colored Lighting',
    }),
  },
  combat: {
    type: 'category',
    icon: CombatIcon,
    message: defineMessage({ id: 'tags.category.combat.name', defaultMessage: 'Combat' }),
  },
  'core-shaders': {
    type: 'category',
    icon: CoreShadersIcon,
    message: defineMessage({
      id: 'tags.category.core-shaders.name',
      defaultMessage: 'Core Shaders',
    }),
  },
  cursed: {
    type: 'category',
    icon: CursedIcon,
    message: defineMessage({ id: 'tags.category.cursed.name', defaultMessage: 'Cursed' }),
  },
  decoration: {
    type: 'category',
    icon: DecorationIcon,
    message: defineMessage({ id: 'tags.category.decoration.name', defaultMessage: 'Decoration' }),
  },
  economy: {
    type: 'category',
    icon: EconomyIcon,
    message: defineMessage({ id: 'tags.category.economy.name', defaultMessage: 'Economy' }),
  },
  entities: {
    type: 'category',
    icon: EntitiesIcon,
    message: defineMessage({ id: 'tags.category.entities.name', defaultMessage: 'Entities' }),
  },
  environment: {
    type: 'category',
    icon: EnvironmentIcon,
    message: defineMessage({ id: 'tags.category.environment.name', defaultMessage: 'Environment' }),
  },
  equipment: {
    type: 'category',
    icon: EquipmentIcon,
    message: defineMessage({ id: 'tags.category.equipment.name', defaultMessage: 'Equipment' }),
  },
  fantasy: {
    type: 'category',
    icon: FantasyIcon,
    message: defineMessage({ id: 'tags.category.fantasy.name', defaultMessage: 'Fantasy' }),
  },
  foliage: {
    type: 'category',
    icon: FoliageIcon,
    message: defineMessage({ id: 'tags.category.foliage.name', defaultMessage: 'Foliage' }),
  },
  fonts: {
    type: 'category',
    icon: FontsIcon,
    message: defineMessage({ id: 'tags.category.fonts.name', defaultMessage: 'Fonts' }),
  },
  food: {
    type: 'category',
    icon: FoodIcon,
    message: defineMessage({ id: 'tags.category.food.name', defaultMessage: 'Food' }),
  },
  'game-mechanics': {
    type: 'category',
    icon: GameMechanicsIcon,
    message: defineMessage({
      id: 'tags.category.game-mechanics.name',
      defaultMessage: 'Game Mechanics',
    }),
  },
  gui: {
    type: 'category',
    icon: GuiIcon,
    message: defineMessage({ id: 'tags.category.gui.name', defaultMessage: 'GUI' }),
  },
  high: {
    type: 'category',
    icon: HighIcon,
    message: defineMessage({ id: 'tags.category.high.name', defaultMessage: 'High' }),
  },
  items: {
    type: 'category',
    icon: ItemsIcon,
    message: defineMessage({ id: 'tags.category.items.name', defaultMessage: 'Items' }),
  },
  'kitchen-sink': {
    type: 'category',
    icon: KitchenSinkIcon,
    message: defineMessage({
      id: 'tags.category.kitchen-sink.name',
      defaultMessage: 'Kitchen Sink',
    }),
  },
  library: {
    type: 'category',
    icon: BookIcon,
    message: defineMessage({ id: 'tags.category.library.name', defaultMessage: 'Library' }),
  },
  lightweight: {
    type: 'category',
    icon: LightweightIcon,
    message: defineMessage({ id: 'tags.category.lightweight.name', defaultMessage: 'Lightweight' }),
  },
  locale: {
    type: 'category',
    icon: LocaleIcon,
    message: defineMessage({ id: 'tags.category.locale.name', defaultMessage: 'Locale' }),
  },
  low: {
    type: 'category',
    icon: LowIcon,
    message: defineMessage({ id: 'tags.category.low.name', defaultMessage: 'Low' }),
  },
  magic: {
    type: 'category',
    icon: MagicIcon,
    message: defineMessage({ id: 'tags.category.magic.name', defaultMessage: 'Magic' }),
  },
  management: {
    type: 'category',
    icon: ManagementIcon,
    message: defineMessage({ id: 'tags.category.management.name', defaultMessage: 'Management' }),
  },
  medium: {
    type: 'category',
    icon: MediumIcon,
    message: defineMessage({ id: 'tags.category.medium.name', defaultMessage: 'Medium' }),
  },
  minigame: {
    type: 'category',
    icon: MinigameIcon,
    message: defineMessage({ id: 'tags.category.minigame.name', defaultMessage: 'Minigame' }),
  },
  mobs: {
    type: 'category',
    icon: MobsIcon,
    message: defineMessage({ id: 'tags.category.mobs.name', defaultMessage: 'Mobs' }),
  },
  modded: {
    type: 'category',
    icon: ModdedIcon,
    message: defineMessage({ id: 'tags.category.modded.name', defaultMessage: 'Modded' }),
  },
  models: {
    type: 'category',
    icon: ModelsIcon,
    message: defineMessage({ id: 'tags.category.models.name', defaultMessage: 'Models' }),
  },
  multiplayer: {
    type: 'category',
    icon: MultiplayerIcon,
    message: defineMessage({ id: 'tags.category.multiplayer.name', defaultMessage: 'Multiplayer' }),
  },
  optimization: {
    type: 'category',
    icon: OptimizationIcon,
    message: defineMessage({
      id: 'tags.category.optimization.name',
      defaultMessage: 'Optimization',
    }),
  },
  'path-tracing': {
    type: 'category',
    icon: PathTracingIcon,
    message: defineMessage({
      id: 'tags.category.path-tracing.name',
      defaultMessage: 'Path Tracing',
    }),
  },
  pbr: {
    type: 'category',
    icon: PbrIcon,
    message: defineMessage({ id: 'tags.category.pbr.name', defaultMessage: 'PBR' }),
  },
  potato: {
    type: 'category',
    icon: PotatoIcon,
    message: defineMessage({ id: 'tags.category.potato.name', defaultMessage: 'Potato' }),
  },
  quests: {
    type: 'category',
    icon: QuestsIcon,
    message: defineMessage({ id: 'tags.category.quests.name', defaultMessage: 'Quests' }),
  },
  realistic: {
    type: 'category',
    icon: RealisticIcon,
    message: defineMessage({ id: 'tags.category.realistic.name', defaultMessage: 'Realistic' }),
  },
  reflections: {
    type: 'category',
    icon: ReflectionsIcon,
    message: defineMessage({ id: 'tags.category.reflections.name', defaultMessage: 'Reflections' }),
  },
  screenshot: {
    type: 'category',
    icon: ScreenshotIcon,
    message: defineMessage({ id: 'tags.category.screenshot.name', defaultMessage: 'Screenshot' }),
  },
  'semi-realistic': {
    type: 'category',
    icon: SemiRealisticIcon,
    message: defineMessage({
      id: 'tags.category.semi-realistic.name',
      defaultMessage: 'Semi-realistic',
    }),
  },
  shadows: {
    type: 'category',
    icon: ShadowsIcon,
    message: defineMessage({ id: 'tags.category.shadows.name', defaultMessage: 'Shadows' }),
  },
  simplistic: {
    type: 'category',
    icon: SimplisticIcon,
    message: defineMessage({ id: 'tags.category.simplistic.name', defaultMessage: 'Simplistic' }),
  },
  social: {
    type: 'category',
    icon: SocialIcon,
    message: defineMessage({ id: 'tags.category.social.name', defaultMessage: 'Social' }),
  },
  storage: {
    type: 'category',
    icon: StorageIcon,
    message: defineMessage({ id: 'tags.category.storage.name', defaultMessage: 'Storage' }),
  },
  technology: {
    type: 'category',
    icon: TechnologyIcon,
    message: defineMessage({ id: 'tags.category.technology.name', defaultMessage: 'Technology' }),
  },
  themed: {
    type: 'category',
    icon: ThemedIcon,
    message: defineMessage({ id: 'tags.category.themed.name', defaultMessage: 'Themed' }),
  },
  transportation: {
    type: 'category',
    icon: TransportationIcon,
    message: defineMessage({
      id: 'tags.category.transportation.name',
      defaultMessage: 'Transportation',
    }),
  },
  tweaks: {
    type: 'category',
    icon: TweaksIcon,
    message: defineMessage({ id: 'tags.category.tweaks.name', defaultMessage: 'Tweaks' }),
  },
  utility: {
    type: 'category',
    icon: UtilityIcon,
    message: defineMessage({ id: 'tags.category.utility.name', defaultMessage: 'Utility' }),
  },
  'vanilla-like': {
    type: 'category',
    icon: VanillaLikeIcon,
    message: defineMessage({
      id: 'tags.category.vanilla-like.name',
      defaultMessage: 'Vanilla-like',
    }),
  },
  worldgen: {
    type: 'category',
    icon: WorldgenIcon,
    message: defineMessage({
      id: 'tags.category.worldgen.name',
      defaultMessage: 'World Generation',
    }),
  },
  '8x-': {
    type: 'category',
    message: defineMessage({ id: 'tags.category.8x-.name', defaultMessage: '8x or lower' }),
  },
  '16x': {
    type: 'category',
    message: defineMessage({ id: 'tags.category.16x.name', defaultMessage: '16x' }),
  },
  '32x': {
    type: 'category',
    message: defineMessage({ id: 'tags.category.32x.name', defaultMessage: '32x' }),
  },
  '48x': {
    type: 'category',
    message: defineMessage({ id: 'tags.category.48x.name', defaultMessage: '48x' }),
  },
  '64x': {
    type: 'category',
    message: defineMessage({ id: 'tags.category.64x.name', defaultMessage: '64x' }),
  },
  '128x': {
    type: 'category',
    message: defineMessage({ id: 'tags.category.128x.name', defaultMessage: '128x' }),
  },
  '256x': {
    type: 'category',
    message: defineMessage({ id: 'tags.category.256x.name', defaultMessage: '256x' }),
  },
  '512x+': {
    type: 'category',
    message: defineMessage({ id: 'tags.category.512x+.name', defaultMessage: '512x or higher' }),
  },
}

export function getPlatformMessage(tag: string): MessageDescriptor | string {
  if (PLATFORM_METADATA[tag]) {
    return PLATFORM_METADATA[tag].message
  }
  return `tags.platform.${tag}.name`
}

export function getCategoryMessage(tag: string): MessageDescriptor | string {
  if (CATEGORY_METADATA[tag]) {
    return CATEGORY_METADATA[tag].message
  }
  return `tags.category.${tag}.name`
}

export function getTagMessage(tag: string): MessageDescriptor | string {
  if (PLATFORM_METADATA[tag]) {
    return PLATFORM_METADATA[tag].message
  }
  if (CATEGORY_METADATA[tag]) {
    return CATEGORY_METADATA[tag].message
  }

  return `tags.unknown.${tag}.name`
}

export function getPlatformIcon(tag: string): Component | undefined {
  if (PLATFORM_METADATA[tag]) {
    return PLATFORM_METADATA[tag].icon
  }
  return undefined
}

export function getCategoryIcon(tag: string): Component | undefined {
  if (CATEGORY_METADATA[tag]) {
    return CATEGORY_METADATA[tag].icon
  }
  return undefined
}

export function getTagIcon(tag: string): Component | undefined {
  return getPlatformIcon(tag) ?? getCategoryIcon(tag)
}

export function isPlatformTag(tag: string): boolean {
  return PLATFORM_METADATA[tag] !== undefined
}

export function isCategoryTag(tag: string): boolean {
  return CATEGORY_METADATA[tag] !== undefined
}
