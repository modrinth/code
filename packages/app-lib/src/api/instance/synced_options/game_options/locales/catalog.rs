/// Translation keys mapped to settings.
pub(super) fn translation_keys(raw: &str, vanilla: bool) -> Vec<String> {
	if let Some(key) = raw.strip_prefix("key_") { return vec![key.to_owned()]; }
	if !vanilla { return Vec::new(); }
	if let Some(key) = raw.strip_prefix("soundCategory_") { return vec![format!("soundCategory.{key}")]; }
	if let Some(key) = raw.strip_prefix("modelPart_") { return vec![format!("options.modelPart.{key}")]; }
	let aliases: &[&str] = match raw {
		"lang" => &["options.language"],
		"mouseSensitivity" => &["options.sensitivity"],
		"enableVsync" => &["options.vsync"],
		"bobView" => &["options.viewBobbing"],
		"maxFps" | "fpsLimit" => &["options.framerateLimit"],
		"fancyGraphics" | "graphicsMode" => &["options.graphics"],
		"graphicsPreset" => &["options.graphics.preset"],
		"renderClouds" => &["options.renderClouds", "options.clouds"],
		"cloudRange" => &["options.renderCloudsDistance"],
		"viewDistance" => &["options.renderDistance"],
		"invertYMouse" => &["options.invertMouseY", "options.invertMouse"],
		"invertXMouse" => &["options.invertMouseX"],
		"toggleCrouch" => &["key.sneak"],
		"toggleSprint" => &["key.sprint"],
		"toggleAttack" => &["key.attack"],
		"toggleUse" => &["key.use"],
		"chatVisibility" => &["options.chat.visibility"],
		"chatColors" => &["options.chat.color"],
		"chatLinks" => &["options.chat.links"],
		"chatLinksPrompt" => &["options.chat.links.prompt"],
		"chatOpacity" => &["options.chat.opacity"],
		"chatScale" => &["options.chat.scale"],
		"chatWidth" => &["options.chat.width"],
		"chatHeightFocused" => &["options.chat.height.focused"],
		"chatHeightUnfocused" => &["options.chat.height.unfocused"],
		"chatLineSpacing" => &["options.chat.line_spacing"],
		"chatDelay" => &["options.chat.delay_instant"],
		"autoSuggestions" => &["options.autoSuggestCommands"],
		"textBackgroundOpacity" => &["options.accessibility.text_background_opacity"],
		"backgroundForChatOnly" => &["options.accessibility.text_background"],
		"darkMojangStudiosBackground" => &["options.darkMojangStudiosBackgroundColor"],
		"highContrast" => &["options.accessibility.high_contrast"],
		"highContrastBlockOutline" => &["options.accessibility.high_contrast_block_outline"],
		"menuBackgroundBlurriness" => &["options.accessibility.menu_background_blurriness"],
		"notificationDisplayTime" => &["options.notifications.display_time"],
		"panoramaScrollSpeed" => &["options.accessibility.panorama_speed"],
		"narratorHotkey" => &["options.accessibility.narrator_hotkey"],
		"showAutosaveIndicator" => &["options.autosaveIndicator"],
		"soundDevice" => &["options.audioDevice"],
		"musicFrequency" => &["options.music_frequency"],
		"music" => &["soundCategory.music", "options.music"],
		"sound" => &["soundCategory.master", "options.sound"],
		"realmsNotifications" => &["options.realmsNotifications.button"],
		"telemetryOptInExtra" => &["options.telemetry.button"],
		"saveChatDrafts" => &["options.chat.drafts"],
		"chunkSectionFadeInTime" => &["options.chunkFade"],
		"maxAnisotropyBit" => &["options.maxAnisotropy"],
		"preferredGraphicsBackend" => &["options.graphicsApi"],
		_ => &[],
	};
	aliases.iter().map(|key| (*key).to_owned())
		.chain(std::iter::once(format!("options.{raw}"))).collect()
}

pub(super) fn choice_keys(option_id: &str) -> &'static [(&'static str, &'static str)] {
	match option_id {
		"graphics" => &[("fast", "options.graphics.fast"), ("fancy", "options.graphics.fancy"), ("fabulous", "options.graphics.fabulous"), ("custom", "options.graphics.custom")],
		"clouds" => &[("false", "options.off"), ("fast", "options.clouds.fast"), ("true", "options.clouds.fancy")],
		"particles" => &[("0", "options.particles.all"), ("1", "options.particles.decreased"), ("2", "options.particles.minimal")],
		"chat_visibility" => &[("0", "options.chat.visibility.full"), ("1", "options.chat.visibility.system"), ("2", "options.chat.visibility.hidden")],
		"main_hand" => &[("left", "options.mainHand.left"), ("right", "options.mainHand.right")],
		"narrator" => &[
			("0", "options.narrator.off"),
			("1", "options.narrator.all"),
			("2", "options.narrator.chat"),
			("3", "options.narrator.system"),
		],
		"ambient_occlusion" => &[
			("off", "options.ao.off"),
			("on", "options.on"),
			("minimum", "options.ao.min"),
			("maximum", "options.ao.max"),
		],
		"music_toast" => &[
			("never", "options.musicToast.never"),
			("pause", "options.musicToast.pauseMenu"),
			("pause_and_toast", "options.musicToast.pauseMenuAndToast"),
		],
		"legacy_view_distance" => &[
			("0", "options.renderDistance.far"),
			("1", "options.renderDistance.normal"),
			("2", "options.renderDistance.short"),
			("3", "options.renderDistance.tiny"),
		],
		"legacy_framerate_limit" => &[
			("0", "options.framerateLimit.max"),
			("1", "options.framerateLimit.balanced"),
			("2", "options.framerateLimit.powersaver"),
		],
		"inactivity_framerate_limit" => &[
			("afk", "options.inactivityFpsLimit.afk"),
			("minimized", "options.inactivityFpsLimit.minimized"),
		],
		"prioritize_chunk_updates" => &[
			("0", "options.prioritizeChunkUpdates.none"),
			("1", "options.prioritizeChunkUpdates.byPlayer"),
			("2", "options.prioritizeChunkUpdates.nearby"),
		],
		"attack_indicator" => &[
			("0", "options.off"),
			("1", "options.attack.crosshair"),
			("2", "options.attack.hotbar"),
		],
		"chat_preview" => &[
			("0", "options.off"),
			("1", "options.chatPreview.live"),
			("2", "options.chatPreview.confirm"),
		],
		"music_frequency" => &[
			("CONSTANT", "options.music_frequency.constant"),
			("DEFAULT", "options.music_frequency.default"),
			("FREQUENT", "options.music_frequency.frequent"),
		],
		"share_presence" => &[
			("all", "options.sharePresence.all"),
			("limited", "options.sharePresence.limited"),
			("none", "options.sharePresence.none"),
		],
		"graphics_backend" => &[
			("default", "options.graphicsApi.default"),
			("opengl", "options.graphicsApi.opengl"),
			("vulkan", "options.graphicsApi.vulkan"),
		],
		"texture_filtering" => &[
			("0", "options.textureFiltering.none"),
			("1", "options.textureFiltering.rgss"),
			("2", "options.textureFiltering.anisotropic"),
		],
		_ => &[],
	}
}
