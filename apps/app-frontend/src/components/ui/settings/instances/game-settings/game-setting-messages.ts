import { defineMessages, type MessageDescriptor, type VIntlFormatters } from '@modrinth/ui'

import type {
	EditableGameSetting,
	GameOptionCompatibilityBucket,
	GameOptionValidationError,
	GameOptionsSourceCandidate,
	GameOptionsSourceDisabledReason,
	GameSettingCategory,
} from '@/helpers/game-options'

type FormatMessage = VIntlFormatters['formatMessage']

const settingMessages = defineMessages({
	fovLabel: { id: 'app.settings.game-options.setting.fov.label', defaultMessage: 'Field of view' },
	fovDescription: { id: 'app.settings.game-options.setting.fov.description', defaultMessage: 'The camera field of view.' },
	graphicsLabel: { id: 'app.settings.game-options.setting.graphics.label', defaultMessage: 'Graphics' },
	graphicsDescription: { id: 'app.settings.game-options.setting.graphics.description', defaultMessage: 'The overall graphics quality preset.' },
	ambientOcclusionLabel: { id: 'app.settings.game-options.setting.ambient-occlusion.label', defaultMessage: 'Smooth lighting' },
	ambientOcclusionDescription: { id: 'app.settings.game-options.setting.ambient-occlusion.description', defaultMessage: 'Smooth lighting on block faces.' },
	renderDistanceLabel: { id: 'app.settings.game-options.setting.render-distance.label', defaultMessage: 'Render distance' },
	renderDistanceDescription: { id: 'app.settings.game-options.setting.render-distance.description', defaultMessage: 'How far terrain is rendered.' },
	simulationDistanceLabel: { id: 'app.settings.game-options.setting.simulation-distance.label', defaultMessage: 'Simulation distance' },
	simulationDistanceDescription: { id: 'app.settings.game-options.setting.simulation-distance.description', defaultMessage: 'How far the world is simulated.' },
	guiScaleLabel: { id: 'app.settings.game-options.setting.gui-scale.label', defaultMessage: 'GUI scale' },
	guiScaleDescription: { id: 'app.settings.game-options.setting.gui-scale.description', defaultMessage: 'The size of the game interface.' },
	particlesLabel: { id: 'app.settings.game-options.setting.particles.label', defaultMessage: 'Particles' },
	particlesDescription: { id: 'app.settings.game-options.setting.particles.description', defaultMessage: 'How many particles are shown.' },
	cloudsLabel: { id: 'app.settings.game-options.setting.clouds.label', defaultMessage: 'Clouds' },
	cloudsDescription: { id: 'app.settings.game-options.setting.clouds.description', defaultMessage: 'The cloud rendering quality.' },
	entityShadowsLabel: { id: 'app.settings.game-options.setting.entity-shadows.label', defaultMessage: 'Entity shadows' },
	entityShadowsDescription: { id: 'app.settings.game-options.setting.entity-shadows.description', defaultMessage: 'Show shadows beneath entities.' },
	viewBobbingLabel: { id: 'app.settings.game-options.setting.view-bobbing.label', defaultMessage: 'View bobbing' },
	viewBobbingDescription: { id: 'app.settings.game-options.setting.view-bobbing.description', defaultMessage: 'Move the camera while walking.' },
	vsyncLabel: { id: 'app.settings.game-options.setting.vsync.label', defaultMessage: 'VSync' },
	vsyncDescription: { id: 'app.settings.game-options.setting.vsync.description', defaultMessage: 'Synchronize frames with the display.' },
	fullscreenLabel: { id: 'app.settings.game-options.setting.fullscreen.label', defaultMessage: 'Fullscreen' },
	fullscreenDescription: { id: 'app.settings.game-options.setting.fullscreen.description', defaultMessage: 'Start the game in fullscreen.' },
	maxFramerateLabel: { id: 'app.settings.game-options.setting.max-framerate.label', defaultMessage: 'Maximum framerate' },
	maxFramerateDescription: { id: 'app.settings.game-options.setting.max-framerate.description', defaultMessage: 'The maximum rendered frames per second.' },
	mipmapLevelsLabel: { id: 'app.settings.game-options.setting.mipmap-levels.label', defaultMessage: 'Mipmap levels' },
	mipmapLevelsDescription: { id: 'app.settings.game-options.setting.mipmap-levels.description', defaultMessage: 'Texture smoothing at a distance.' },
	biomeBlendRadiusLabel: { id: 'app.settings.game-options.setting.biome-blend-radius.label', defaultMessage: 'Biome blend' },
	biomeBlendRadiusDescription: { id: 'app.settings.game-options.setting.biome-blend-radius.description', defaultMessage: 'How smoothly biome colors blend.' },
	languageLabel: { id: 'app.settings.game-options.setting.language.label', defaultMessage: 'Language' },
	languageDescription: { id: 'app.settings.game-options.setting.language.description', defaultMessage: 'The language used by Minecraft.' },
	masterVolumeLabel: { id: 'app.settings.game-options.setting.master-volume.label', defaultMessage: 'Master volume' },
	masterVolumeDescription: { id: 'app.settings.game-options.setting.master-volume.description', defaultMessage: 'Overall game volume.' },
	musicVolumeLabel: { id: 'app.settings.game-options.setting.music-volume.label', defaultMessage: 'Music' },
	musicVolumeDescription: { id: 'app.settings.game-options.setting.music-volume.description', defaultMessage: 'Music volume.' },
	musicToastLabel: { id: 'app.settings.game-options.setting.music-toast.label', defaultMessage: 'Music notification' },
	musicToastDescription: { id: 'app.settings.game-options.setting.music-toast.description', defaultMessage: 'Choose when Minecraft pauses music and shows its title.' },
	blocksVolumeLabel: { id: 'app.settings.game-options.setting.blocks-volume.label', defaultMessage: 'Blocks' },
	blocksVolumeDescription: { id: 'app.settings.game-options.setting.blocks-volume.description', defaultMessage: 'Block sound volume.' },
	hostileVolumeLabel: { id: 'app.settings.game-options.setting.hostile-volume.label', defaultMessage: 'Hostile creatures' },
	hostileVolumeDescription: { id: 'app.settings.game-options.setting.hostile-volume.description', defaultMessage: 'Hostile creature volume.' },
	playersVolumeLabel: { id: 'app.settings.game-options.setting.players-volume.label', defaultMessage: 'Players' },
	playersVolumeDescription: { id: 'app.settings.game-options.setting.players-volume.description', defaultMessage: 'Player sound volume.' },
	voiceVolumeLabel: { id: 'app.settings.game-options.setting.voice-volume.label', defaultMessage: 'Voice and speech' },
	voiceVolumeDescription: { id: 'app.settings.game-options.setting.voice-volume.description', defaultMessage: 'Voice and speech volume.' },
	sensitivityLabel: { id: 'app.settings.game-options.setting.sensitivity.label', defaultMessage: 'Mouse sensitivity' },
	sensitivityDescription: { id: 'app.settings.game-options.setting.sensitivity.description', defaultMessage: 'Mouse look sensitivity.' },
	invertMouseLabel: { id: 'app.settings.game-options.setting.invert-mouse.label', defaultMessage: 'Invert mouse' },
	invertMouseDescription: { id: 'app.settings.game-options.setting.invert-mouse.description', defaultMessage: 'Invert vertical mouse movement.' },
	autoJumpLabel: { id: 'app.settings.game-options.setting.auto-jump.label', defaultMessage: 'Auto-jump' },
	autoJumpDescription: { id: 'app.settings.game-options.setting.auto-jump.description', defaultMessage: 'Automatically jump at small obstacles.' },
	toggleCrouchLabel: { id: 'app.settings.game-options.setting.toggle-crouch.label', defaultMessage: 'Toggle crouch' },
	toggleCrouchDescription: { id: 'app.settings.game-options.setting.toggle-crouch.description', defaultMessage: 'Press once to remain crouched.' },
	toggleSprintLabel: { id: 'app.settings.game-options.setting.toggle-sprint.label', defaultMessage: 'Toggle sprint' },
	toggleSprintDescription: { id: 'app.settings.game-options.setting.toggle-sprint.description', defaultMessage: 'Press once to remain sprinting.' },
	discreteMouseScrollLabel: { id: 'app.settings.game-options.setting.discrete-mouse-scroll.label', defaultMessage: 'Discrete scrolling' },
	discreteMouseScrollDescription: { id: 'app.settings.game-options.setting.discrete-mouse-scroll.description', defaultMessage: 'Use discrete mouse-wheel steps.' },
	keyForwardLabel: { id: 'app.settings.game-options.setting.key-forward.label', defaultMessage: 'Move forward' },
	keyForwardDescription: { id: 'app.settings.game-options.setting.key-forward.description', defaultMessage: 'The key used to move forward.' },
	keyLeftLabel: { id: 'app.settings.game-options.setting.key-left.label', defaultMessage: 'Strafe left' },
	keyLeftDescription: { id: 'app.settings.game-options.setting.key-left.description', defaultMessage: 'The key used to move left.' },
	keyBackLabel: { id: 'app.settings.game-options.setting.key-back.label', defaultMessage: 'Move backward' },
	keyBackDescription: { id: 'app.settings.game-options.setting.key-back.description', defaultMessage: 'The key used to move backward.' },
	keyRightLabel: { id: 'app.settings.game-options.setting.key-right.label', defaultMessage: 'Strafe right' },
	keyRightDescription: { id: 'app.settings.game-options.setting.key-right.description', defaultMessage: 'The key used to move right.' },
	keyJumpLabel: { id: 'app.settings.game-options.setting.key-jump.label', defaultMessage: 'Jump' },
	keyJumpDescription: { id: 'app.settings.game-options.setting.key-jump.description', defaultMessage: 'The key used to jump.' },
	keySneakLabel: { id: 'app.settings.game-options.setting.key-sneak.label', defaultMessage: 'Sneak' },
	keySneakDescription: { id: 'app.settings.game-options.setting.key-sneak.description', defaultMessage: 'The key used to sneak.' },
	keySprintLabel: { id: 'app.settings.game-options.setting.key-sprint.label', defaultMessage: 'Sprint' },
	keySprintDescription: { id: 'app.settings.game-options.setting.key-sprint.description', defaultMessage: 'The key used to sprint.' },
	keyInventoryLabel: { id: 'app.settings.game-options.setting.key-inventory.label', defaultMessage: 'Inventory' },
	keyInventoryDescription: { id: 'app.settings.game-options.setting.key-inventory.description', defaultMessage: 'The key used to open the inventory.' },
	keySwapOffhandLabel: { id: 'app.settings.game-options.setting.key-swap-offhand.label', defaultMessage: 'Swap offhand' },
	keySwapOffhandDescription: { id: 'app.settings.game-options.setting.key-swap-offhand.description', defaultMessage: 'The key used to swap the held item.' },
	keyDropLabel: { id: 'app.settings.game-options.setting.key-drop.label', defaultMessage: 'Drop item' },
	keyDropDescription: { id: 'app.settings.game-options.setting.key-drop.description', defaultMessage: 'The key used to drop an item.' },
	keyUseLabel: { id: 'app.settings.game-options.setting.key-use.label', defaultMessage: 'Use item' },
	keyUseDescription: { id: 'app.settings.game-options.setting.key-use.description', defaultMessage: 'The key used to use an item.' },
	keyAttackLabel: { id: 'app.settings.game-options.setting.key-attack.label', defaultMessage: 'Attack' },
	keyAttackDescription: { id: 'app.settings.game-options.setting.key-attack.description', defaultMessage: 'The key used to attack.' },
	keyPickItemLabel: { id: 'app.settings.game-options.setting.key-pick-item.label', defaultMessage: 'Pick block' },
	keyPickItemDescription: { id: 'app.settings.game-options.setting.key-pick-item.description', defaultMessage: 'The key used to pick a block.' },
	keyChatLabel: { id: 'app.settings.game-options.setting.key-chat.label', defaultMessage: 'Open chat' },
	keyChatDescription: { id: 'app.settings.game-options.setting.key-chat.description', defaultMessage: 'The key used to open chat.' },
	keyPlayerListLabel: { id: 'app.settings.game-options.setting.key-player-list.label', defaultMessage: 'Player list' },
	keyPlayerListDescription: { id: 'app.settings.game-options.setting.key-player-list.description', defaultMessage: 'The key used to show the player list.' },
	keyCommandLabel: { id: 'app.settings.game-options.setting.key-command.label', defaultMessage: 'Command' },
	keyCommandDescription: { id: 'app.settings.game-options.setting.key-command.description', defaultMessage: 'The key used to enter a command.' },
	keyScreenshotLabel: { id: 'app.settings.game-options.setting.key-screenshot.label', defaultMessage: 'Screenshot' },
	keyScreenshotDescription: { id: 'app.settings.game-options.setting.key-screenshot.description', defaultMessage: 'The key used to take a screenshot.' },
	keyPerspectiveLabel: { id: 'app.settings.game-options.setting.key-perspective.label', defaultMessage: 'Change perspective' },
	keyPerspectiveDescription: { id: 'app.settings.game-options.setting.key-perspective.description', defaultMessage: 'The key used to change camera perspective.' },
	keyFullscreenLabel: { id: 'app.settings.game-options.setting.key-fullscreen.label', defaultMessage: 'Toggle fullscreen' },
	keyFullscreenDescription: { id: 'app.settings.game-options.setting.key-fullscreen.description', defaultMessage: 'The key used to toggle fullscreen.' },
	keyAdvancementsLabel: { id: 'app.settings.game-options.setting.key-advancements.label', defaultMessage: 'Advancements' },
	keyAdvancementsDescription: { id: 'app.settings.game-options.setting.key-advancements.description', defaultMessage: 'The key used to open advancements.' },
	chatVisibilityLabel: { id: 'app.settings.game-options.setting.chat-visibility.label', defaultMessage: 'Chat visibility' },
	chatVisibilityDescription: { id: 'app.settings.game-options.setting.chat-visibility.description', defaultMessage: 'Choose which chat messages are visible.' },
	chatColorsLabel: { id: 'app.settings.game-options.setting.chat-colors.label', defaultMessage: 'Chat colors' },
	chatColorsDescription: { id: 'app.settings.game-options.setting.chat-colors.description', defaultMessage: 'Show colors in chat messages.' },
	chatLinksLabel: { id: 'app.settings.game-options.setting.chat-links.label', defaultMessage: 'Web links' },
	chatLinksDescription: { id: 'app.settings.game-options.setting.chat-links.description', defaultMessage: 'Allow links in chat.' },
	chatLinksPromptLabel: { id: 'app.settings.game-options.setting.chat-links-prompt.label', defaultMessage: 'Prompt on links' },
	chatLinksPromptDescription: { id: 'app.settings.game-options.setting.chat-links-prompt.description', defaultMessage: 'Ask before opening links from chat.' },
	chatOpacityLabel: { id: 'app.settings.game-options.setting.chat-opacity.label', defaultMessage: 'Chat opacity' },
	chatOpacityDescription: { id: 'app.settings.game-options.setting.chat-opacity.description', defaultMessage: 'The opacity of chat text.' },
	chatScaleLabel: { id: 'app.settings.game-options.setting.chat-scale.label', defaultMessage: 'Chat scale' },
	chatScaleDescription: { id: 'app.settings.game-options.setting.chat-scale.description', defaultMessage: 'The size of the chat interface.' },
	narratorLabel: { id: 'app.settings.game-options.setting.narrator.label', defaultMessage: 'Narrator' },
	narratorDescription: { id: 'app.settings.game-options.setting.narrator.description', defaultMessage: 'Choose what the narrator reads.' },
	subtitlesLabel: { id: 'app.settings.game-options.setting.subtitles.label', defaultMessage: 'Subtitles' },
	subtitlesDescription: { id: 'app.settings.game-options.setting.subtitles.description', defaultMessage: 'Show captions for nearby sounds.' },
	highContrastLabel: { id: 'app.settings.game-options.setting.high-contrast.label', defaultMessage: 'High contrast' },
	highContrastDescription: { id: 'app.settings.game-options.setting.high-contrast.description', defaultMessage: 'Use high-contrast resource styling.' },
	darkSplashLabel: { id: 'app.settings.game-options.setting.dark-splash.label', defaultMessage: 'Monochrome logo' },
	darkSplashDescription: { id: 'app.settings.game-options.setting.dark-splash.description', defaultMessage: 'Use a dark loading screen background.' },
	notificationTimeLabel: { id: 'app.settings.game-options.setting.notification-time.label', defaultMessage: 'Notification time' },
	notificationTimeDescription: { id: 'app.settings.game-options.setting.notification-time.description', defaultMessage: 'How long notifications remain visible.' },
	mainHandLabel: { id: 'app.settings.game-options.setting.main-hand.label', defaultMessage: 'Main hand' },
	mainHandDescription: { id: 'app.settings.game-options.setting.main-hand.description', defaultMessage: 'The hand used for held items.' },
	capeLabel: { id: 'app.settings.game-options.setting.cape.label', defaultMessage: 'Cape' },
	capeDescription: { id: 'app.settings.game-options.setting.cape.description', defaultMessage: 'Show the cape skin layer.' },
	hatLabel: { id: 'app.settings.game-options.setting.hat.label', defaultMessage: 'Hat' },
	hatDescription: { id: 'app.settings.game-options.setting.hat.description', defaultMessage: 'Show the hat skin layer.' },
	jacketLabel: { id: 'app.settings.game-options.setting.jacket.label', defaultMessage: 'Jacket' },
	jacketDescription: { id: 'app.settings.game-options.setting.jacket.description', defaultMessage: 'Show the jacket skin layer.' },
	allowServerListingLabel: { id: 'app.settings.game-options.setting.allow-server-listing.label', defaultMessage: 'Server listings' },
	allowServerListingDescription: { id: 'app.settings.game-options.setting.allow-server-listing.description', defaultMessage: 'Allow servers to list this player online.' },
	realmsNotificationsLabel: { id: 'app.settings.game-options.setting.realms-notifications.label', defaultMessage: 'Realms notifications' },
	realmsNotificationsDescription: { id: 'app.settings.game-options.setting.realms-notifications.description', defaultMessage: 'Show notifications from Realms.' },
	customDescription: { id: 'app.settings.game-options.setting.custom.description', defaultMessage: 'A setting stored in options.txt by a mod or an unrecognized client extension.' },
})

