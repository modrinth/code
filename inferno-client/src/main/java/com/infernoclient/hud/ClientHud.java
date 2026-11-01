package com.infernoclient.hud;

import com.infernoclient.config.ClientConfig;
import com.infernoclient.feature.ClickTracker;
import com.infernoclient.ui.Theme;
import com.infernoclient.ui.Ui;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gl.RenderPipelines;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.network.ClientPlayerEntity;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.entity.EquipmentSlot;
import net.minecraft.entity.effect.StatusEffectInstance;
import net.minecraft.item.ItemStack;
import net.minecraft.util.Identifier;
import net.minecraft.util.hit.BlockHitResult;
import net.minecraft.util.hit.HitResult;

import java.time.LocalTime;
import java.time.format.DateTimeFormatter;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

public final class ClientHud {
	public static final String[] ELEMENTS = {"info", "keystrokes", "armor", "potion", "inventory", "heldItem"};
	private static final EquipmentSlot[] ARMOR = {EquipmentSlot.HEAD, EquipmentSlot.CHEST, EquipmentSlot.LEGS, EquipmentSlot.FEET};
	private static final Identifier HOTBAR = Identifier.ofVanilla("textures/gui/sprites/hud/hotbar.png");
	private static final DateTimeFormatter CLOCK = DateTimeFormatter.ofPattern("HH:mm:ss");

	private static ClientConfig cfg;
	private static final Map<String, int[]> BOUNDS = new LinkedHashMap<>();
	private static double lastX;
	private static double lastZ;
	private static boolean haveLast;
	private static double speed;

	private ClientHud() {
	}

	public static void register(ClientConfig config) {
		cfg = config;
		ClientTickEvents.END_CLIENT_TICK.register(ClientHud::tick);
		HudRenderCallback.EVENT.register((context, tickCounter) -> {
			MinecraftClient client = MinecraftClient.getInstance();
			if (client.player == null || client.options.hudHidden) {
				return;
			}
			render(context, client, false);
		});
	}

	private static void tick(MinecraftClient client) {
		ClientPlayerEntity p = client.player;
		if (p == null) {
			haveLast = false;
			return;
		}
		if (haveLast) {
			speed = Math.hypot(p.getX() - lastX, p.getZ() - lastZ) * 20.0;
		}
		lastX = p.getX();
		lastZ = p.getZ();
		haveLast = true;
	}

	public static int[] bounds(String id) {
		return BOUNDS.get(id);
	}

	public static double getScale(String id) {
		ClientConfig.Hud h = cfg.hud;
		return switch (id) {
			case "info" -> h.infoScale;
			case "keystrokes" -> h.keystrokesScale;
			case "armor" -> h.armorScale;
			case "potion" -> h.potionScale;
			case "inventory" -> h.inventoryScale;
			default -> h.heldItemScale;
		};
	}

	public static void setScale(String id, double v) {
		v = ClientConfig.clamp(v, 0.5, 2.5);
		ClientConfig.Hud h = cfg.hud;
		switch (id) {
			case "info" -> h.infoScale = v;
			case "keystrokes" -> h.keystrokesScale = v;
			case "armor" -> h.armorScale = v;
			case "potion" -> h.potionScale = v;
			case "inventory" -> h.inventoryScale = v;
			default -> h.heldItemScale = v;
		}
	}

	public static void setPosition(String id, int x, int y) {
		ClientConfig.Hud h = cfg.hud;
		switch (id) {
			case "info" -> {
				h.x = x;
				h.y = y;
			}
			case "keystrokes" -> {
				h.keystrokesX = x;
				h.keystrokesY = y;
			}
			case "armor" -> {
				h.armorX = x;
				h.armorY = y;
			}
			case "potion" -> {
				h.potionX = x;
				h.potionY = y;
			}
			case "inventory" -> {
				h.inventoryX = x;
				h.inventoryY = y;
			}
			default -> {
				h.heldItemX = x;
				h.heldItemY = y;
			}
		}
	}

	private static int[] position(String id, int w, int h) {
		ClientConfig.Hud c = cfg.hud;
		return switch (id) {
			case "info" -> new int[]{c.x < 0 ? 8 : c.x, c.y < 0 ? 8 : c.y};
			case "keystrokes" -> new int[]{c.keystrokesX < 0 ? 8 : c.keystrokesX, c.keystrokesY < 0 ? h - 90 : c.keystrokesY};
			case "armor" -> new int[]{c.armorX < 0 ? w / 2 - 91 - 29 - 4 - 82 : c.armorX, c.armorY < 0 ? h - 22 : c.armorY};
			case "potion" -> new int[]{c.potionX < 0 ? w - 128 : c.potionX, c.potionY < 0 ? 8 : c.potionY};
			case "inventory" -> new int[]{c.inventoryX < 0 ? w - 176 : c.inventoryX, c.inventoryY < 0 ? h - 86 : c.inventoryY};
			default -> new int[]{c.heldItemX < 0 ? 8 : c.heldItemX, c.heldItemY < 0 ? h / 2 : c.heldItemY};
		};
	}

