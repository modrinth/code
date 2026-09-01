import { defineMessages, type MessageDescriptor, type VIntlFormatters } from '@modrinth/ui'

import type {
	EditableGameSetting,
	GameOptionsSourceCandidate,
	GameOptionsSourceDisabledReason,
	GameOptionValidationError,
	GameSettingCategory,
} from '@/helpers/game-options'

type FormatMessage = VIntlFormatters['formatMessage']

const settingMessages = defineMessages({
	fovLabel: { id: 'app.settings.game-options.setting.fov.label', defaultMessage: 'Field of view' },
	graphicsLabel: {
		id: 'app.settings.game-options.setting.graphics.label',
		defaultMessage: 'Graphics',
	},
	graphicsDescription: {
		id: 'app.settings.game-options.setting.graphics.description',
		defaultMessage: 'Controls visual quality and performance.',
	},
	ambientOcclusionLabel: {
		id: 'app.settings.game-options.setting.ambient-occlusion.label',
		defaultMessage: 'Smooth lighting',
	},
	renderDistanceLabel: {
		id: 'app.settings.game-options.setting.render-distance.label',
		defaultMessage: 'Render distance',
	},
	simulationDistanceLabel: {
		id: 'app.settings.game-options.setting.simulation-distance.label',
		defaultMessage: 'Simulation distance',
	},
	simulationDistanceDescription: {
		id: 'app.settings.game-options.setting.simulation-distance.description',
		defaultMessage: 'How far away entities update and blocks and fluids tick.',
	},
	guiScaleLabel: {
		id: 'app.settings.game-options.setting.gui-scale.label',
		defaultMessage: 'GUI scale',
	},
	guiScaleDescription: {
		id: 'app.settings.game-options.setting.gui-scale.description',
		defaultMessage: 'The size of the game interface and HUD.',
	},
	particlesLabel: {
		id: 'app.settings.game-options.setting.particles.label',
		defaultMessage: 'Particles',
	},
	cloudsLabel: { id: 'app.settings.game-options.setting.clouds.label', defaultMessage: 'Clouds' },
	entityShadowsLabel: {
		id: 'app.settings.game-options.setting.entity-shadows.label',
		defaultMessage: 'Entity shadows',
	},
	viewBobbingLabel: {
		id: 'app.settings.game-options.setting.view-bobbing.label',
		defaultMessage: 'View bobbing',
	},
	viewBobbingDescription: {
		id: 'app.settings.game-options.setting.view-bobbing.description',
		defaultMessage: 'Add a bobbing motion to the camera while walking.',
	},
	vsyncLabel: { id: 'app.settings.game-options.setting.vsync.label', defaultMessage: 'VSync' },
	vsyncDescription: {
		id: 'app.settings.game-options.setting.vsync.description',
		defaultMessage: 'Limit the frame rate to the display refresh rate to prevent screen tearing.',
	},
	fullscreenLabel: {
		id: 'app.settings.game-options.setting.fullscreen.label',
		defaultMessage: 'Fullscreen',
	},
	maxFramerateLabel: {
		id: 'app.settings.game-options.setting.max-framerate.label',
		defaultMessage: 'Maximum framerate',
	},
	mipmapLevelsLabel: {
		id: 'app.settings.game-options.setting.mipmap-levels.label',
		defaultMessage: 'Mipmap levels',
	},
	mipmapLevelsDescription: {
		id: 'app.settings.game-options.setting.mipmap-levels.description',
		defaultMessage: 'Texture smoothing at a distance.',
	},
	biomeBlendRadiusLabel: {
		id: 'app.settings.game-options.setting.biome-blend-radius.label',
		defaultMessage: 'Biome blend',
	},
	biomeBlendRadiusDescription: {
		id: 'app.settings.game-options.setting.biome-blend-radius.description',
		defaultMessage: 'The distance over which biome colors transition.',
	},
	languageLabel: {
		id: 'app.settings.game-options.setting.language.label',
		defaultMessage: 'Language',
	},
	masterVolumeLabel: {
		id: 'app.settings.game-options.setting.master-volume.label',
		defaultMessage: 'Master volume',
	},
	musicVolumeLabel: {
		id: 'app.settings.game-options.setting.music-volume.label',
		defaultMessage: 'Music',
	},
	musicToastLabel: {
		id: 'app.settings.game-options.setting.music-toast.label',
		defaultMessage: 'Music notification',
	},
	musicToastDescription: {
		id: 'app.settings.game-options.setting.music-toast.description',
		defaultMessage: 'Choose whether music titles appear in the pause menu and as toasts.',
	},
	recordVolumeLabel: {
		id: 'app.settings.game-options.setting.record-volume.label',
		defaultMessage: 'Jukebox/Note Blocks',
	},
	weatherVolumeLabel: {
		id: 'app.settings.game-options.setting.weather-volume.label',
		defaultMessage: 'Weather',
	},
	blocksVolumeLabel: {
		id: 'app.settings.game-options.setting.blocks-volume.label',
		defaultMessage: 'Blocks',
	},
	hostileVolumeLabel: {
		id: 'app.settings.game-options.setting.hostile-volume.label',
		defaultMessage: 'Hostile creatures',
	},
	neutralVolumeLabel: {
		id: 'app.settings.game-options.setting.neutral-volume.label',
		defaultMessage: 'Friendly creatures',
	},
	playersVolumeLabel: {
		id: 'app.settings.game-options.setting.players-volume.label',
		defaultMessage: 'Players',
	},
	ambientVolumeLabel: {
		id: 'app.settings.game-options.setting.ambient-volume.label',
		defaultMessage: 'Ambient/Environment',
	},
	voiceVolumeLabel: {
		id: 'app.settings.game-options.setting.voice-volume.label',
		defaultMessage: 'Voice and speech',
	},
	uiVolumeLabel: {
		id: 'app.settings.game-options.setting.ui-volume.label',
		defaultMessage: 'UI',
	},
	sensitivityLabel: {
		id: 'app.settings.game-options.setting.sensitivity.label',
		defaultMessage: 'Mouse sensitivity',
	},
	invertMouseLabel: {
		id: 'app.settings.game-options.setting.invert-mouse.label',
		defaultMessage: 'Invert mouse',
	},
	invertMouseDescription: {
		id: 'app.settings.game-options.setting.invert-mouse.description',
		defaultMessage: 'Invert vertical mouse movement.',
	},
	autoJumpLabel: {
		id: 'app.settings.game-options.setting.auto-jump.label',
		defaultMessage: 'Auto-jump',
	},
	autoJumpDescription: {
		id: 'app.settings.game-options.setting.auto-jump.description',
		defaultMessage: 'Automatically jump up one-block-high obstacles.',
	},
	toggleCrouchLabel: {
		id: 'app.settings.game-options.setting.toggle-crouch.label',
		defaultMessage: 'Toggle crouch',
	},
	toggleCrouchDescription: {
		id: 'app.settings.game-options.setting.toggle-crouch.description',
		defaultMessage: 'Press once to remain crouched.',
	},
	toggleSprintLabel: {
		id: 'app.settings.game-options.setting.toggle-sprint.label',
		defaultMessage: 'Toggle sprint',
	},
	toggleSprintDescription: {
		id: 'app.settings.game-options.setting.toggle-sprint.description',
		defaultMessage: 'Press once to remain sprinting.',
	},
	discreteMouseScrollLabel: {
		id: 'app.settings.game-options.setting.discrete-mouse-scroll.label',
		defaultMessage: 'Discrete scrolling',
	},
	discreteMouseScrollDescription: {
		id: 'app.settings.game-options.setting.discrete-mouse-scroll.description',
		defaultMessage: 'Treat each mouse-wheel input as a single scroll step.',
	},
	keyForwardLabel: {
		id: 'app.settings.game-options.setting.key-forward.label',
		defaultMessage: 'Move forward',
	},
	keyLeftLabel: {
		id: 'app.settings.game-options.setting.key-left.label',
		defaultMessage: 'Strafe left',
	},
	keyBackLabel: {
		id: 'app.settings.game-options.setting.key-back.label',
		defaultMessage: 'Move backward',
	},
	keyRightLabel: {
		id: 'app.settings.game-options.setting.key-right.label',
		defaultMessage: 'Strafe right',
	},
	keyJumpLabel: { id: 'app.settings.game-options.setting.key-jump.label', defaultMessage: 'Jump' },
	keySneakLabel: {
		id: 'app.settings.game-options.setting.key-sneak.label',
		defaultMessage: 'Sneak',
	},
	keySprintLabel: {
		id: 'app.settings.game-options.setting.key-sprint.label',
		defaultMessage: 'Sprint',
	},
	keyInventoryLabel: {
		id: 'app.settings.game-options.setting.key-inventory.label',
		defaultMessage: 'Inventory',
	},
	keySwapOffhandLabel: {
		id: 'app.settings.game-options.setting.key-swap-offhand.label',
		defaultMessage: 'Swap offhand',
	},
	keyDropLabel: {
		id: 'app.settings.game-options.setting.key-drop.label',
		defaultMessage: 'Drop item',
	},
	keyUseLabel: {
		id: 'app.settings.game-options.setting.key-use.label',
		defaultMessage: 'Use item',
	},
	keyAttackLabel: {
		id: 'app.settings.game-options.setting.key-attack.label',
		defaultMessage: 'Attack',
	},
	keyPickItemLabel: {
		id: 'app.settings.game-options.setting.key-pick-item.label',
		defaultMessage: 'Pick block',
	},
	keyChatLabel: {
		id: 'app.settings.game-options.setting.key-chat.label',
		defaultMessage: 'Open chat',
	},
	keyPlayerListLabel: {
		id: 'app.settings.game-options.setting.key-player-list.label',
		defaultMessage: 'Player list',
	},
	keyCommandLabel: {
		id: 'app.settings.game-options.setting.key-command.label',
		defaultMessage: 'Command',
	},
	keyScreenshotLabel: {
		id: 'app.settings.game-options.setting.key-screenshot.label',
		defaultMessage: 'Screenshot',
	},
	keyPerspectiveLabel: {
		id: 'app.settings.game-options.setting.key-perspective.label',
		defaultMessage: 'Change perspective',
	},
	keyFullscreenLabel: {
		id: 'app.settings.game-options.setting.key-fullscreen.label',
		defaultMessage: 'Toggle fullscreen',
	},
	keyAdvancementsLabel: {
		id: 'app.settings.game-options.setting.key-advancements.label',
		defaultMessage: 'Advancements',
	},
	chatVisibilityLabel: {
		id: 'app.settings.game-options.setting.chat-visibility.label',
		defaultMessage: 'Chat visibility',
	},
	chatColorsLabel: {
		id: 'app.settings.game-options.setting.chat-colors.label',
		defaultMessage: 'Chat colors',
	},
	chatLinksLabel: {
		id: 'app.settings.game-options.setting.chat-links.label',
		defaultMessage: 'Web links',
	},
	chatLinksDescription: {
		id: 'app.settings.game-options.setting.chat-links.description',
		defaultMessage: 'Allow web links in chat to be opened.',
	},
	chatLinksPromptLabel: {
		id: 'app.settings.game-options.setting.chat-links-prompt.label',
		defaultMessage: 'Prompt on links',
	},
	chatLinksPromptDescription: {
		id: 'app.settings.game-options.setting.chat-links-prompt.description',
		defaultMessage: 'Ask before opening links from chat.',
	},
	chatOpacityLabel: {
		id: 'app.settings.game-options.setting.chat-opacity.label',
		defaultMessage: 'Chat opacity',
	},
	chatOpacityDescription: {
		id: 'app.settings.game-options.setting.chat-opacity.description',
		defaultMessage: 'The opacity of chat text.',
	},
	chatScaleLabel: {
		id: 'app.settings.game-options.setting.chat-scale.label',
		defaultMessage: 'Chat scale',
	},
	narratorLabel: {
		id: 'app.settings.game-options.setting.narrator.label',
		defaultMessage: 'Narrator',
	},
	narratorDescription: {
		id: 'app.settings.game-options.setting.narrator.description',
		defaultMessage: 'Choose what the narrator reads.',
	},
	subtitlesLabel: {
		id: 'app.settings.game-options.setting.subtitles.label',
		defaultMessage: 'Subtitles',
	},
	subtitlesDescription: {
		id: 'app.settings.game-options.setting.subtitles.description',
		defaultMessage: 'Show captions for sounds played in the game.',
	},
	highContrastLabel: {
		id: 'app.settings.game-options.setting.high-contrast.label',
		defaultMessage: 'High contrast',
	},
	highContrastDescription: {
		id: 'app.settings.game-options.setting.high-contrast.description',
		defaultMessage: 'Enhance the contrast of interface elements.',
	},
	darkSplashLabel: {
		id: 'app.settings.game-options.setting.dark-splash.label',
		defaultMessage: 'Monochrome logo',
	},
	darkSplashDescription: {
		id: 'app.settings.game-options.setting.dark-splash.description',
		defaultMessage: 'Change the Mojang Studios loading screen from red to black.',
	},
	notificationTimeLabel: {
		id: 'app.settings.game-options.setting.notification-time.label',
		defaultMessage: 'Notification time',
	},
	notificationTimeDescription: {
		id: 'app.settings.game-options.setting.notification-time.description',
		defaultMessage: 'How long toast notifications remain visible.',
	},
	mainHandLabel: {
		id: 'app.settings.game-options.setting.main-hand.label',
		defaultMessage: 'Main hand',
	},
	mainHandDescription: {
		id: 'app.settings.game-options.setting.main-hand.description',
		defaultMessage: 'Choose whether the main hand is left or right.',
	},
	capeLabel: { id: 'app.settings.game-options.setting.cape.label', defaultMessage: 'Cape' },
	capeDescription: {
		id: 'app.settings.game-options.setting.cape.description',
		defaultMessage: "Show the player's cape, including its elytra texture.",
	},
	hatLabel: { id: 'app.settings.game-options.setting.hat.label', defaultMessage: 'Hat' },
	hatDescription: {
		id: 'app.settings.game-options.setting.hat.description',
		defaultMessage: 'Show the hat skin layer.',
	},
	jacketLabel: { id: 'app.settings.game-options.setting.jacket.label', defaultMessage: 'Jacket' },
	jacketDescription: {
		id: 'app.settings.game-options.setting.jacket.description',
		defaultMessage: 'Show the jacket skin layer.',
	},
	allowServerListingLabel: {
		id: 'app.settings.game-options.setting.allow-server-listing.label',
		defaultMessage: 'Server listings',
	},
	allowServerListingDescription: {
		id: 'app.settings.game-options.setting.allow-server-listing.description',
		defaultMessage: "Allow the player's name to appear in server listings.",
	},
	realmsNotificationsLabel: {
		id: 'app.settings.game-options.setting.realms-notifications.label',
		defaultMessage: 'Realms notifications',
	},
})