const categoryMessages = defineMessages({
	skinCustomizationLabel: { id: 'app.settings.game-options.category.skin-customization.label', defaultMessage: 'Skin customization' },
	skinCustomizationDescription: { id: 'app.settings.game-options.category.skin-customization.description', defaultMessage: 'Skin layers and main hand' },
	videoLabel: { id: 'app.settings.game-options.category.video.label', defaultMessage: 'Video' },
	videoDescription: { id: 'app.settings.game-options.category.video.description', defaultMessage: 'Camera and display settings' },
	languageLabel: { id: 'app.settings.game-options.category.language.label', defaultMessage: 'Language' },
	languageDescription: { id: 'app.settings.game-options.category.language.description', defaultMessage: 'Game language' },
	musicAndSoundLabel: { id: 'app.settings.game-options.category.music-and-sound.label', defaultMessage: 'Music and sound' },
	musicAndSoundDescription: { id: 'app.settings.game-options.category.music-and-sound.description', defaultMessage: 'Volume and audio preferences' },
	controlsLabel: { id: 'app.settings.game-options.category.controls.label', defaultMessage: 'Controls' },
	controlsDescription: { id: 'app.settings.game-options.category.controls.description', defaultMessage: 'Mouse, movement, and key bindings' },
	chatLabel: { id: 'app.settings.game-options.category.chat.label', defaultMessage: 'Chat' },
	chatDescription: { id: 'app.settings.game-options.category.chat.description', defaultMessage: 'Chat visibility and appearance' },
	accessibilityLabel: { id: 'app.settings.game-options.category.accessibility.label', defaultMessage: 'Accessibility' },
	accessibilityDescription: { id: 'app.settings.game-options.category.accessibility.description', defaultMessage: 'Accessibility preferences' },
	onlineLabel: { id: 'app.settings.game-options.category.online.label', defaultMessage: 'Online' },
	onlineDescription: { id: 'app.settings.game-options.category.online.description', defaultMessage: 'Online and Realms preferences' },
	customLabel: { id: 'app.settings.game-options.category.custom.label', defaultMessage: 'Custom settings' },
	customDescription: { id: 'app.settings.game-options.category.custom.description', defaultMessage: 'Settings added by mods or unrecognized clients' },
})

