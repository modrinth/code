# options.txt catalog history

## Legacy

- Infdev/Alpha: removed `key_Build`, `key_Load location`, `key_Save location`, and `showFrameRate`.
- Beta 1.1: removed `key_Back`, `key_Chat`, `key_Drop`, `key_Forward`, `key_Inventory`, `key_Jump`, `key_Left`, `key_Right`, `key_Sneak`, and `key_Toggle fog`.
- Beta 1.7/1.8: removed `limitFramerate` and `key_key.fog`.

## 1.4.2

- Added `pauseOnLostFocus`.

## 1.7

- Added `soundCategory_master`, `soundCategory_music`, `soundCategory_record`, `soundCategory_weather`, `soundCategory_block`, `soundCategory_hostile`, `soundCategory_neutral`, `soundCategory_player`, and `soundCategory_ambient`.

## 1.7.2

- Added `forceUnicodeFont`.
- Removed `fpsLimit`, `music`, `skin`, `sound`, and `viewDistance`.

## 1.7.6

- Removed `serverTextures`.
- Moved server resource-pack acceptance to a per-server Prompt/Enabled/Disabled setting.

## 1.8

- Removed `advancedOpengl`, `anisotropicFiltering`, `clouds`, and `showCape`.

## 1.9

- Removed `allowBlockAlternatives`.
- Removed `key_key.streamCommercial`, `key_key.streamPauseUnpause`, `key_key.streamStartStop`, and `key_key.streamToggleMic`.
- Removed `streamBytesPerPixel`, `streamChatEnabled`, `streamChatUserFilter`, `streamCompression`, `streamFps`, `streamKbps`, `streamMicToggleBehavior`, `streamMicVolume`, `streamPreferredServer`, `streamSendMetadata`, and `streamSystemVolume`.

## 1.10

- Added `version`.
- Added `autoJump`.

## 1.12

- Added `narrator`.
- Added `tutorialStep`.
- Added `key_key.saveToolbarActivator`.
- Removed `showInventoryAchievementHint`.

## 1.13

- Added `biomeBlendRadius`.
- Added `autoSuggestions`.
- Added `mouseWheelSensitivity`.
- Added `glDebugVerbosity`.
- Changed numeric LWJGL2 key values to `key.*` and `key.mouse.*` tokens.
- Changed keyboard tokens from `key.*` to `key.keyboard.*`.
- Removed `anaglyph3d`.

## 1.14

- Added `discrete_mouse_scroll`.
- Removed `enableWeakAttacks`, `fboEnable`, `saturation`, and `useVbo`.

## 1.14.4

- Added `rawMouseInput`.

## 1.15

- Added `toggleCrouch`.

## 1.15.2

- Added `skipMultiplayerWarning`.

## 1.16

- Added `entityDistanceScaling`.
- Added `chatLineSpacing`.
- Added `chatDelay`.
- Added `syncChunkWrites`.
- Renamed `key_key.swapHands` to `key_key.swapOffhand`.

## 1.16.2

- Added `fovEffectScale`.
- Added `screenEffectScale`.
- Replaced `fancyGraphics` with numeric `graphicsMode`.

## 1.16.4

- Added `hideMatchedNames`.
- Added `key_key.socialInteractions`.
- Added `joinedFirstServer`.

## 1.17

- Added `darkMojangStudiosBackground`.

## 1.18

- Added `prioritizeChunkUpdates`.
- Added `simulationDistance`.
- Added `soundDevice`.
- Added `hideLightningFlashes`.
- Added `showAutosaveIndicator`.
- Added `allowServerListing`.
- Removed `snooperEnabled`.

## 1.19

- Added `darknessEffectScale`.
- Added `directionalAudio`.
- Added `onlyShowSecureChat`.
- Removed `difficulty`.

## 1.19.3

- Added `operatorItemsTab`.
- Added `panoramaScrollSpeed`.
- Added `telemetryOptInExtra`.
- Removed `chatPreview`.