const categoryMessages = defineMessages({
	skinCustomizationLabel: {
		id: 'app.settings.game-options.category.skin-customization.label',
		defaultMessage: 'Skin customization',
	},
	skinCustomizationDescription: {
		id: 'app.settings.game-options.category.skin-customization.description',
		defaultMessage: 'Skin layers and main hand',
	},
	videoLabel: { id: 'app.settings.game-options.category.video.label', defaultMessage: 'Video' },
	videoDescription: {
		id: 'app.settings.game-options.category.video.description',
		defaultMessage: 'Camera and display settings',
	},
	languageLabel: {
		id: 'app.settings.game-options.category.language.label',
		defaultMessage: 'Language',
	},
	languageDescription: {
		id: 'app.settings.game-options.category.language.description',
		defaultMessage: 'Game language',
	},
	musicAndSoundLabel: {
		id: 'app.settings.game-options.category.music-and-sound.label',
		defaultMessage: 'Music and sound',
	},
	musicAndSoundDescription: {
		id: 'app.settings.game-options.category.music-and-sound.description',
		defaultMessage: 'Volume and audio preferences',
	},
	controlsLabel: {
		id: 'app.settings.game-options.category.controls.label',
		defaultMessage: 'Controls',
	},
	controlsDescription: {
		id: 'app.settings.game-options.category.controls.description',
		defaultMessage: 'Mouse, movement, and key bindings',
	},
	chatLabel: { id: 'app.settings.game-options.category.chat.label', defaultMessage: 'Chat' },
	chatDescription: {
		id: 'app.settings.game-options.category.chat.description',
		defaultMessage: 'Chat visibility and appearance',
	},
	accessibilityLabel: {
		id: 'app.settings.game-options.category.accessibility.label',
		defaultMessage: 'Accessibility',
	},
	accessibilityDescription: {
		id: 'app.settings.game-options.category.accessibility.description',
		defaultMessage: 'Accessibility preferences',
	},
	onlineLabel: { id: 'app.settings.game-options.category.online.label', defaultMessage: 'Online' },
	onlineDescription: {
		id: 'app.settings.game-options.category.online.description',
		defaultMessage: 'Online and Realms preferences',
	},
	customLabel: {
		id: 'app.settings.game-options.category.custom.label',
		defaultMessage: 'Custom settings',
	},
	customDescription: {
		id: 'app.settings.game-options.category.custom.description',
		defaultMessage: 'Settings added by mods',
	},
})