const choiceMessages = defineMessages({
	fast: { id: 'app.settings.game-options.choice.fast', defaultMessage: 'Fast' },
	fancy: { id: 'app.settings.game-options.choice.fancy', defaultMessage: 'Fancy' },
	fabulous: { id: 'app.settings.game-options.choice.fabulous', defaultMessage: 'Fabulous' },
	left: { id: 'app.settings.game-options.choice.left', defaultMessage: 'Left' },
	right: { id: 'app.settings.game-options.choice.right', defaultMessage: 'Right' },
	shown: { id: 'app.settings.game-options.choice.shown', defaultMessage: 'Shown' },
	commandsOnly: { id: 'app.settings.game-options.choice.commands-only', defaultMessage: 'Commands only' },
	hidden: { id: 'app.settings.game-options.choice.hidden', defaultMessage: 'Hidden' },
	all: { id: 'app.settings.game-options.choice.all', defaultMessage: 'All' },
	decreased: { id: 'app.settings.game-options.choice.decreased', defaultMessage: 'Decreased' },
	minimal: { id: 'app.settings.game-options.choice.minimal', defaultMessage: 'Minimal' },
	off: { id: 'app.settings.game-options.choice.off', defaultMessage: 'Off' },
	chat: { id: 'app.settings.game-options.choice.chat', defaultMessage: 'Chat' },
	system: { id: 'app.settings.game-options.choice.system', defaultMessage: 'System' },
	on: { id: 'app.settings.game-options.choice.on', defaultMessage: 'On' },
	minimum: { id: 'app.settings.game-options.choice.minimum', defaultMessage: 'Minimum' },
	maximum: { id: 'app.settings.game-options.choice.maximum', defaultMessage: 'Maximum' },
	never: { id: 'app.settings.game-options.choice.never', defaultMessage: 'Never' },
	pause: { id: 'app.settings.game-options.choice.pause', defaultMessage: 'Pause' },
	pauseAndToast: { id: 'app.settings.game-options.choice.pause-and-toast', defaultMessage: 'Pause and show toast' },
})