## 1.19.4

- Added `glintSpeed`.
- Added `damageTiltStrength`.
- Added `onboardAccessibility`.
- Added `highContrast`.
- Removed `heldItemTooltips`.

## 1.20.2

- Added `narratorHotKey`.

## 1.20.3

- Added `hideSplashTexts`.

## 1.20.5

- Added `japaneseGlyphVariants`.
- Added `menuBackgroundBlurriness`.
- Removed `skipRealms32bitWarning`.

## 1.21.1

- Removed `hideBundleTutorial`.

## 1.21.2

- Added `inactivityFpsLimit`.
- Added `rotateWithMinecart`.
- Added `highContrastBlockOutline`.

## 1.21.5

- Added `startedCleanly`.

## 1.21.6

- Added `cloudRange`.
- Added `musicFrequency`.
- Added `key_key.quickActions`.
- Added `soundCategory_ui`.

## 1.21.9

- Added `invertXMouse`.
- Added `toggleAttack`.
- Added `allowCursorChanges`.
- Added `saveChatDrafts`.
- Added `key_key.spectatorHotbar`.

## 1.21.11

- Added `chunkSectionFadeInTime`.
- Added `graphicsPreset`.
- Added `cutoutLeaves`.
- Added `weatherRadius`.
- Added `maxAnisotropyBit`.
- Added `textureFiltering`.
- Added `improvedTransparency`.
- Added `vignette`.
- Added `musicToast`.
- Added `key_key.toggleGui`.
- Added `key_key.debug.overlay`.
- Added `key_key.debug.reloadChunk`.
- Replaced `graphicsMode` with `graphicsPreset` and separate graphics-detail fields.
- Replaced `showNowPlayingToast` with `musicToast`.

## 26.1

- Added `exclusiveFullscreen`.
- Added `key_key.debug.lightmapTexture`.

## 26.2

- Added `preferredGraphicsBackend`.
- Added `inGameNotification`.
- Added `key_key.friends`.
- Removed `touchscreen`.

## 26.3

- Added `debugGuiScale`.
- Added `quitShortcuts`.
- Added `ctrlClickEmulatesRightClick`.
- Added `macFullscreenMenuVisibility`.
- Added `key_key.debug.improvedTransparency`.
- Removed `rawMouseInput`.

## Vanilla migrations

Vanilla does these migrations for us

| options.txt data version | Change                                                                                                                 |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------------- |
| 505                      | Set `useVbo=true`.                                                                                                     |
| 816                      | Lowercase `lang`.                                                                                                      |
| 1344                     | Convert numeric LWJGL2 key mappings to token names; convert `-100`, `-99`, and `-98` to left, right, and middle mouse. |
| 1446                     | Convert `key.*` keyboard tokens to `key.keyboard.*`; retain mouse and scancode forms.                                  |
| 1936                     | Derive text background opacity from chat opacity.                                                                      |
| 2556                     | Convert `fancyGraphics=true` to `graphicsMode=1`; convert other values to `graphicsMode=0`.                            |
| 2558                     | Rename `key_key.swapHands` to `key_key.swapOffhand`.                                                                   |
| 3201                     | Rename `programer_art` to `programmer_art` in `resourcePacks` and `incompatibleResourcePacks`.                         |
| 3214                     | Convert ambient occlusion `0` to `false`; convert `1` and `2` to `true`.                                               |
| 3319                     | Set `onboardAccessibility=false` for upgraded files.                                                                   |
| 3943                     | Convert menu blur from a float to `round(value * 10)`; use `5` on conversion failure.                                  |
| 4651                     | Expand `graphicsMode` into graphics-detail fields and set `graphicsPreset=custom`.                                     |
| 4661                     | Convert `showNowPlayingToast=false` to `musicToast=never`; convert `true` to `musicToast=pause_and_toast`.             |
| 4892                     | Set `preferredGraphicsBackend=default`.                                                                                |
