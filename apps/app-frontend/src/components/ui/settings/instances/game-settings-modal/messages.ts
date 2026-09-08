import { defineMessages, type MessageDescriptor, type VIntlFormatters } from '@modrinth/ui'

import type {
	EditableGameSetting,
	GameOptionValidationError,
	GameSettingCategory,
} from '@/helpers/game-options'

type FormatMessage = VIntlFormatters['formatMessage']

const settingMessages = defineMessages({
	graphicsDescription: {
		id: 'app.settings.game-options.setting.graphics.description',
		defaultMessage: 'Controls visual quality and performance.',
	},
	simulationDistanceDescription: {
		id: 'app.settings.game-options.setting.simulation-distance.description',
		defaultMessage: 'How far away entities update and blocks and fluids tick.',
	},
	guiScaleDescription: {
		id: 'app.settings.game-options.setting.gui-scale.description',
		defaultMessage: 'The size of the game interface and HUD.',
	},
	viewBobbingDescription: {
		id: 'app.settings.game-options.setting.view-bobbing.description',
		defaultMessage: 'Add a bobbing motion to the camera while walking.',
	},
	vsyncDescription: {
		id: 'app.settings.game-options.setting.vsync.description',
		defaultMessage: 'Limit the frame rate to the display refresh rate to prevent screen tearing.',
	},
	mipmapLevelsDescription: {
		id: 'app.settings.game-options.setting.mipmap-levels.description',
		defaultMessage: 'Texture smoothing at a distance.',
	},
	biomeBlendRadiusDescription: {
		id: 'app.settings.game-options.setting.biome-blend-radius.description',
		defaultMessage: 'The distance over which biome colors transition.',
	},
	musicToastDescription: {
		id: 'app.settings.game-options.setting.music-toast.description',
		defaultMessage: 'Choose whether music titles appear in the pause menu and as toasts.',
	},
	invertMouseDescription: {
		id: 'app.settings.game-options.setting.invert-mouse.description',
		defaultMessage: 'Invert vertical mouse movement.',
	},
	autoJumpDescription: {
		id: 'app.settings.game-options.setting.auto-jump.description',
		defaultMessage: 'Automatically jump up one-block-high obstacles.',
	},
	toggleCrouchDescription: {
		id: 'app.settings.game-options.setting.toggle-crouch.description',
		defaultMessage: 'Press once to remain crouched.',
	},
	toggleSprintDescription: {
		id: 'app.settings.game-options.setting.toggle-sprint.description',
		defaultMessage: 'Press once to remain sprinting.',
	},
	discreteMouseScrollDescription: {
		id: 'app.settings.game-options.setting.discrete-mouse-scroll.description',
		defaultMessage: 'Treat each mouse-wheel input as a single scroll step.',
	},
	chatLinksDescription: {
		id: 'app.settings.game-options.setting.chat-links.description',
		defaultMessage: 'Allow web links in chat to be opened.',
	},
	chatLinksPromptDescription: {
		id: 'app.settings.game-options.setting.chat-links-prompt.description',
		defaultMessage: 'Ask before opening links from chat.',
	},
	chatOpacityDescription: {
		id: 'app.settings.game-options.setting.chat-opacity.description',
		defaultMessage: 'The opacity of chat text.',
	},
	narratorDescription: {
		id: 'app.settings.game-options.setting.narrator.description',
		defaultMessage: 'Choose what the narrator reads.',
	},
	subtitlesDescription: {
		id: 'app.settings.game-options.setting.subtitles.description',
		defaultMessage: 'Show captions for sounds played in the game.',
	},
	highContrastDescription: {
		id: 'app.settings.game-options.setting.high-contrast.description',
		defaultMessage: 'Enhance the contrast of interface elements.',
	},
	darkSplashDescription: {
		id: 'app.settings.game-options.setting.dark-splash.description',
		defaultMessage: 'Change the Mojang Studios loading screen from red to black.',
	},
	notificationTimeDescription: {
		id: 'app.settings.game-options.setting.notification-time.description',
		defaultMessage: 'How long toast notifications remain visible.',
	},
	mainHandDescription: {
		id: 'app.settings.game-options.setting.main-hand.description',
		defaultMessage: 'Choose whether the main hand is left or right.',
	},
	capeDescription: {
		id: 'app.settings.game-options.setting.cape.description',
		defaultMessage: "Show the player's cape, including its elytra texture.",
	},
	hatDescription: {
		id: 'app.settings.game-options.setting.hat.description',
		defaultMessage: 'Show the hat skin layer.',
	},
	jacketDescription: {
		id: 'app.settings.game-options.setting.jacket.description',
		defaultMessage: 'Show the jacket skin layer.',
	},
	allowServerListingDescription: {
		id: 'app.settings.game-options.setting.allow-server-listing.description',
		defaultMessage: "Allow the player's name to appear in server listings.",
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

export const presentationMessages = defineMessages({
	customValuePlaceholder: {
		id: 'app.settings.game-options.custom-value.placeholder',
		defaultMessage: 'Enter a value',
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
	compatibilityNone: {
		id: 'app.settings.game-options.compatibility.none',
		defaultMessage: 'Some of your instances cannot use this setting',
	},
	bucketLauncherControlled: {
		id: 'app.settings.game-options.compatibility.reason.launcher-controlled',
		defaultMessage: 'This setting is managed by Modrinth’s launch settings.',
	},
})

const settingDescriptions: Record<string, MessageDescriptor> = {
	graphics: settingMessages.graphicsDescription,
	simulation_distance: settingMessages.simulationDistanceDescription,
	gui_scale: settingMessages.guiScaleDescription,
	view_bobbing: settingMessages.viewBobbingDescription,
	vsync: settingMessages.vsyncDescription,
	mipmap_levels: settingMessages.mipmapLevelsDescription,
	biome_blend_radius: settingMessages.biomeBlendRadiusDescription,
	music_toast: settingMessages.musicToastDescription,
	invert_mouse: settingMessages.invertMouseDescription,
	auto_jump: settingMessages.autoJumpDescription,
	toggle_crouch: settingMessages.toggleCrouchDescription,
	toggle_sprint: settingMessages.toggleSprintDescription,
	discrete_mouse_scroll: settingMessages.discreteMouseScrollDescription,
	chat_links: settingMessages.chatLinksDescription,
	chat_links_prompt: settingMessages.chatLinksPromptDescription,
	chat_opacity: settingMessages.chatOpacityDescription,
	narrator: settingMessages.narratorDescription,
	subtitles: settingMessages.subtitlesDescription,
	high_contrast: settingMessages.highContrastDescription,
	dark_splash: settingMessages.darkSplashDescription,
	notification_time: settingMessages.notificationTimeDescription,
	main_hand: settingMessages.mainHandDescription,
	cape: settingMessages.capeDescription,
	hat: settingMessages.hatDescription,
	jacket: settingMessages.jacketDescription,
	allow_server_listing: settingMessages.allowServerListingDescription,
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

const validationMessages: Record<GameOptionValidationError, MessageDescriptor> = {
	missing_value: presentationMessages.validationMissingValue,
	no_compatible_instances: presentationMessages.validationNoCompatibleInstances,
	invalid_value: presentationMessages.validationInvalidValue,
	changed_since_opened: presentationMessages.validationChangedSinceOpened,
}

export function formatGameSettingDescription(
	formatMessage: FormatMessage,
	setting: EditableGameSetting,
): string {
	if (setting.kind === 'external') return ''
	const description = settingDescriptions[setting.option_id]
	return description ? formatMessage(description) : ''
}

export function gameSettingCategoryMessage(category: GameSettingCategory): MessageDescriptor {
	return (
		categories[category.id]?.label ?? {
			id: `app.settings.game-options.category.${category.id}.label`,
			defaultMessage: category.id,
		}
	)
}

export function formatGameSettingValidation(
	formatMessage: FormatMessage,
	error: GameOptionValidationError | null | undefined,
): string | null {
	return error ? formatMessage(validationMessages[error]) : null
}