export const presentationMessages = defineMessages({
	customValuePlaceholder: { id: 'app.settings.game-options.custom-value.placeholder', defaultMessage: 'Raw options.txt value' },
	validationLocalValueNeedsSaving: { id: 'app.settings.game-options.validation.local-value-needs-saving', defaultMessage: 'Save this local value to resume syncing this setting.' },
	validationMissingValue: { id: 'app.settings.game-options.validation.missing-value', defaultMessage: 'Choose a value before enabling sync.' },
	validationNoCompatibleInstances: { id: 'app.settings.game-options.validation.no-compatible-instances', defaultMessage: 'This setting is not compatible with any participating instances.' },
	validationInvalidValue: { id: 'app.settings.game-options.validation.invalid-value', defaultMessage: 'Enter a valid value for this setting.' },
	validationChangedSinceOpened: { id: 'app.settings.game-options.validation.changed-since-opened', defaultMessage: 'Changed elsewhere. Review this setting before saving again.' },
	compatibilityNoParticipants: { id: 'app.settings.game-options.compatibility.no-participants', defaultMessage: 'Will sync when an instance participates' },
	compatibilityNone: { id: 'app.settings.game-options.compatibility.none', defaultMessage: 'Not compatible with current instances' },
	compatibilityAll: { id: 'app.settings.game-options.compatibility.all', defaultMessage: 'Syncs to all {count, plural, one {# instance} other {# instances}}' },
	compatibilityAllDisabled: { id: 'app.settings.game-options.compatibility.all-disabled', defaultMessage: 'Would sync to all {count, plural, one {# instance} other {# instances}}' },
	compatibilitySome: { id: 'app.settings.game-options.compatibility.some', defaultMessage: 'Syncs to {recipients} of {total, plural, one {# instance} other {# instances}}' },
	compatibilitySomeDisabled: { id: 'app.settings.game-options.compatibility.some-disabled', defaultMessage: 'Would sync to {recipients} of {total, plural, one {# instance} other {# instances}}' },
	compatibilityTooltipNoParticipants: { id: 'app.settings.game-options.compatibility.tooltip.no-participants', defaultMessage: 'This setting will begin syncing when an instance participates.' },
	compatibilityTooltipRecipients: { id: 'app.settings.game-options.compatibility.tooltip.recipients', defaultMessage: 'This setting will sync to {recipients} of {total, plural, one {# participating instance} other {# participating instances}}.' },
	bucketLauncherControlled: { id: 'app.settings.game-options.compatibility.reason.launcher-controlled', defaultMessage: '{count, plural, one {# instance has this setting controlled by its launch settings.} other {# instances have this setting controlled by their launch settings.}}' },
	bucketCatalogUncovered: { id: 'app.settings.game-options.compatibility.reason.catalog-uncovered', defaultMessage: '{count, plural, one {# instance uses a Minecraft version this setting catalog does not cover.} other {# instances use Minecraft versions this setting catalog does not cover.}}' },
	bucketInspectionFailed: { id: 'app.settings.game-options.compatibility.reason.inspection-failed', defaultMessage: '{count, plural, one {# instance could not be inspected, so its local value is kept.} other {# instances could not be inspected, so their local values are kept.}}' },
	bucketWaitingForOptionsFile: { id: 'app.settings.game-options.compatibility.reason.waiting-for-options-file', defaultMessage: '{count, plural, one {# instance is waiting for Minecraft to create options.txt.} other {# instances are waiting for Minecraft to create options.txt.}}' },
	bucketUnsupportedValue: { id: 'app.settings.game-options.compatibility.reason.unsupported-value', defaultMessage: '{count, plural, one {# instance cannot represent the selected value.} other {# instances cannot represent the selected value.}}' },
	bucketMigratesOnWrite: { id: 'app.settings.game-options.compatibility.reason.migrates-on-write', defaultMessage: '{count, plural, one {# instance will migrate this setting when options.txt is written.} other {# instances will migrate this setting when their options.txt files are written.}}' },
	bucketWaitingForCompatibleBase: { id: 'app.settings.game-options.compatibility.reason.waiting-for-compatible-base', defaultMessage: '{count, plural, one {# instance is waiting for a compatible modpack options.txt base.} other {# instances are waiting for a compatible modpack options.txt base.}}' },
	bucketMissingSetting: { id: 'app.settings.game-options.compatibility.reason.missing-setting', defaultMessage: '{count, plural, one {# instance does not currently contain this setting.} other {# instances do not currently contain this setting.}}' },
	bucketVersionsAndKeys: { id: 'app.settings.game-options.compatibility.bucket-context.versions-and-keys', defaultMessage: 'Minecraft versions: {versions}. options.txt keys: {keys}.' },
	bucketVersions: { id: 'app.settings.game-options.compatibility.bucket-context.versions', defaultMessage: 'Minecraft versions: {versions}.' },
	bucketKeys: { id: 'app.settings.game-options.compatibility.bucket-context.keys', defaultMessage: 'options.txt keys: {keys}.' },
	sourceInstallingOrUpdating: { id: 'app.settings.game-options.source.disabled.installing-or-updating', defaultMessage: 'Installing or updating' },
	sourceRunning: { id: 'app.settings.game-options.source.disabled.running', defaultMessage: 'Currently running' },
	sourceUnsupportedVersion: { id: 'app.settings.game-options.source.disabled.unsupported-version', defaultMessage: 'Unsupported Minecraft version' },
	sourceMissingOptionsFile: { id: 'app.settings.game-options.source.disabled.missing-options-file', defaultMessage: 'No options.txt file' },
	sourceNoSyncableSettings: { id: 'app.settings.game-options.source.disabled.no-syncable-settings', defaultMessage: 'No syncable settings found' },
	sourceUnreadableOptionsFile: { id: 'app.settings.game-options.source.disabled.unreadable-options-file', defaultMessage: 'options.txt could not be read' },
	sourceSettingsSummary: { id: 'app.settings.game-options.source.settings-summary', defaultMessage: '{recognized, plural, one {# recognized setting} other {# recognized settings}}, {custom, plural, one {# custom setting} other {# custom settings}}' },
})

