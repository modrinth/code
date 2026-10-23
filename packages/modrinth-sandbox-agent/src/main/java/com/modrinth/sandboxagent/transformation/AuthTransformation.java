package com.modrinth.sandboxagent.transformation;

import com.modrinth.sandboxagent.MinecraftVersionInfo;
import com.modrinth.sandboxagent.transformer.TransformerMap;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.tree.*;

import java.util.ListIterator;

public class AuthTransformation {

    public static void register(TransformerMap transformerMap, MinecraftVersionInfo minecraftVersionInfo) {
        // Transform authlib 1.x
        transformerMap.registerMethod("com/mojang/authlib/HttpAuthenticationService", "constantURL", "(Ljava/lang/String;)Ljava/net/URL;", AuthTransformation::transform);
    }

    private static boolean transform(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        it.add(new VarInsnNode(Opcodes.ALOAD, isStatic ? 0 : 1));
        it.add(new MethodInsnNode(Opcodes.INVOKESTATIC, "com/modrinth/sandboxagent/transformation/AuthTransformation",
                "modifyConstantUrl", "(Ljava/lang/String;)Ljava/lang/String;"));
        it.add(new VarInsnNode(Opcodes.ASTORE, isStatic ? 0 : 1));
        return true;
    }

    public static String modifyConstantUrl(String url) {
        url = replaceFromProperty(url, "https://authserver.mojang.com/", "minecraft.api.auth.host");
        url = replaceFromProperty(url, "https://api.mojang.com/", "minecraft.api.account.host");
        url = replaceFromProperty(url, "https://sessionserver.mojang.com/", "minecraft.api.session.host");
        url = replaceFromProperty(url, "https://api.minecraftservices.com/", "minecraft.api.services.host");

        return url;
    }

    private static String replaceFromProperty(String url, String baseUrl, String property) {
        if (url.startsWith(baseUrl)) {
            String customAuthHost = System.getProperty(property);
            if (customAuthHost != null) {
                if (!customAuthHost.endsWith("/")) {
                    customAuthHost += "/";
                }
                return customAuthHost + url.substring(baseUrl.length());
            }
        }
        return url;
    }

}