const choiceMessages = defineMessages({
	fast: { id: 'app.settings.game-options.choice.fast', defaultMessage: 'Fast' },
	fancy: { id: 'app.settings.game-options.choice.fancy', defaultMessage: 'Fancy' },
	fabulous: { id: 'app.settings.game-options.choice.fabulous', defaultMessage: 'Fabulous' },
	custom: { id: 'app.settings.game-options.choice.custom', defaultMessage: 'Custom' },
	left: { id: 'app.settings.game-options.choice.left', defaultMessage: 'Left' },
	right: { id: 'app.settings.game-options.choice.right', defaultMessage: 'Right' },
	shown: { id: 'app.settings.game-options.choice.shown', defaultMessage: 'Shown' },
	commandsOnly: {
		id: 'app.settings.game-options.choice.commands-only',
		defaultMessage: 'Commands only',
	},
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
	pause: { id: 'app.settings.game-options.choice.pause', defaultMessage: 'Pause menu' },
	pauseAndToast: {
		id: 'app.settings.game-options.choice.pause-and-toast',
		defaultMessage: 'Pause menu and toast',
	},
})

export const presentationMessages = defineMessages({
	customValuePlaceholder: {
		id: 'app.settings.game-options.custom-value.placeholder',
		defaultMessage: 'Enter a value',
	},
	validationLocalValueNeedsSaving: {
		id: 'app.settings.game-options.validation.local-value-needs-saving',
		defaultMessage: 'Save this value before turning sync back on.',
	},
	validationMissingValue: {
		id: 'app.settings.game-options.validation.missing-value',
		defaultMessage: 'Choose a value first.',
	},
	validationNoCompatibleInstances: {
		id: 'app.settings.game-options.validation.no-compatible-instances',
		defaultMessage: 'Some of your instances cannot use this setting',
	},
	validationInvalidValue: {
		id: 'app.settings.game-options.validation.invalid-value',
		defaultMessage: 'Choose a valid value.',
	},
	validationChangedSinceOpened: {
		id: 'app.settings.game-options.validation.changed-since-opened',
		defaultMessage: 'This setting changed elsewhere. Check it and try again.',
	},
	compatibilityNoParticipants: {
		id: 'app.settings.game-options.compatibility.no-participants',
		defaultMessage: 'Add an instance to start syncing',
	},
	compatibilityNone: {
		id: 'app.settings.game-options.compatibility.none',
		defaultMessage: 'Some of your instances cannot use this setting',
	},
	compatibilityAll: {
		id: 'app.settings.game-options.compatibility.all',
		defaultMessage: 'Syncs to {count, plural, one {# instance} other {# instances}}',
	},
	compatibilityAllDisabled: {
		id: 'app.settings.game-options.compatibility.all-disabled',
		defaultMessage: 'Sync is off',
	},
	compatibilitySome: {
		id: 'app.settings.game-options.compatibility.some',
		defaultMessage: 'Some of your instances cannot use this setting',
	},
	compatibilitySomeDisabled: {
		id: 'app.settings.game-options.compatibility.some-disabled',
		defaultMessage: 'Some of your instances cannot use this setting',
	},
	compatibilityTooltipNoParticipants: {
		id: 'app.settings.game-options.compatibility.tooltip.no-participants',
		defaultMessage: 'Add an instance to start syncing this setting.',
	},
	compatibilityTooltipAll: {
		id: 'app.settings.game-options.compatibility.tooltip.all',
		defaultMessage: 'All of your instances can use this setting.',
	},
	compatibilityTooltipLimited: {
		id: 'app.settings.game-options.compatibility.tooltip.recipients',
		defaultMessage: 'Some of your instances cannot use this setting',
	},
	compatibilityTooltipWaiting: {
		id: 'app.settings.game-options.compatibility.tooltip.waiting',
		defaultMessage:
			'{count, plural, one {Launch this instance once before syncing this setting.} other {Launch these instances once before syncing this setting.}}',
	},
	bucketLauncherControlled: {
		id: 'app.settings.game-options.compatibility.reason.launcher-controlled',
		defaultMessage: 'This setting is managed by Modrinth’s launch settings.',
	},
	sourceInstallingOrUpdating: {
		id: 'app.settings.game-options.source.disabled.installing-or-updating',
		defaultMessage: 'Installing or updating',
	},
	sourceUnsupportedVersion: {
		id: 'app.settings.game-options.source.disabled.unsupported-version',
		defaultMessage: 'Unsupported Minecraft version',
	},
	sourceMissingOptionsFile: {
		id: 'app.settings.game-options.source.disabled.missing-options-file',
		defaultMessage: 'No options.txt file',
	},
	sourceNoSyncableSettings: {
		id: 'app.settings.game-options.source.disabled.no-syncable-settings',
		defaultMessage: 'No syncable settings found',
	},
	sourceUnreadableOptionsFile: {
		id: 'app.settings.game-options.source.disabled.unreadable-options-file',
		defaultMessage: 'options.txt could not be read',
	},
	sourceSettingsSummary: {
		id: 'app.settings.game-options.source.settings-summary',
		defaultMessage:
			'{recognized, plural, one {# recognized setting} other {# recognized settings}}, {custom, plural, one {# custom setting} other {# custom settings}}',
	},
})