	public static void render(DrawContext ctx, MinecraftClient mc, boolean preview) {
		ClientPlayerEntity p = mc.player;
		if (p == null) {
			return;
		}
		BOUNDS.clear();
		ClientConfig.Hud h = cfg.hud;
		int sw = ctx.getScaledWindowWidth();
		int sh = ctx.getScaledWindowHeight();

		if (preview || h.info) {
			List<String> lines = infoLines(mc, p);
			if (preview && lines.isEmpty()) {
				lines.add("Info panel");
			}
			if (!lines.isEmpty()) {
				drawElement(ctx, "info", sw, sh, (float) h.infoScale, (c) -> drawInfo(c, lines));
			}
		}
		if (preview || h.keystrokes) {
			drawElement(ctx, "keystrokes", sw, sh, (float) h.keystrokesScale, ClientHud::drawKeystrokes);
		}
		if (preview || h.armor) {
			drawElement(ctx, "armor", sw, sh, (float) h.armorScale, c -> drawArmor(c, p, preview));
		}
		if (preview || h.potionEffects) {
			List<StatusEffectInstance> effects = new ArrayList<>(p.getStatusEffects());
			if (!effects.isEmpty() || preview) {
				drawElement(ctx, "potion", sw, sh, (float) h.potionScale, c -> drawEffects(c, effects));
			}
		}
		if (preview || h.inventory) {
			drawElement(ctx, "inventory", sw, sh, (float) h.inventoryScale, c -> drawInventory(c, p));
		}
		if (preview || h.heldItem) {
			ItemStack held = p.getMainHandStack();
			if (!held.isEmpty() || preview) {
				drawElement(ctx, "heldItem", sw, sh, (float) h.heldItemScale, c -> drawHeld(c, held));
			}
		}
		if (!preview && h.durabilityAlert) {
			drawDurabilityAlert(ctx, p, sw);
		}
	}

	private interface Painter {
		int[] paint(DrawContext ctx);
	}

	private static void drawElement(DrawContext ctx, String id, int sw, int sh, float scale, Painter painter) {
		// Measure at (0,0) first so right-anchored defaults and bounds use the real size.
		int[] pos = position(id, sw, sh);
		ctx.getMatrices().pushMatrix();
		ctx.getMatrices().translate(pos[0], pos[1]);
		ctx.getMatrices().scale(scale, scale);
		int[] size = painter.paint(ctx);
		ctx.getMatrices().popMatrix();
		BOUNDS.put(id, new int[]{pos[0], pos[1], (int) (size[0] * scale), (int) (size[1] * scale)});
	}

	private static void panel(DrawContext ctx, int w, int h) {
		String theme = cfg.hud.theme;
		if ("minimal".equals(theme)) {
			return;
		}
		if ("glass".equals(theme)) {
			Ui.round(ctx, 0, 0, w, h, 0x70000000);
			ctx.fill(1, 0, w - 1, 1, 0x30FFFFFF);
			return;
		}
		Ui.round(ctx, 0, 0, w, h, 0xA0100C0A);
		ctx.fill(0, 1, 2, h - 1, Theme.accent());
	}

	private static boolean shadowed() {
		return "minimal".equals(cfg.hud.theme);
	}