const knownSettings: Record<string, { label: MessageDescriptor; description: MessageDescriptor }> = {
	fov: { label: settingMessages.fovLabel, description: settingMessages.fovDescription },
	graphics: { label: settingMessages.graphicsLabel, description: settingMessages.graphicsDescription },
	ambient_occlusion: { label: settingMessages.ambientOcclusionLabel, description: settingMessages.ambientOcclusionDescription },
	render_distance: { label: settingMessages.renderDistanceLabel, description: settingMessages.renderDistanceDescription },
	simulation_distance: { label: settingMessages.simulationDistanceLabel, description: settingMessages.simulationDistanceDescription },
	gui_scale: { label: settingMessages.guiScaleLabel, description: settingMessages.guiScaleDescription },
	particles: { label: settingMessages.particlesLabel, description: settingMessages.particlesDescription },
	clouds: { label: settingMessages.cloudsLabel, description: settingMessages.cloudsDescription },
	entity_shadows: { label: settingMessages.entityShadowsLabel, description: settingMessages.entityShadowsDescription },
	view_bobbing: { label: settingMessages.viewBobbingLabel, description: settingMessages.viewBobbingDescription },
	vsync: { label: settingMessages.vsyncLabel, description: settingMessages.vsyncDescription },
	fullscreen: { label: settingMessages.fullscreenLabel, description: settingMessages.fullscreenDescription },
	max_framerate: { label: settingMessages.maxFramerateLabel, description: settingMessages.maxFramerateDescription },
	mipmap_levels: { label: settingMessages.mipmapLevelsLabel, description: settingMessages.mipmapLevelsDescription },
	biome_blend_radius: { label: settingMessages.biomeBlendRadiusLabel, description: settingMessages.biomeBlendRadiusDescription },
	language: { label: settingMessages.languageLabel, description: settingMessages.languageDescription },
	master_volume: { label: settingMessages.masterVolumeLabel, description: settingMessages.masterVolumeDescription },
	music_volume: { label: settingMessages.musicVolumeLabel, description: settingMessages.musicVolumeDescription },
	music_toast: { label: settingMessages.musicToastLabel, description: settingMessages.musicToastDescription },
	blocks_volume: { label: settingMessages.blocksVolumeLabel, description: settingMessages.blocksVolumeDescription },
	hostile_volume: { label: settingMessages.hostileVolumeLabel, description: settingMessages.hostileVolumeDescription },
	players_volume: { label: settingMessages.playersVolumeLabel, description: settingMessages.playersVolumeDescription },
	voice_volume: { label: settingMessages.voiceVolumeLabel, description: settingMessages.voiceVolumeDescription },
	sensitivity: { label: settingMessages.sensitivityLabel, description: settingMessages.sensitivityDescription },
	invert_mouse: { label: settingMessages.invertMouseLabel, description: settingMessages.invertMouseDescription },
	auto_jump: { label: settingMessages.autoJumpLabel, description: settingMessages.autoJumpDescription },
	toggle_crouch: { label: settingMessages.toggleCrouchLabel, description: settingMessages.toggleCrouchDescription },
	toggle_sprint: { label: settingMessages.toggleSprintLabel, description: settingMessages.toggleSprintDescription },
	discrete_mouse_scroll: { label: settingMessages.discreteMouseScrollLabel, description: settingMessages.discreteMouseScrollDescription },
	'key.forward': { label: settingMessages.keyForwardLabel, description: settingMessages.keyForwardDescription },
	'key.left': { label: settingMessages.keyLeftLabel, description: settingMessages.keyLeftDescription },
	'key.back': { label: settingMessages.keyBackLabel, description: settingMessages.keyBackDescription },
	'key.right': { label: settingMessages.keyRightLabel, description: settingMessages.keyRightDescription },
	'key.jump': { label: settingMessages.keyJumpLabel, description: settingMessages.keyJumpDescription },
	'key.sneak': { label: settingMessages.keySneakLabel, description: settingMessages.keySneakDescription },
	'key.sprint': { label: settingMessages.keySprintLabel, description: settingMessages.keySprintDescription },
	'key.inventory': { label: settingMessages.keyInventoryLabel, description: settingMessages.keyInventoryDescription },
	'key.swap_offhand': { label: settingMessages.keySwapOffhandLabel, description: settingMessages.keySwapOffhandDescription },
	'key.drop': { label: settingMessages.keyDropLabel, description: settingMessages.keyDropDescription },
	'key.use': { label: settingMessages.keyUseLabel, description: settingMessages.keyUseDescription },
	'key.attack': { label: settingMessages.keyAttackLabel, description: settingMessages.keyAttackDescription },
	'key.pick_item': { label: settingMessages.keyPickItemLabel, description: settingMessages.keyPickItemDescription },
	'key.chat': { label: settingMessages.keyChatLabel, description: settingMessages.keyChatDescription },
	'key.player_list': { label: settingMessages.keyPlayerListLabel, description: settingMessages.keyPlayerListDescription },
	'key.command': { label: settingMessages.keyCommandLabel, description: settingMessages.keyCommandDescription },
	'key.screenshot': { label: settingMessages.keyScreenshotLabel, description: settingMessages.keyScreenshotDescription },
	'key.perspective': { label: settingMessages.keyPerspectiveLabel, description: settingMessages.keyPerspectiveDescription },
	'key.fullscreen': { label: settingMessages.keyFullscreenLabel, description: settingMessages.keyFullscreenDescription },
	'key.advancements': { label: settingMessages.keyAdvancementsLabel, description: settingMessages.keyAdvancementsDescription },
	chat_visibility: { label: settingMessages.chatVisibilityLabel, description: settingMessages.chatVisibilityDescription },
	chat_colors: { label: settingMessages.chatColorsLabel, description: settingMessages.chatColorsDescription },
	chat_links: { label: settingMessages.chatLinksLabel, description: settingMessages.chatLinksDescription },
	chat_links_prompt: { label: settingMessages.chatLinksPromptLabel, description: settingMessages.chatLinksPromptDescription },
	chat_opacity: { label: settingMessages.chatOpacityLabel, description: settingMessages.chatOpacityDescription },
	chat_scale: { label: settingMessages.chatScaleLabel, description: settingMessages.chatScaleDescription },
	narrator: { label: settingMessages.narratorLabel, description: settingMessages.narratorDescription },
	subtitles: { label: settingMessages.subtitlesLabel, description: settingMessages.subtitlesDescription },
	high_contrast: { label: settingMessages.highContrastLabel, description: settingMessages.highContrastDescription },
	dark_splash: { label: settingMessages.darkSplashLabel, description: settingMessages.darkSplashDescription },
	notification_time: { label: settingMessages.notificationTimeLabel, description: settingMessages.notificationTimeDescription },
	main_hand: { label: settingMessages.mainHandLabel, description: settingMessages.mainHandDescription },
	cape: { label: settingMessages.capeLabel, description: settingMessages.capeDescription },
	hat: { label: settingMessages.hatLabel, description: settingMessages.hatDescription },
	jacket: { label: settingMessages.jacketLabel, description: settingMessages.jacketDescription },
	allow_server_listing: { label: settingMessages.allowServerListingLabel, description: settingMessages.allowServerListingDescription },
	realms_notifications: { label: settingMessages.realmsNotificationsLabel, description: settingMessages.realmsNotificationsDescription },
}