const knownSettings: Record<string, { label: MessageDescriptor; description?: MessageDescriptor }> =
	{
		fov: { label: settingMessages.fovLabel },
		graphics: {
			label: settingMessages.graphicsLabel,
			description: settingMessages.graphicsDescription,
		},
		ambient_occlusion: { label: settingMessages.ambientOcclusionLabel },
		render_distance: { label: settingMessages.renderDistanceLabel },
		simulation_distance: {
			label: settingMessages.simulationDistanceLabel,
			description: settingMessages.simulationDistanceDescription,
		},
		gui_scale: {
			label: settingMessages.guiScaleLabel,
			description: settingMessages.guiScaleDescription,
		},
		particles: { label: settingMessages.particlesLabel },
		clouds: { label: settingMessages.cloudsLabel },
		entity_shadows: { label: settingMessages.entityShadowsLabel },
		view_bobbing: {
			label: settingMessages.viewBobbingLabel,
			description: settingMessages.viewBobbingDescription,
		},
		vsync: { label: settingMessages.vsyncLabel, description: settingMessages.vsyncDescription },
		fullscreen: { label: settingMessages.fullscreenLabel },
		max_framerate: { label: settingMessages.maxFramerateLabel },
		mipmap_levels: {
			label: settingMessages.mipmapLevelsLabel,
			description: settingMessages.mipmapLevelsDescription,
		},
		biome_blend_radius: {
			label: settingMessages.biomeBlendRadiusLabel,
			description: settingMessages.biomeBlendRadiusDescription,
		},
		language: { label: settingMessages.languageLabel },
		master_volume: { label: settingMessages.masterVolumeLabel },
		music_volume: { label: settingMessages.musicVolumeLabel },
		music_toast: {
			label: settingMessages.musicToastLabel,
			description: settingMessages.musicToastDescription,
		},
		record_volume: { label: settingMessages.recordVolumeLabel },
		weather_volume: { label: settingMessages.weatherVolumeLabel },
		blocks_volume: { label: settingMessages.blocksVolumeLabel },
		hostile_volume: { label: settingMessages.hostileVolumeLabel },
		neutral_volume: { label: settingMessages.neutralVolumeLabel },
		players_volume: { label: settingMessages.playersVolumeLabel },
		ambient_volume: { label: settingMessages.ambientVolumeLabel },
		voice_volume: { label: settingMessages.voiceVolumeLabel },
		ui_volume: { label: settingMessages.uiVolumeLabel },
		sensitivity: { label: settingMessages.sensitivityLabel },
		invert_mouse: {
			label: settingMessages.invertMouseLabel,
			description: settingMessages.invertMouseDescription,
		},
		auto_jump: {
			label: settingMessages.autoJumpLabel,
			description: settingMessages.autoJumpDescription,
		},
		toggle_crouch: {
			label: settingMessages.toggleCrouchLabel,
			description: settingMessages.toggleCrouchDescription,
		},
		toggle_sprint: {
			label: settingMessages.toggleSprintLabel,
			description: settingMessages.toggleSprintDescription,
		},
		discrete_mouse_scroll: {
			label: settingMessages.discreteMouseScrollLabel,
			description: settingMessages.discreteMouseScrollDescription,
		},
		'key.forward': { label: settingMessages.keyForwardLabel },
		'key.left': { label: settingMessages.keyLeftLabel },
		'key.back': { label: settingMessages.keyBackLabel },
		'key.right': { label: settingMessages.keyRightLabel },
		'key.jump': { label: settingMessages.keyJumpLabel },
		'key.sneak': { label: settingMessages.keySneakLabel },
		'key.sprint': { label: settingMessages.keySprintLabel },
		'key.inventory': { label: settingMessages.keyInventoryLabel },
		'key.swap_offhand': { label: settingMessages.keySwapOffhandLabel },
		'key.drop': { label: settingMessages.keyDropLabel },
		'key.use': { label: settingMessages.keyUseLabel },
		'key.attack': { label: settingMessages.keyAttackLabel },
		'key.pick_item': { label: settingMessages.keyPickItemLabel },
		'key.chat': { label: settingMessages.keyChatLabel },
		'key.player_list': { label: settingMessages.keyPlayerListLabel },
		'key.command': { label: settingMessages.keyCommandLabel },
		'key.screenshot': { label: settingMessages.keyScreenshotLabel },
		'key.perspective': { label: settingMessages.keyPerspectiveLabel },
		'key.fullscreen': { label: settingMessages.keyFullscreenLabel },
		'key.advancements': { label: settingMessages.keyAdvancementsLabel },
		chat_visibility: { label: settingMessages.chatVisibilityLabel },
		chat_colors: { label: settingMessages.chatColorsLabel },
		chat_links: {
			label: settingMessages.chatLinksLabel,
			description: settingMessages.chatLinksDescription,
		},
		chat_links_prompt: {
			label: settingMessages.chatLinksPromptLabel,
			description: settingMessages.chatLinksPromptDescription,
		},
		chat_opacity: {
			label: settingMessages.chatOpacityLabel,
			description: settingMessages.chatOpacityDescription,
		},
		chat_scale: { label: settingMessages.chatScaleLabel },
		narrator: {
			label: settingMessages.narratorLabel,
			description: settingMessages.narratorDescription,
		},
		subtitles: {
			label: settingMessages.subtitlesLabel,
			description: settingMessages.subtitlesDescription,
		},
		high_contrast: {
			label: settingMessages.highContrastLabel,
			description: settingMessages.highContrastDescription,
		},
		dark_splash: {
			label: settingMessages.darkSplashLabel,
			description: settingMessages.darkSplashDescription,
		},
		notification_time: {
			label: settingMessages.notificationTimeLabel,
			description: settingMessages.notificationTimeDescription,
		},
		main_hand: {
			label: settingMessages.mainHandLabel,
			description: settingMessages.mainHandDescription,
		},
		cape: { label: settingMessages.capeLabel, description: settingMessages.capeDescription },
		hat: { label: settingMessages.hatLabel, description: settingMessages.hatDescription },
		jacket: { label: settingMessages.jacketLabel, description: settingMessages.jacketDescription },
		allow_server_listing: {
			label: settingMessages.allowServerListingLabel,
			description: settingMessages.allowServerListingDescription,
		},
		realms_notifications: { label: settingMessages.realmsNotificationsLabel },
	}