	private static List<String> infoLines(MinecraftClient mc, ClientPlayerEntity p) {
		ClientConfig.Hud h = cfg.hud;
		List<String> out = new ArrayList<>();
		if (h.fps) out.add("FPS  " + mc.getCurrentFps());
		if (h.cps) out.add("CPS  " + ClickTracker.leftCps() + " | " + ClickTracker.rightCps());
		if (h.ping) {
			PlayerListEntry entry = mc.getNetworkHandler() == null ? null : mc.getNetworkHandler().getPlayerListEntry(p.getUuid());
			out.add("Ping  " + (entry == null ? "-" : entry.getLatency() + " ms"));
		}
		if (h.coordinates) out.add(String.format("XYZ  %d  %d  %d", (int) Math.floor(p.getX()), (int) Math.floor(p.getY()), (int) Math.floor(p.getZ())));
		if (h.netherCoordinates && mc.world != null) {
			boolean nether = mc.world.getRegistryKey().getValue().getPath().contains("nether");
			double f = nether ? 8.0 : 1.0 / 8.0;
			out.add(String.format("%s  %d  %d", nether ? "Overworld" : "Nether", (int) Math.floor(p.getX() * f), (int) Math.floor(p.getZ() * f)));
		}
		if (h.chunkCoordinates) out.add("Chunk  " + p.getChunkPos().x + "  " + p.getChunkPos().z);
		if (h.direction) {
			String d = p.getHorizontalFacing().asString();
			out.add("Facing  " + Character.toUpperCase(d.charAt(0)) + d.substring(1));
		}
		if (h.biome && mc.world != null) {
			String biome = mc.world.getBiome(p.getBlockPos()).getKey().map(k -> k.getValue().getPath()).orElse("unknown");
			out.add("Biome  " + pretty(biome));
		}
		if (h.dimension && mc.world != null) out.add("Dimension  " + pretty(mc.world.getRegistryKey().getValue().getPath()));
		if (h.lightLevel && mc.world != null) out.add("Light  " + mc.world.getLightLevel(p.getBlockPos()));
		if (h.speed) out.add(String.format("Speed  %.1f b/s", speed));
		if (h.health) {
			float total = p.getHealth() + p.getAbsorptionAmount();
			out.add(String.format("Health  %.1f", total));
		}
		if (h.targetBlock && mc.crosshairTarget != null && mc.crosshairTarget.getType() == HitResult.Type.BLOCK && mc.world != null) {
			BlockHitResult hit = (BlockHitResult) mc.crosshairTarget;
			out.add("Looking at  " + mc.world.getBlockState(hit.getBlockPos()).getBlock().getName().getString());
		}
		if (h.weather && mc.world != null) out.add("Weather  " + (mc.world.isThundering() ? "Thunder" : mc.world.isRaining() ? "Rain" : "Clear"));
		if (h.day && mc.world != null) out.add("Day  " + (mc.world.getTimeOfDay() / 24000L));
		if (h.memory) {
			Runtime rt = Runtime.getRuntime();
			long used = (rt.totalMemory() - rt.freeMemory()) / 1048576L;
			out.add("Memory  " + used + " / " + rt.maxMemory() / 1048576L + " MB");
		}
		if (h.clock) out.add("Time  " + LocalTime.now().format(CLOCK));
		return out;
	}

	private static String pretty(String path) {
		String[] parts = path.replace('_', ' ').split(" ");
		StringBuilder b = new StringBuilder();
		for (String part : parts) {
			if (part.isEmpty()) continue;
			if (b.length() > 0) b.append(' ');
			b.append(Character.toUpperCase(part.charAt(0))).append(part.substring(1));
		}
		return b.toString();
	}

	private static int[] drawInfo(DrawContext ctx, List<String> lines) {
		int w = 0;
		for (String l : lines) w = Math.max(w, Ui.width(l));
		w += 12;
		int h = lines.size() * 11 + 6;
		panel(ctx, w, h);
		int y = 4;
		for (String l : lines) {
			int split = l.indexOf("  ");
			if (split > 0) {
				String key = l.substring(0, split);
				String val = l.substring(split + 2);
				ctx.drawText(Ui.font(), key, 7, y, Theme.alpha(Theme.hudText(), 170), shadowed());
				ctx.drawText(Ui.font(), val, 7 + Ui.width(key) + 5, y, Theme.hudText(), shadowed());
			} else {
				ctx.drawText(Ui.font(), l, 7, y, Theme.hudText(), shadowed());
			}
			y += 11;
		}
		return new int[]{w, h};
	}

	private static void key(DrawContext ctx, int x, int y, int w, int h, String label, boolean down) {
		Ui.round(ctx, x, y, w, h, down ? Theme.alpha(Theme.accent(), 230) : 0x90100C0A);
		Ui.centered(ctx, label, x + w / 2, y + (h - 8) / 2, down ? 0xFF1A0E08 : Theme.hudText());
	}

	private static int[] drawKeystrokes(DrawContext ctx) {
		MinecraftClient mc = MinecraftClient.getInstance();
		var o = mc.options;
		key(ctx, 20, 0, 18, 18, "W", o.forwardKey.isPressed());
		key(ctx, 0, 20, 18, 18, "A", o.leftKey.isPressed());
		key(ctx, 20, 20, 18, 18, "S", o.backKey.isPressed());
		key(ctx, 40, 20, 18, 18, "D", o.rightKey.isPressed());
		key(ctx, 0, 40, 28, 18, "L " + ClickTracker.leftCps(), o.attackKey.isPressed());
		key(ctx, 30, 40, 28, 18, "R " + ClickTracker.rightCps(), o.useKey.isPressed());
		key(ctx, 0, 60, 58, 8, "", o.jumpKey.isPressed());
		return new int[]{58, 68};
	}