const categories: Record<string, { label: MessageDescriptor; description: MessageDescriptor }> = {
	skin_customization: { label: categoryMessages.skinCustomizationLabel, description: categoryMessages.skinCustomizationDescription },
	video: { label: categoryMessages.videoLabel, description: categoryMessages.videoDescription },
	video_settings: { label: categoryMessages.videoLabel, description: categoryMessages.videoDescription },
	language: { label: categoryMessages.languageLabel, description: categoryMessages.languageDescription },
	music_and_sound: { label: categoryMessages.musicAndSoundLabel, description: categoryMessages.musicAndSoundDescription },
	controls: { label: categoryMessages.controlsLabel, description: categoryMessages.controlsDescription },
	chat: { label: categoryMessages.chatLabel, description: categoryMessages.chatDescription },
	chat_settings: { label: categoryMessages.chatLabel, description: categoryMessages.chatDescription },
	accessibility: { label: categoryMessages.accessibilityLabel, description: categoryMessages.accessibilityDescription },
	online: { label: categoryMessages.onlineLabel, description: categoryMessages.onlineDescription },
	custom: { label: categoryMessages.customLabel, description: categoryMessages.customDescription },
	custom_settings: { label: categoryMessages.customLabel, description: categoryMessages.customDescription },
}

const choices: Record<string, MessageDescriptor> = {
	'graphics:fast': choiceMessages.fast,
	'graphics:fancy': choiceMessages.fancy,
	'graphics:fabulous': choiceMessages.fabulous,
	'main_hand:left': choiceMessages.left,
	'main_hand:right': choiceMessages.right,
	'chat_visibility:0': choiceMessages.shown,
	'chat_visibility:1': choiceMessages.commandsOnly,
	'chat_visibility:2': choiceMessages.hidden,
	'particles:0': choiceMessages.all,
	'particles:1': choiceMessages.decreased,
	'particles:2': choiceMessages.minimal,
	'narrator:0': choiceMessages.off,
	'narrator:1': choiceMessages.all,
	'narrator:2': choiceMessages.chat,
	'narrator:3': choiceMessages.system,
	'clouds:false': choiceMessages.off,
	'clouds:fast': choiceMessages.fast,
	'clouds:true': choiceMessages.fancy,
	'ambient_occlusion:off': choiceMessages.off,
	'ambient_occlusion:on': choiceMessages.on,
	'ambient_occlusion:minimum': choiceMessages.minimum,
	'ambient_occlusion:maximum': choiceMessages.maximum,
	'music_toast:never': choiceMessages.never,
	'music_toast:pause': choiceMessages.pause,
	'music_toast:pause_and_toast': choiceMessages.pauseAndToast,
}

