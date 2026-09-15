import { defineMessages, type MessageDescriptor, type VIntlFormatters } from '@modrinth/ui'

import type {
	EditableGameSetting,
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

const catalogSettingMessages = defineMessages({
	brightnessLabel: {
		id: 'app.settings.game-options.setting.brightness.label',
		defaultMessage: 'Brightness',
	},
	legacyViewDistanceLabel: {
		id: 'app.settings.game-options.setting.legacy-view-distance.label',
		defaultMessage: 'View distance',
	},
	entityDistanceLabel: {
		id: 'app.settings.game-options.setting.entity-distance.label',
		defaultMessage: 'Entity distance',
	},
	debugGuiScaleLabel: {
		id: 'app.settings.game-options.setting.debug-gui-scale.label',
		defaultMessage: 'Debug GUI scale',
	},
	graphicsBackendLabel: {
		id: 'app.settings.game-options.setting.graphics-backend.label',
		defaultMessage: 'Graphics backend',
	},
	cloudRangeLabel: {
		id: 'app.settings.game-options.setting.cloud-range.label',
		defaultMessage: 'Cloud distance',
	},
	exclusiveFullscreenLabel: {
		id: 'app.settings.game-options.setting.exclusive-fullscreen.label',
		defaultMessage: 'Exclusive fullscreen',
	},
	macFullscreenMenuLabel: {
		id: 'app.settings.game-options.setting.mac-fullscreen-menu.label',
		defaultMessage: 'Show macOS menu in fullscreen',
	},
	legacyFramerateLimitLabel: {
		id: 'app.settings.game-options.setting.legacy-framerate-limit.label',
		defaultMessage: 'Framerate limit',
	},
	inactivityFramerateLimitLabel: {
		id: 'app.settings.game-options.setting.inactivity-framerate-limit.label',
		defaultMessage: 'Reduced framerate',
	},
	prioritizeChunkUpdatesLabel: {
		id: 'app.settings.game-options.setting.prioritize-chunk-updates.label',
		defaultMessage: 'Prioritize chunk updates',
	},
	attackIndicatorLabel: {
		id: 'app.settings.game-options.setting.attack-indicator.label',
		defaultMessage: 'Attack indicator',
	},
	reducedDebugInfoLabel: {
		id: 'app.settings.game-options.setting.reduced-debug-info.label',
		defaultMessage: 'Reduced debug information',
	},
	chunkFadeTimeLabel: {
		id: 'app.settings.game-options.setting.chunk-fade-time.label',
		defaultMessage: 'Chunk fade time',
	},
	cutoutLeavesLabel: {
		id: 'app.settings.game-options.setting.cutout-leaves.label',
		defaultMessage: 'Cutout leaves',
	},
	improvedTransparencyLabel: {
		id: 'app.settings.game-options.setting.improved-transparency.label',
		defaultMessage: 'Improved transparency',
	},
	textureFilteringLabel: {
		id: 'app.settings.game-options.setting.texture-filtering.label',
		defaultMessage: 'Texture filtering',
	},
	anisotropyLabel: {
		id: 'app.settings.game-options.setting.anisotropy.label',
		defaultMessage: 'Anisotropy',
	},
	vignetteLabel: {
		id: 'app.settings.game-options.setting.vignette.label',
		defaultMessage: 'Vignette',
	},
	weatherRadiusLabel: {
		id: 'app.settings.game-options.setting.weather-radius.label',
		defaultMessage: 'Weather radius',
	},
	advancedOpenGlLabel: {
		id: 'app.settings.game-options.setting.advanced-opengl.label',
		defaultMessage: 'Advanced OpenGL',
	},
	anaglyph3dLabel: {
		id: 'app.settings.game-options.setting.anaglyph-3d.label',
		defaultMessage: '3D anaglyph',
	},
	anisotropicFilteringLabel: {
		id: 'app.settings.game-options.setting.anisotropic-filtering.label',
		defaultMessage: 'Anisotropic filtering',
	},
	alternateBlocksLabel: {
		id: 'app.settings.game-options.setting.alternate-blocks.label',
		defaultMessage: 'Alternate blocks',
	},
	heldItemTooltipsLabel: {
		id: 'app.settings.game-options.setting.held-item-tooltips.label',
		defaultMessage: 'Held item tooltips',
	},
	useVboLabel: {
		id: 'app.settings.game-options.setting.use-vbo.label',
		defaultMessage: 'Use VBOs',
	},
	forceUnicodeFontLabel: {
		id: 'app.settings.game-options.setting.force-unicode-font.label',
		defaultMessage: 'Force Unicode font',
	},
	japaneseGlyphVariantsLabel: {
		id: 'app.settings.game-options.setting.japanese-glyph-variants.label',
		defaultMessage: 'Japanese glyph variants',
	},
	musicFrequencyLabel: {
		id: 'app.settings.game-options.setting.music-frequency.label',
		defaultMessage: 'Music frequency',
	},
	directionalAudioLabel: {
		id: 'app.settings.game-options.setting.directional-audio.label',
		defaultMessage: 'Directional audio',
	},
	invertHorizontalMouseLabel: {
		id: 'app.settings.game-options.setting.invert-horizontal-mouse.label',
		defaultMessage: 'Invert horizontal mouse',
	},
	toggleAttackLabel: {
		id: 'app.settings.game-options.setting.toggle-attack.label',
		defaultMessage: 'Toggle attack',
	},
	toggleUseLabel: {
		id: 'app.settings.game-options.setting.toggle-use.label',
		defaultMessage: 'Toggle use',
	},
	mouseWheelSensitivityLabel: {
		id: 'app.settings.game-options.setting.mouse-wheel-sensitivity.label',
		defaultMessage: 'Mouse wheel sensitivity',
	},
	rawMouseInputLabel: {
		id: 'app.settings.game-options.setting.raw-mouse-input.label',
		defaultMessage: 'Raw mouse input',
	},
	touchscreenLabel: {
		id: 'app.settings.game-options.setting.touchscreen.label',
		defaultMessage: 'Touchscreen mode',
	},
	allowCursorChangesLabel: {
		id: 'app.settings.game-options.setting.allow-cursor-changes.label',
		defaultMessage: 'Allow cursor changes',
	},
	sprintWindowLabel: {
		id: 'app.settings.game-options.setting.sprint-window.label',
		defaultMessage: 'Sprint window',
	},
	operatorItemsTabLabel: {
		id: 'app.settings.game-options.setting.operator-items-tab.label',
		defaultMessage: 'Operator items tab',
	},
	ctrlClickRightClickLabel: {
		id: 'app.settings.game-options.setting.ctrl-click-right-click.label',
		defaultMessage: 'Control-click as right-click',
	},
	quitShortcutsLabel: {
		id: 'app.settings.game-options.setting.quit-shortcuts.label',
		defaultMessage: 'Quit shortcuts',
	},
	chatWidthLabel: {
		id: 'app.settings.game-options.setting.chat-width.label',
		defaultMessage: 'Chat width',
	},
	focusedChatHeightLabel: {
		id: 'app.settings.game-options.setting.focused-chat-height.label',
		defaultMessage: 'Focused chat height',
	},
	unfocusedChatHeightLabel: {
		id: 'app.settings.game-options.setting.unfocused-chat-height.label',
		defaultMessage: 'Unfocused chat height',
	},
	chatLineSpacingLabel: {
		id: 'app.settings.game-options.setting.chat-line-spacing.label',
		defaultMessage: 'Chat line spacing',
	},
	chatDelayLabel: {
		id: 'app.settings.game-options.setting.chat-delay.label',
		defaultMessage: 'Chat delay',
	},
	textBackgroundOpacityLabel: {
		id: 'app.settings.game-options.setting.text-background-opacity.label',
		defaultMessage: 'Text background opacity',
	},
	chatBackgroundOnlyLabel: {
		id: 'app.settings.game-options.setting.chat-background-only.label',
		defaultMessage: 'Chat background only',
	},
	autoSuggestionsLabel: {
		id: 'app.settings.game-options.setting.auto-suggestions.label',
		defaultMessage: 'Command suggestions',
	},
	secureChatOnlyLabel: {
		id: 'app.settings.game-options.setting.secure-chat-only.label',
		defaultMessage: 'Only show secure chat',
	},
	saveChatDraftsLabel: {
		id: 'app.settings.game-options.setting.save-chat-drafts.label',
		defaultMessage: 'Save chat drafts',
	},
	hideMatchedNamesLabel: {
		id: 'app.settings.game-options.setting.hide-matched-names.label',
		defaultMessage: 'Hide matched names',
	},
	chatPreviewLabel: {
		id: 'app.settings.game-options.setting.chat-preview.label',
		defaultMessage: 'Chat preview',
	},
	fovEffectsLabel: {
		id: 'app.settings.game-options.setting.fov-effects.label',
		defaultMessage: 'FOV effects',
	},
	screenEffectsLabel: {
		id: 'app.settings.game-options.setting.screen-effects.label',
		defaultMessage: 'Screen effects',
	},
	darknessPulsingLabel: {
		id: 'app.settings.game-options.setting.darkness-pulsing.label',
		defaultMessage: 'Darkness pulsing',
	},
	damageTiltLabel: {
		id: 'app.settings.game-options.setting.damage-tilt.label',
		defaultMessage: 'Damage tilt',
	},
	glintSpeedLabel: {
		id: 'app.settings.game-options.setting.glint-speed.label',
		defaultMessage: 'Glint speed',
	},
	glintStrengthLabel: {
		id: 'app.settings.game-options.setting.glint-strength.label',
		defaultMessage: 'Glint strength',
	},
	hideLightningFlashesLabel: {
		id: 'app.settings.game-options.setting.hide-lightning-flashes.label',
		defaultMessage: 'Hide lightning flashes',
	},
	hideSplashTextsLabel: {
		id: 'app.settings.game-options.setting.hide-splash-texts.label',
		defaultMessage: 'Hide splash texts',
	},
	highContrastOutlineLabel: {
		id: 'app.settings.game-options.setting.high-contrast-outline.label',
		defaultMessage: 'High contrast block outline',
	},
	narratorHotkeyLabel: {
		id: 'app.settings.game-options.setting.narrator-hotkey.label',
		defaultMessage: 'Narrator hotkey',
	},
	autosaveIndicatorLabel: {
		id: 'app.settings.game-options.setting.autosave-indicator.label',
		defaultMessage: 'Autosave indicator',
	},
	panoramaSpeedLabel: {
		id: 'app.settings.game-options.setting.panorama-speed.label',
		defaultMessage: 'Panorama speed',
	},
	menuBackgroundBlurLabel: {
		id: 'app.settings.game-options.setting.menu-background-blur.label',
		defaultMessage: 'Menu background blur',
	},
	rotateWithMinecartLabel: {
		id: 'app.settings.game-options.setting.rotate-with-minecart.label',
		defaultMessage: 'Rotate with minecart',
	},
	leftSleeveLabel: {
		id: 'app.settings.game-options.setting.left-sleeve.label',
		defaultMessage: 'Left sleeve',
	},
	rightSleeveLabel: {
		id: 'app.settings.game-options.setting.right-sleeve.label',
		defaultMessage: 'Right sleeve',
	},
	leftPantsLegLabel: {
		id: 'app.settings.game-options.setting.left-pants-leg.label',
		defaultMessage: 'Left pants leg',
	},
	rightPantsLegLabel: {
		id: 'app.settings.game-options.setting.right-pants-leg.label',
		defaultMessage: 'Right pants leg',
	},
	hideServerAddressLabel: {
		id: 'app.settings.game-options.setting.hide-server-address.label',
		defaultMessage: 'Hide server address',
	},
	serverTexturesLabel: {
		id: 'app.settings.game-options.setting.server-textures.label',
		defaultMessage: 'Server textures',
	},
	snooperLabel: {
		id: 'app.settings.game-options.setting.snooper.label',
		defaultMessage: 'Snooper',
	},
	extraTelemetryLabel: {
		id: 'app.settings.game-options.setting.extra-telemetry.label',
		defaultMessage: 'Optional telemetry',
	},
	inGameNotificationsLabel: {
		id: 'app.settings.game-options.setting.in-game-notifications.label',
		defaultMessage: 'In-game notifications',
	},
	sharePresenceLabel: {
		id: 'app.settings.game-options.setting.share-presence.label',
		defaultMessage: 'Share presence',
	},
})

const catalogKeyMessages = defineMessages({
	smoothCameraLabel: {
		id: 'app.settings.game-options.setting.key-smooth-camera.label',
		defaultMessage: 'Toggle cinematic camera',
	},
	spectatorOutlinesLabel: {
		id: 'app.settings.game-options.setting.key-spectator-outlines.label',
		defaultMessage: 'Highlight spectators',
	},
	saveToolbarLabel: {
		id: 'app.settings.game-options.setting.key-save-toolbar.label',
		defaultMessage: 'Save toolbar',
	},
	loadToolbarLabel: {
		id: 'app.settings.game-options.setting.key-load-toolbar.label',
		defaultMessage: 'Load toolbar',
	},
	socialInteractionsLabel: {
		id: 'app.settings.game-options.setting.key-social-interactions.label',
		defaultMessage: 'Social interactions',
	},
	quickActionsLabel: {
		id: 'app.settings.game-options.setting.key-quick-actions.label',
		defaultMessage: 'Quick actions',
	},
	spectatorHotbarLabel: {
		id: 'app.settings.game-options.setting.key-spectator-hotbar.label',
		defaultMessage: 'Spectator hotbar',
	},
	friendsLabel: {
		id: 'app.settings.game-options.setting.key-friends.label',
		defaultMessage: 'Friends',
	},
	toggleGuiLabel: {
		id: 'app.settings.game-options.setting.key-toggle-gui.label',
		defaultMessage: 'Toggle HUD',
	},
	toggleSpectatorShaderLabel: {
		id: 'app.settings.game-options.setting.key-toggle-spectator-shader.label',
		defaultMessage: 'Toggle spectator shader',
	},
	hotbar1Label: {
		id: 'app.settings.game-options.setting.key-hotbar-1.label',
		defaultMessage: 'Hotbar 1',
	},
	hotbar2Label: {
		id: 'app.settings.game-options.setting.key-hotbar-2.label',
		defaultMessage: 'Hotbar 2',
	},
	hotbar3Label: {
		id: 'app.settings.game-options.setting.key-hotbar-3.label',
		defaultMessage: 'Hotbar 3',
	},
	hotbar4Label: {
		id: 'app.settings.game-options.setting.key-hotbar-4.label',
		defaultMessage: 'Hotbar 4',
	},
	hotbar5Label: {
		id: 'app.settings.game-options.setting.key-hotbar-5.label',
		defaultMessage: 'Hotbar 5',
	},
	hotbar6Label: {
		id: 'app.settings.game-options.setting.key-hotbar-6.label',
		defaultMessage: 'Hotbar 6',
	},
	hotbar7Label: {
		id: 'app.settings.game-options.setting.key-hotbar-7.label',
		defaultMessage: 'Hotbar 7',
	},
	hotbar8Label: {
		id: 'app.settings.game-options.setting.key-hotbar-8.label',
		defaultMessage: 'Hotbar 8',
	},
	hotbar9Label: {
		id: 'app.settings.game-options.setting.key-hotbar-9.label',
		defaultMessage: 'Hotbar 9',
	},
	debugOverlayLabel: {
		id: 'app.settings.game-options.setting.key-debug-overlay.label',
		defaultMessage: 'Debug overlay',
	},
	debugModifierLabel: {
		id: 'app.settings.game-options.setting.key-debug-modifier.label',
		defaultMessage: 'Debug modifier',
	},
	debugReloadChunksLabel: {
		id: 'app.settings.game-options.setting.key-debug-reload-chunks.label',
		defaultMessage: 'Reload chunks',
	},
	debugHitboxesLabel: {
		id: 'app.settings.game-options.setting.key-debug-hitboxes.label',
		defaultMessage: 'Show hitboxes',
	},
	debugClearChatLabel: {
		id: 'app.settings.game-options.setting.key-debug-clear-chat.label',
		defaultMessage: 'Clear chat',
	},
	debugCrashLabel: {
		id: 'app.settings.game-options.setting.key-debug-crash.label',
		defaultMessage: 'Trigger debug crash',
	},
	debugChunkBordersLabel: {
		id: 'app.settings.game-options.setting.key-debug-chunk-borders.label',
		defaultMessage: 'Show chunk borders',
	},
	debugAdvancedTooltipsLabel: {
		id: 'app.settings.game-options.setting.key-debug-advanced-tooltips.label',
		defaultMessage: 'Show advanced tooltips',
	},
	debugCopyRecreateCommandLabel: {
		id: 'app.settings.game-options.setting.key-debug-copy-recreate-command.label',
		defaultMessage: 'Copy recreate command',
	},
	debugSpectateLabel: {
		id: 'app.settings.game-options.setting.key-debug-spectate.label',
		defaultMessage: 'Spectate entity',
	},
	debugSwitchGameModeLabel: {
		id: 'app.settings.game-options.setting.key-debug-switch-game-mode.label',
		defaultMessage: 'Switch game mode',
	},
	debugOptionsLabel: {
		id: 'app.settings.game-options.setting.key-debug-options.label',
		defaultMessage: 'Debug options',
	},
	debugFocusPauseLabel: {
		id: 'app.settings.game-options.setting.key-debug-focus-pause.label',
		defaultMessage: 'Pause on lost focus',
	},
	debugDumpDynamicTexturesLabel: {
		id: 'app.settings.game-options.setting.key-debug-dump-dynamic-textures.label',
		defaultMessage: 'Dump dynamic textures',
	},
	debugReloadResourcePacksLabel: {
		id: 'app.settings.game-options.setting.key-debug-reload-resource-packs.label',
		defaultMessage: 'Reload resource packs',
	},
	debugProfilingLabel: {
		id: 'app.settings.game-options.setting.key-debug-profiling.label',
		defaultMessage: 'Start profiling',
	},
	debugCopyLocationLabel: {
		id: 'app.settings.game-options.setting.key-debug-copy-location.label',
		defaultMessage: 'Copy location',
	},
	debugDumpVersionLabel: {
		id: 'app.settings.game-options.setting.key-debug-dump-version.label',
		defaultMessage: 'Dump version',
	},
	debugProfilingChartLabel: {
		id: 'app.settings.game-options.setting.key-debug-profiling-chart.label',
		defaultMessage: 'Profiling chart',
	},
	debugFpsChartsLabel: {
		id: 'app.settings.game-options.setting.key-debug-fps-charts.label',
		defaultMessage: 'FPS charts',
	},
	debugNetworkChartsLabel: {
		id: 'app.settings.game-options.setting.key-debug-network-charts.label',
		defaultMessage: 'Network charts',
	},
	debugLightmapTextureLabel: {
		id: 'app.settings.game-options.setting.key-debug-lightmap-texture.label',
		defaultMessage: 'Lightmap texture',
	},
	debugImprovedTransparencyLabel: {
		id: 'app.settings.game-options.setting.key-debug-improved-transparency.label',
		defaultMessage: 'Improved transparency debug view',
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
	far: { id: 'app.settings.game-options.choice.far', defaultMessage: 'Far' },
	normal: { id: 'app.settings.game-options.choice.normal', defaultMessage: 'Normal' },
	short: { id: 'app.settings.game-options.choice.short', defaultMessage: 'Short' },
	tiny: { id: 'app.settings.game-options.choice.tiny', defaultMessage: 'Tiny' },
	maxFps: { id: 'app.settings.game-options.choice.max-fps', defaultMessage: 'Max FPS' },
	balanced: { id: 'app.settings.game-options.choice.balanced', defaultMessage: 'Balanced' },
	powerSaver: { id: 'app.settings.game-options.choice.power-saver', defaultMessage: 'Power saver' },
	whileAfk: { id: 'app.settings.game-options.choice.while-afk', defaultMessage: 'While AFK' },
	whenMinimized: {
		id: 'app.settings.game-options.choice.when-minimized',
		defaultMessage: 'When minimized',
	},
	none: { id: 'app.settings.game-options.choice.none', defaultMessage: 'None' },
	byPlayer: { id: 'app.settings.game-options.choice.by-player', defaultMessage: 'By player' },
	nearby: { id: 'app.settings.game-options.choice.nearby', defaultMessage: 'Nearby' },
	crosshair: { id: 'app.settings.game-options.choice.crosshair', defaultMessage: 'Crosshair' },
	hotbar: { id: 'app.settings.game-options.choice.hotbar', defaultMessage: 'Hotbar' },
	constant: { id: 'app.settings.game-options.choice.constant', defaultMessage: 'Constant' },
	default: { id: 'app.settings.game-options.choice.default', defaultMessage: 'Default' },
	frequent: { id: 'app.settings.game-options.choice.frequent', defaultMessage: 'Frequent' },
	limited: { id: 'app.settings.game-options.choice.limited', defaultMessage: 'Limited' },
	openGl: { id: 'app.settings.game-options.choice.opengl', defaultMessage: 'OpenGL' },
	vulkan: { id: 'app.settings.game-options.choice.vulkan', defaultMessage: 'Vulkan' },
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
		brightness: { label: catalogSettingMessages.brightnessLabel },
		legacy_view_distance: { label: catalogSettingMessages.legacyViewDistanceLabel },
		entity_distance: { label: catalogSettingMessages.entityDistanceLabel },
		debug_gui_scale: { label: catalogSettingMessages.debugGuiScaleLabel },
		graphics_backend: { label: catalogSettingMessages.graphicsBackendLabel },
		cloud_range: { label: catalogSettingMessages.cloudRangeLabel },
		exclusive_fullscreen: { label: catalogSettingMessages.exclusiveFullscreenLabel },
		mac_fullscreen_menu: { label: catalogSettingMessages.macFullscreenMenuLabel },
		legacy_framerate_limit: { label: catalogSettingMessages.legacyFramerateLimitLabel },
		inactivity_framerate_limit: {
			label: catalogSettingMessages.inactivityFramerateLimitLabel,
		},
		prioritize_chunk_updates: { label: catalogSettingMessages.prioritizeChunkUpdatesLabel },
		attack_indicator: { label: catalogSettingMessages.attackIndicatorLabel },
		reduced_debug_info: { label: catalogSettingMessages.reducedDebugInfoLabel },
		chunk_fade_time: { label: catalogSettingMessages.chunkFadeTimeLabel },
		cutout_leaves: { label: catalogSettingMessages.cutoutLeavesLabel },
		improved_transparency: { label: catalogSettingMessages.improvedTransparencyLabel },
		texture_filtering: { label: catalogSettingMessages.textureFilteringLabel },
		anisotropy: { label: catalogSettingMessages.anisotropyLabel },
		vignette: { label: catalogSettingMessages.vignetteLabel },
		weather_radius: { label: catalogSettingMessages.weatherRadiusLabel },
		advanced_opengl: { label: catalogSettingMessages.advancedOpenGlLabel },
		anaglyph_3d: { label: catalogSettingMessages.anaglyph3dLabel },
		anisotropic_filtering: { label: catalogSettingMessages.anisotropicFilteringLabel },
		alternate_blocks: { label: catalogSettingMessages.alternateBlocksLabel },
		held_item_tooltips: { label: catalogSettingMessages.heldItemTooltipsLabel },
		use_vbo: { label: catalogSettingMessages.useVboLabel },
		force_unicode_font: { label: catalogSettingMessages.forceUnicodeFontLabel },
		japanese_glyph_variants: { label: catalogSettingMessages.japaneseGlyphVariantsLabel },
		music_frequency: { label: catalogSettingMessages.musicFrequencyLabel },
		directional_audio: { label: catalogSettingMessages.directionalAudioLabel },
		invert_horizontal_mouse: { label: catalogSettingMessages.invertHorizontalMouseLabel },
		toggle_attack: { label: catalogSettingMessages.toggleAttackLabel },
		toggle_use: { label: catalogSettingMessages.toggleUseLabel },
		mouse_wheel_sensitivity: { label: catalogSettingMessages.mouseWheelSensitivityLabel },
		raw_mouse_input: { label: catalogSettingMessages.rawMouseInputLabel },
		touchscreen: { label: catalogSettingMessages.touchscreenLabel },
		allow_cursor_changes: { label: catalogSettingMessages.allowCursorChangesLabel },
		sprint_window: { label: catalogSettingMessages.sprintWindowLabel },
		operator_items_tab: { label: catalogSettingMessages.operatorItemsTabLabel },
		ctrl_click_right_click: { label: catalogSettingMessages.ctrlClickRightClickLabel },
		quit_shortcuts: { label: catalogSettingMessages.quitShortcutsLabel },
		chat_width: { label: catalogSettingMessages.chatWidthLabel },
		focused_chat_height: { label: catalogSettingMessages.focusedChatHeightLabel },
		unfocused_chat_height: { label: catalogSettingMessages.unfocusedChatHeightLabel },
		chat_line_spacing: { label: catalogSettingMessages.chatLineSpacingLabel },
		chat_delay: { label: catalogSettingMessages.chatDelayLabel },
		text_background_opacity: { label: catalogSettingMessages.textBackgroundOpacityLabel },
		chat_background_only: { label: catalogSettingMessages.chatBackgroundOnlyLabel },
		auto_suggestions: { label: catalogSettingMessages.autoSuggestionsLabel },
		secure_chat_only: { label: catalogSettingMessages.secureChatOnlyLabel },
		save_chat_drafts: { label: catalogSettingMessages.saveChatDraftsLabel },
		hide_matched_names: { label: catalogSettingMessages.hideMatchedNamesLabel },
		chat_preview: { label: catalogSettingMessages.chatPreviewLabel },
		fov_effects: { label: catalogSettingMessages.fovEffectsLabel },
		screen_effects: { label: catalogSettingMessages.screenEffectsLabel },
		darkness_pulsing: { label: catalogSettingMessages.darknessPulsingLabel },
		damage_tilt: { label: catalogSettingMessages.damageTiltLabel },
		glint_speed: { label: catalogSettingMessages.glintSpeedLabel },
		glint_strength: { label: catalogSettingMessages.glintStrengthLabel },
		hide_lightning_flashes: { label: catalogSettingMessages.hideLightningFlashesLabel },
		hide_splash_texts: { label: catalogSettingMessages.hideSplashTextsLabel },
		high_contrast_outline: { label: catalogSettingMessages.highContrastOutlineLabel },
		narrator_hotkey: { label: catalogSettingMessages.narratorHotkeyLabel },
		autosave_indicator: { label: catalogSettingMessages.autosaveIndicatorLabel },
		panorama_speed: { label: catalogSettingMessages.panoramaSpeedLabel },
		menu_background_blur: { label: catalogSettingMessages.menuBackgroundBlurLabel },
		rotate_with_minecart: { label: catalogSettingMessages.rotateWithMinecartLabel },
		left_sleeve: { label: catalogSettingMessages.leftSleeveLabel },
		right_sleeve: { label: catalogSettingMessages.rightSleeveLabel },
		left_pants_leg: { label: catalogSettingMessages.leftPantsLegLabel },
		right_pants_leg: { label: catalogSettingMessages.rightPantsLegLabel },
		hide_server_address: { label: catalogSettingMessages.hideServerAddressLabel },
		server_textures: { label: catalogSettingMessages.serverTexturesLabel },
		snooper: { label: catalogSettingMessages.snooperLabel },
		extra_telemetry: { label: catalogSettingMessages.extraTelemetryLabel },
		in_game_notifications: { label: catalogSettingMessages.inGameNotificationsLabel },
		share_presence: { label: catalogSettingMessages.sharePresenceLabel },
		'key.smooth_camera': { label: catalogKeyMessages.smoothCameraLabel },
		'key.spectator_outlines': { label: catalogKeyMessages.spectatorOutlinesLabel },
		'key.save_toolbar': { label: catalogKeyMessages.saveToolbarLabel },
		'key.load_toolbar': { label: catalogKeyMessages.loadToolbarLabel },
		'key.social_interactions': { label: catalogKeyMessages.socialInteractionsLabel },
		'key.quick_actions': { label: catalogKeyMessages.quickActionsLabel },
		'key.spectator_hotbar': { label: catalogKeyMessages.spectatorHotbarLabel },
		'key.friends': { label: catalogKeyMessages.friendsLabel },
		'key.toggle_gui': { label: catalogKeyMessages.toggleGuiLabel },
		'key.toggle_spectator_shader': { label: catalogKeyMessages.toggleSpectatorShaderLabel },
		'key.hotbar.1': { label: catalogKeyMessages.hotbar1Label },
		'key.hotbar.2': { label: catalogKeyMessages.hotbar2Label },
		'key.hotbar.3': { label: catalogKeyMessages.hotbar3Label },
		'key.hotbar.4': { label: catalogKeyMessages.hotbar4Label },
		'key.hotbar.5': { label: catalogKeyMessages.hotbar5Label },
		'key.hotbar.6': { label: catalogKeyMessages.hotbar6Label },
		'key.hotbar.7': { label: catalogKeyMessages.hotbar7Label },
		'key.hotbar.8': { label: catalogKeyMessages.hotbar8Label },
		'key.hotbar.9': { label: catalogKeyMessages.hotbar9Label },
		'key.debug.overlay': { label: catalogKeyMessages.debugOverlayLabel },
		'key.debug.modifier': { label: catalogKeyMessages.debugModifierLabel },
		'key.debug.reload_chunks': { label: catalogKeyMessages.debugReloadChunksLabel },
		'key.debug.hitboxes': { label: catalogKeyMessages.debugHitboxesLabel },
		'key.debug.clear_chat': { label: catalogKeyMessages.debugClearChatLabel },
		'key.debug.crash': { label: catalogKeyMessages.debugCrashLabel },
		'key.debug.chunk_borders': { label: catalogKeyMessages.debugChunkBordersLabel },
		'key.debug.advanced_tooltips': {
			label: catalogKeyMessages.debugAdvancedTooltipsLabel,
		},
		'key.debug.copy_recreate_command': {
			label: catalogKeyMessages.debugCopyRecreateCommandLabel,
		},
		'key.debug.spectate': { label: catalogKeyMessages.debugSpectateLabel },
		'key.debug.switch_game_mode': { label: catalogKeyMessages.debugSwitchGameModeLabel },
		'key.debug.options': { label: catalogKeyMessages.debugOptionsLabel },
		'key.debug.focus_pause': { label: catalogKeyMessages.debugFocusPauseLabel },
		'key.debug.dump_dynamic_textures': {
			label: catalogKeyMessages.debugDumpDynamicTexturesLabel,
		},
		'key.debug.reload_resource_packs': {
			label: catalogKeyMessages.debugReloadResourcePacksLabel,
		},
		'key.debug.profiling': { label: catalogKeyMessages.debugProfilingLabel },
		'key.debug.copy_location': { label: catalogKeyMessages.debugCopyLocationLabel },
		'key.debug.dump_version': { label: catalogKeyMessages.debugDumpVersionLabel },
		'key.debug.profiling_chart': { label: catalogKeyMessages.debugProfilingChartLabel },
		'key.debug.fps_charts': { label: catalogKeyMessages.debugFpsChartsLabel },
		'key.debug.network_charts': { label: catalogKeyMessages.debugNetworkChartsLabel },
		'key.debug.lightmap_texture': { label: catalogKeyMessages.debugLightmapTextureLabel },
		'key.debug.improved_transparency': {
			label: catalogKeyMessages.debugImprovedTransparencyLabel,
		},
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
	'legacy_view_distance:0': choiceMessages.far,
	'legacy_view_distance:1': choiceMessages.normal,
	'legacy_view_distance:2': choiceMessages.short,
	'legacy_view_distance:3': choiceMessages.tiny,
	'legacy_framerate_limit:0': choiceMessages.maxFps,
	'legacy_framerate_limit:1': choiceMessages.balanced,
	'legacy_framerate_limit:2': choiceMessages.powerSaver,
	'inactivity_framerate_limit:afk': choiceMessages.whileAfk,
	'inactivity_framerate_limit:minimized': choiceMessages.whenMinimized,
	'prioritize_chunk_updates:0': choiceMessages.none,
	'prioritize_chunk_updates:1': choiceMessages.byPlayer,
	'prioritize_chunk_updates:2': choiceMessages.nearby,
	'attack_indicator:0': choiceMessages.off,
	'attack_indicator:1': choiceMessages.crosshair,
	'attack_indicator:2': choiceMessages.hotbar,
	'chat_preview:0': choiceMessages.off,
	'chat_preview:1': choiceMessages.commandsOnly,
	'chat_preview:2': choiceMessages.on,
	'music_frequency:CONSTANT': choiceMessages.constant,
	'music_frequency:DEFAULT': choiceMessages.default,
	'music_frequency:FREQUENT': choiceMessages.frequent,
	'share_presence:all': choiceMessages.all,
	'share_presence:limited': choiceMessages.limited,
	'share_presence:none': choiceMessages.none,
	'graphics_backend:default': choiceMessages.default,
	'graphics_backend:opengl': choiceMessages.openGl,
	'graphics_backend:vulkan': choiceMessages.vulkan,
}

const validationMessages: Record<GameOptionValidationError, MessageDescriptor> = {
	missing_value: presentationMessages.validationMissingValue,
	no_compatible_instances: presentationMessages.validationNoCompatibleInstances,
	invalid_value: presentationMessages.validationInvalidValue,
	changed_since_opened: presentationMessages.validationChangedSinceOpened,
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
