package com.modrinth.sandboxagent.transformation;

import com.modrinth.sandboxagent.MinecraftVersionInfo;
import com.modrinth.sandboxagent.SandboxApi;
import com.modrinth.sandboxagent.transformer.TransformerMap;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.tree.*;

import java.net.URI;
import java.util.ListIterator;
import java.util.concurrent.CompletableFuture;

public class OpenUriTransformation {

    public static void register(TransformerMap transformerMap, MinecraftVersionInfo minecraftVersionInfo) {
        // Mojang (26.3+)
        transformerMap.registerMethod("com/mojang/blaze3d/Blaze3D", "openUri", "(Ljava/net/URI;)V", OpenUriTransformation::transform);
        // Mojang (1.21.11+)
        transformerMap.registerMethod("net/minecraft/util/Util$OS", "openUri", "(Ljava/net/URI;)V", OpenUriTransformation::transform);
        // Mojang (before 1.21.11)
        transformerMap.registerMethod("net/minecraft/Util$OS", "openUri", "(Ljava/net/URI;)V", OpenUriTransformation::transform);
        // Intermediary
        transformerMap.registerMethod("net/minecraft/class_156$class_158", "method_673", "(Ljava/net/URI;)V", OpenUriTransformation::transform);
        // SRG
        transformerMap.registerMethod("net/minecraft/src/C_5322_$C_5330_", "m_137648_", "(Ljava/net/URI;)V", OpenUriTransformation::transform);
        // Old MCP (Intermediary, 1.14+)
        transformerMap.registerMethod("net/minecraft/util/Util$OS", "func_195642_a", "(Ljava/net/URI;)V", OpenUriTransformation::transform);
        // Old MCP (Unobfuscated, 1.14+)
        transformerMap.registerMethod("net/minecraft/util/Util$OS", "openURI", "(Ljava/net/URI;)V", OpenUriTransformation::transform);
        // Old MCP (Intermediary, 1.13)
        transformerMap.registerMethod("net/minecraft/util/Util$EnumOS", "func_195642_a", "(Ljava/net/URI;)V", OpenUriTransformation::transform);
        // Old MCP (Unobfuscated, 1.13)
        transformerMap.registerMethod("net/minecraft/util/Util$EnumOS", "openURI", "(Ljava/net/URI;)V", OpenUriTransformation::transform);
        // Old MCP (Intermediary, before 1.13)
        transformerMap.registerMethod("net/minecraft/client/gui/GuiScreen", "func_175282_a", "(Ljava/net/URI;)V", OpenUriTransformation::transform);
        // Old MCP (Unobfuscated, before 1.13)
        transformerMap.registerMethod("net/minecraft/client/gui/GuiScreen", "openWebLink", "(Ljava/net/URI;)V", OpenUriTransformation::transform);

        // Util$OS#openUri
        Transformation.registerObfuscated(transformerMap, OpenUriTransformation::transform,
                minecraftVersionInfo, "/com/modrinth/sandboxagent/method_673.csv");
        // Screen#openUri (before 1.13)
        Transformation.registerObfuscated(transformerMap, OpenUriTransformation::transform,
                minecraftVersionInfo, "/com/modrinth/sandboxagent/method_2221.csv");
    }

    private static boolean transform(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        final LabelNode after = new LabelNode();
        it.add(new VarInsnNode(Opcodes.ALOAD, isStatic ? 0 : 1));
        it.add(new MethodInsnNode(Opcodes.INVOKESTATIC, "com/modrinth/sandboxagent/transformation/OpenUriTransformation",
                "openUri", "(Ljava/net/URI;)Z"));
        it.add(new JumpInsnNode(Opcodes.IFEQ, after)); // If false, jump to after to skip early return
        it.add(new InsnNode(Opcodes.RETURN));
        it.add(after);
        return true;
    }

    public static boolean openUri(URI uri) {
        String scheme = uri.getScheme();
        if (scheme.equals("http") || scheme.equals("https") || scheme.equals("file")) {
            CompletableFuture.runAsync(() -> SandboxApi.getOrInit().openUri(uri));
            return true;
        }
        return false;
    }

}