const bucketReasonMessages: Record<
	NonNullable<GameOptionCompatibilityBucket['reason']>,
	MessageDescriptor
> = {
	launcher_controlled: presentationMessages.bucketLauncherControlled,
	catalog_uncovered: presentationMessages.bucketCatalogUncovered,
	inspection_failed: presentationMessages.bucketInspectionFailed,
	waiting_for_options_file: presentationMessages.bucketWaitingForOptionsFile,
	unsupported_value: presentationMessages.bucketUnsupportedValue,
	migrates_on_write: presentationMessages.bucketMigratesOnWrite,
	waiting_for_compatible_base: presentationMessages.bucketWaitingForCompatibleBase,
	missing_setting: presentationMessages.bucketMissingSetting,
}

const validationMessages: Record<GameOptionValidationError, MessageDescriptor> = {
	local_value_needs_saving: presentationMessages.validationLocalValueNeedsSaving,
	missing_value: presentationMessages.validationMissingValue,
	no_compatible_instances: presentationMessages.validationNoCompatibleInstances,
	invalid_value: presentationMessages.validationInvalidValue,
	changed_since_opened: presentationMessages.validationChangedSinceOpened,
}

const sourceDisabledMessages: Record<GameOptionsSourceDisabledReason, MessageDescriptor> = {
	installing_or_updating: presentationMessages.sourceInstallingOrUpdating,
	running: presentationMessages.sourceRunning,
	unsupported_version: presentationMessages.sourceUnsupportedVersion,
	missing_options_file: presentationMessages.sourceMissingOptionsFile,
	no_syncable_settings: presentationMessages.sourceNoSyncableSettings,
	unreadable_options_file: presentationMessages.sourceUnreadableOptionsFile,
}