const categories: Record<string, { label: MessageDescriptor; description: MessageDescriptor }> = {
	skin_customization: {
		label: categoryMessages.skinCustomizationLabel,
		description: categoryMessages.skinCustomizationDescription,
	},
	video: { label: categoryMessages.videoLabel, description: categoryMessages.videoDescription },
	video_settings: {
		label: categoryMessages.videoLabel,
		description: categoryMessages.videoDescription,
	},
	language: {
		label: categoryMessages.languageLabel,
		description: categoryMessages.languageDescription,
	},
	music_and_sound: {
		label: categoryMessages.musicAndSoundLabel,
		description: categoryMessages.musicAndSoundDescription,
	},
	controls: {
		label: categoryMessages.controlsLabel,
		description: categoryMessages.controlsDescription,
	},
	chat: { label: categoryMessages.chatLabel, description: categoryMessages.chatDescription },
	chat_settings: {
		label: categoryMessages.chatLabel,
		description: categoryMessages.chatDescription,
	},
	accessibility: {
		label: categoryMessages.accessibilityLabel,
		description: categoryMessages.accessibilityDescription,
	},
	online: { label: categoryMessages.onlineLabel, description: categoryMessages.onlineDescription },
	custom: { label: categoryMessages.customLabel, description: categoryMessages.customDescription },
	custom_settings: {
		label: categoryMessages.customLabel,
		description: categoryMessages.customDescription,
	},
}