	/** Armor as a horizontal strip cut from the game's own hotbar texture, with vanilla durability bars. */
	private static int[] drawArmor(DrawContext ctx, ClientPlayerEntity p, boolean preview) {
		int w = 4 * 20 + 2;
		int h = 22;
		List<ItemStack> stacks = new ArrayList<>();
		for (EquipmentSlot slot : ARMOR) stacks.add(p.getEquippedStack(slot));
		boolean any = stacks.stream().anyMatch(s -> !s.isEmpty());
		if (!any && !preview) {
			return new int[]{w, 0};
		}
		// Slice the first four slots plus the right border out of the real hotbar texture.
		ctx.drawTexture(RenderPipelines.GUI_TEXTURED, HOTBAR, 0, 0, 0.0f, 0.0f, 81, 22, 182, 22);
		ctx.drawTexture(RenderPipelines.GUI_TEXTURED, HOTBAR, 81, 0, 181.0f, 0.0f, 1, 22, 182, 22);
		for (int i = 0; i < 4; i++) {
			ItemStack s = stacks.get(i);
			if (!s.isEmpty()) {
				ctx.drawItem(s, 3 + i * 20, 3);
				ctx.drawStackOverlay(Ui.font(), s, 3 + i * 20, 3);
			}
		}
		return new int[]{w, h};
	}

	private static String time(StatusEffectInstance e) {
		if (e.isInfinite()) return "inf";
		int s = e.getDuration() / 20;
		return String.format("%d:%02d", s / 60, s % 60);
	}

	private static String roman(int n) {
		return switch (n) {
			case 1 -> "";
			case 2 -> " II";
			case 3 -> " III";
			case 4 -> " IV";
			case 5 -> " V";
			default -> " " + n;
		};
	}

	private static int[] drawEffects(DrawContext ctx, List<StatusEffectInstance> effects) {
		if (effects.isEmpty()) {
			panel(ctx, 100, 16);
			Ui.text(ctx, "Potion effects", 6, 4, Theme.alpha(Theme.hudText(), 160));
			return new int[]{100, 16};
		}
		int w = 0;
		List<String> names = new ArrayList<>();
		for (StatusEffectInstance e : effects) {
			String n = e.getEffectType().value().getName().getString() + roman(e.getAmplifier() + 1);
			names.add(n);
			w = Math.max(w, Ui.width(n) + 8 + Ui.width(time(e)));
		}
		w += 14;
		int h = effects.size() * 11 + 6;
		panel(ctx, w, h);
		int y = 4;
		for (int i = 0; i < effects.size(); i++) {
			StatusEffectInstance e = effects.get(i);
			int color = e.getEffectType().value().isBeneficial() ? Theme.GOOD : Theme.BAD;
			ctx.drawText(Ui.font(), names.get(i), 7, y, color, shadowed());
			String t = time(e);
			int tc = !e.isInfinite() && e.getDuration() < 200 ? Theme.WARN : Theme.hudText();
			ctx.drawText(Ui.font(), t, w - 7 - Ui.width(t), y, tc, shadowed());
			y += 11;
		}
		return new int[]{w, h};
	}

	private static int[] drawInventory(DrawContext ctx, ClientPlayerEntity p) {
		int w = 9 * 18 + 4;
		int h = 3 * 18 + 4;
		panel(ctx, w, h);
		for (int i = 0; i < 27; i++) {
			ItemStack s = p.getInventory().getStack(9 + i);
			int x = 2 + (i % 9) * 18;
			int y = 2 + (i / 9) * 18;
			if (!s.isEmpty()) {
				ctx.drawItem(s, x + 1, y + 1);
				ctx.drawStackOverlay(Ui.font(), s, x + 1, y + 1);
			}
		}
		return new int[]{w, h};
	}

	private static int[] drawHeld(DrawContext ctx, ItemStack s) {
		String name = s.isEmpty() ? "Empty hand" : s.getName().getString() + (s.getCount() > 1 ? " x" + s.getCount() : "");
		int w = 26 + Ui.width(name) + 8;
		panel(ctx, w, 22);
		if (!s.isEmpty()) {
			ctx.drawItem(s, 4, 3);
		}
		ctx.drawText(Ui.font(), name, 24, 7, Theme.hudText(), shadowed());
		return new int[]{w, 22};
	}

	private static void drawDurabilityAlert(DrawContext ctx, ClientPlayerEntity p, int sw) {
		List<ItemStack> check = new ArrayList<>();
		for (EquipmentSlot slot : ARMOR) check.add(p.getEquippedStack(slot));
		check.add(p.getMainHandStack());
		for (ItemStack s : check) {
			if (s.isEmpty() || !s.isDamageable()) continue;
			int left = s.getMaxDamage() - s.getDamage();
			if (left <= Math.max(10, s.getMaxDamage() / 10)) {
				String msg = s.getName().getString() + " is about to break (" + left + ")";
				Ui.centered(ctx, msg, sw / 2, 40, Theme.WARN);
				return;
			}
		}
	}
}