export function formatGameSettingLabel(formatMessage: FormatMessage, setting: EditableGameSetting): string {
	if (setting.kind === 'external') return setting.raw_key ?? setting.option_id
	const definition = knownSettings[setting.option_id]
	return definition ? formatMessage(definition.label) : setting.option_id
}

export function formatGameSettingDescription(formatMessage: FormatMessage, setting: EditableGameSetting): string {
	if (setting.kind === 'external') return formatMessage(settingMessages.customDescription)
	const definition = knownSettings[setting.option_id]
	return definition ? formatMessage(definition.description) : ''
}

export function formatGameSettingCategory(formatMessage: FormatMessage, category: GameSettingCategory): string {
	const definition = categories[category.id]
	return definition ? formatMessage(definition.label) : category.id
}

export function formatGameSettingChoice(formatMessage: FormatMessage, optionId: string, value: string): string {
	const message = choices[`${optionId}:${value}`]
	return message ? formatMessage(message) : value
}

export function formatGameSettingValidation(formatMessage: FormatMessage, error: GameOptionValidationError | null | undefined): string | null {
	return error ? formatMessage(validationMessages[error]) : null
}

export function formatCompatibilitySubtitle(formatMessage: FormatMessage, setting: EditableGameSetting): string {
	const { total_participating: total, will_receive: recipients } = setting.compatibility
	if (total === 0) return formatMessage(presentationMessages.compatibilityNoParticipants)
	if (recipients === 0) return formatMessage(presentationMessages.compatibilityNone)
	if (recipients === total) {
		return formatMessage(
			setting.sync_enabled
				? presentationMessages.compatibilityAll
				: presentationMessages.compatibilityAllDisabled,
			{ count: total },
		)
	}
	return formatMessage(
		setting.sync_enabled
			? presentationMessages.compatibilitySome
			: presentationMessages.compatibilitySomeDisabled,
		{ recipients, total },
	)
}

export function formatCompatibilityTooltip(formatMessage: FormatMessage, setting: EditableGameSetting): string {
	const summary = setting.compatibility
	const lines = [
		summary.total_participating === 0
			? formatMessage(presentationMessages.compatibilityTooltipNoParticipants)
			: formatMessage(presentationMessages.compatibilityTooltipRecipients, {
					recipients: summary.will_receive,
					total: summary.total_participating,
				}),
	]

	for (const bucket of summary.buckets) {
		if (bucket.reason) {
			lines.push(formatMessage(bucketReasonMessages[bucket.reason], { count: bucket.instance_count }))
		}
		const versions = bucket.game_versions.join(', ')
		const keys = [...bucket.write_keys, ...bucket.eventual_keys].join(', ')
		if (versions && keys) {
			lines.push(formatMessage(presentationMessages.bucketVersionsAndKeys, { versions, keys }))
		} else if (versions) {
			lines.push(formatMessage(presentationMessages.bucketVersions, { versions }))
		} else if (keys) {
			lines.push(formatMessage(presentationMessages.bucketKeys, { keys }))
		}
	}

	return lines.join('\n')
}

export function shouldShowCompatibilityIndicator(setting: EditableGameSetting): boolean {
	return setting.compatibility.buckets.some(
		(bucket) =>
			bucket.mapping !== 'direct' ||
			bucket.status !== 'ready' ||
			bucket.eventual_keys.length > 0,
	)
}

export function formatSourceDisabledReason(formatMessage: FormatMessage, reason: GameOptionsSourceDisabledReason | null | undefined): string | null {
	return reason ? formatMessage(sourceDisabledMessages[reason]) : null
}

export function formatSourceSettingsSummary(
	formatMessage: FormatMessage,
	source: Pick<GameOptionsSourceCandidate, 'recognized_setting_count' | 'custom_setting_count'>,
): string {
	return formatMessage(presentationMessages.sourceSettingsSummary, {
		recognized: source.recognized_setting_count,
		custom: source.custom_setting_count,
	})
}