const choices: Record<string, MessageDescriptor> = {
	'graphics:fast': choiceMessages.fast,
	'graphics:fancy': choiceMessages.fancy,
	'graphics:fabulous': choiceMessages.fabulous,
	'graphics:custom': choiceMessages.custom,
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

const validationMessages: Record<GameOptionValidationError, MessageDescriptor> = {
	local_value_needs_saving: presentationMessages.validationLocalValueNeedsSaving,
	missing_value: presentationMessages.validationMissingValue,
	no_compatible_instances: presentationMessages.validationNoCompatibleInstances,
	invalid_value: presentationMessages.validationInvalidValue,
	changed_since_opened: presentationMessages.validationChangedSinceOpened,
}

const sourceDisabledMessages: Record<GameOptionsSourceDisabledReason, MessageDescriptor> = {
	installing_or_updating: presentationMessages.sourceInstallingOrUpdating,
	unsupported_version: presentationMessages.sourceUnsupportedVersion,
	missing_options_file: presentationMessages.sourceMissingOptionsFile,
	no_syncable_settings: presentationMessages.sourceNoSyncableSettings,
	unreadable_options_file: presentationMessages.sourceUnreadableOptionsFile,
}

export function formatGameSettingLabel(
	formatMessage: FormatMessage,
	setting: EditableGameSetting,
): string {
	if (setting.kind === 'external') return setting.raw_key ?? setting.option_id
	const definition = knownSettings[setting.option_id]
	return definition ? formatMessage(definition.label) : setting.option_id
}

export function formatGameSettingDescription(
	formatMessage: FormatMessage,
	setting: EditableGameSetting,
): string {
	if (setting.kind === 'external') return ''
	const definition = knownSettings[setting.option_id]
	return definition?.description ? formatMessage(definition.description) : ''
}

export function gameSettingCategoryMessage(category: GameSettingCategory): MessageDescriptor {
	return (
		categories[category.id]?.label ?? {
			id: `app.settings.game-options.category.${category.id}.label`,
			defaultMessage: category.id,
		}
	)
}

export function formatGameSettingCategory(
	formatMessage: FormatMessage,
	category: GameSettingCategory,
): string {
	return formatMessage(gameSettingCategoryMessage(category))
}

export function formatGameSettingChoice(
	formatMessage: FormatMessage,
	optionId: string,
	value: string,
): string {
	const message = choices[`${optionId}:${value}`]
	return message ? formatMessage(message) : value
}

export function formatGameSettingValidation(
	formatMessage: FormatMessage,
	error: GameOptionValidationError | null | undefined,
): string | null {
	return error ? formatMessage(validationMessages[error]) : null
}

export function formatCompatibilitySubtitle(
	formatMessage: FormatMessage,
	setting: EditableGameSetting,
): string {
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

export function formatCompatibilityTooltip(
	formatMessage: FormatMessage,
	setting: EditableGameSetting,
): string {
	const summary = setting.compatibility
	if (summary.total_participating === 0) {
		return formatMessage(presentationMessages.compatibilityTooltipNoParticipants)
	}
	if (summary.left_local > 0) {
		return formatMessage(presentationMessages.compatibilityTooltipLimited)
	}
	const waiting = summary.buckets
		.filter(
			(bucket) => bucket.status === 'waiting_for_file' || bucket.status === 'waiting_for_base',
		)
		.reduce((count, bucket) => count + bucket.instance_count, 0)
	if (waiting > 0) {
		return formatMessage(presentationMessages.compatibilityTooltipWaiting, { count: waiting })
	}
	return formatMessage(presentationMessages.compatibilityTooltipAll)
}

export function shouldShowCompatibilityIndicator(setting: EditableGameSetting): boolean {
	return (
		setting.compatibility.left_local > 0 ||
		setting.compatibility.buckets.some(
			(bucket) => bucket.status === 'waiting_for_file' || bucket.status === 'waiting_for_base',
		)
	)
}

export function formatSourceDisabledReason(
	formatMessage: FormatMessage,
	reason: GameOptionsSourceDisabledReason | null | undefined,
): string | null {
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
