package com.modrinth.sandboxagent.transformation;

import com.modrinth.sandboxagent.MinecraftVersionInfo;
import com.modrinth.sandboxagent.SandboxAgent;
import com.modrinth.sandboxagent.SandboxApi;
import com.modrinth.sandboxagent.transformer.TransformerMap;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.tree.*;

import java.net.URI;
import java.util.ListIterator;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;

public class RealmsTransformation {

    public static void register(TransformerMap transformerMap, MinecraftVersionInfo minecraftVersionInfo) {
        // todo: not working below 1.14

        register(transformerMap, "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;");
        register(transformerMap, "(Ljava/lang/String;Ljava/lang/String;Z)Ljava/lang/String;");

        Transformation.registerObfuscated(transformerMap, RealmsTransformation::transform,
                minecraftVersionInfo, "/com/modrinth/sandboxagent/method_21007.csv");
        Transformation.registerObfuscated(transformerMap, RealmsTransformation::transform,
                minecraftVersionInfo, "/com/modrinth/sandboxagent/method_68464.csv");
    }

    private static void register(TransformerMap transformerMap, String desc) {
        // Mojang
        transformerMap.registerMethod("com/mojang/realmsclient/client/RealmsClient", "url", desc, RealmsTransformation::transform);
        // Intermediary
        transformerMap.registerMethod("net/minecraft/class_4341", "method_21007", desc, RealmsTransformation::transform);
        // Intermediary (new)
        transformerMap.registerMethod("net/minecraft/class_4341", "method_68464", desc, RealmsTransformation::transform);
        // SRG
        transformerMap.registerMethod("net/minecraft/src/C_3223_", "m_87203_", desc, RealmsTransformation::transform);
        // Old MCP
        transformerMap.registerMethod("com/mojang/realmsclient/client/RealmsClient", "func_224907_b", desc, RealmsTransformation::transform);
    }

    private static boolean transform(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        final LabelNode after = new LabelNode();
        it.add(new VarInsnNode(Opcodes.ALOAD, isStatic ? 0 : 1));
        it.add(new VarInsnNode(Opcodes.ALOAD, isStatic ? 1 : 2));
        it.add(new MethodInsnNode(Opcodes.INVOKESTATIC, "com/modrinth/sandboxagent/transformation/RealmsTransformation",
                "createUrl", "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;"));
        it.add(new InsnNode(Opcodes.DUP));
        it.add(new JumpInsnNode(Opcodes.IFNULL, after)); // If null, jump to after to skip early return
        it.add(new InsnNode(Opcodes.ARETURN));
        it.add(after);
        return true;
    }

    public static String createUrl(String path, String query) {
        String url = SandboxApi.getOrInit().getBaseUrl() + "/sandboxrealms/" + path;
        if (query != null) {
            url += "?" + query;
        }
        return url;
    }

}
