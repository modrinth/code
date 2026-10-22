package com.modrinth.sandboxagent.transformation;

import com.modrinth.sandboxagent.MinecraftVersionInfo;
import com.modrinth.sandboxagent.SandboxAgent;
import com.modrinth.sandboxagent.SandboxApi;
import com.modrinth.sandboxagent.transformer.TransformerMap;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.tree.*;

import java.io.File;
import java.net.URI;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.ListIterator;
import java.util.concurrent.CompletableFuture;

public class DropFileTransformation {

    public static void register(TransformerMap transformerMap, MinecraftVersionInfo minecraftVersionInfo) {
        String[] descs = new String[]{"(JLjava/util/List;)V", "(JLjava/util/List;I)V"};
        for (String desc : descs) {
            // Mojang
            transformerMap.registerMethod("net/minecraft/client/MouseHandler", "onDrop", desc, DropFileTransformation::transform);

            // Intermediary
            transformerMap.registerMethod("net/minecraft/class_312", "method_29616", desc, DropFileTransformation::transform);
            // SRG
            transformerMap.registerMethod("net/minecraft/src/C_3397_", "m_91539_", desc, DropFileTransformation::transform);
            // Old MCP (Intermediary)
            transformerMap.registerMethod("net/minecraft/client/MouseHelper", "func_238228_a_", desc, DropFileTransformation::transform);
            // Old MCP (Unobfuscated)
            transformerMap.registerMethod("net/minecraft/client/MouseHelper", "addPacksToScreen", desc, DropFileTransformation::transform);
        }

        Transformation.registerObfuscated(transformerMap, DropFileTransformation::transform,
                minecraftVersionInfo, "/com/modrinth/sandboxagent/method_29616.csv");
    }

    private static boolean transform(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        it.add(new VarInsnNode(Opcodes.ALOAD, isStatic ? 2 : 3));
        it.add(new MethodInsnNode(Opcodes.INVOKESTATIC, "com/modrinth/sandboxagent/transformation/DropFileTransformation",
                "accessPaths", "(Ljava/util/List;)Ljava/util/List;"));
        it.add(new VarInsnNode(Opcodes.ASTORE, isStatic ? 2 : 3));
        return true;
    }

    public static List<Object> accessPaths(List<Object> list) {
        if (list.isEmpty()) {
            return list;
        }

        boolean changed = false;
        List<CompletableFuture<Object>> futureList = new ArrayList<>();

		SandboxApi sandboxApi = SandboxApi.getOrInit();

        for (Object o : list) {
            Path path;
            Class<?> srcType;

            if (o instanceof Path) {
                path = (Path) o;
                srcType = Path.class;
            } else if (o instanceof File) {
                path = ((File) o).toPath();
                srcType = File.class;
            } else if (o instanceof String) {
                path = Paths.get((String) o);
                srcType = String.class;
            } else {
                futureList.add(CompletableFuture.completedFuture(o));
                continue;
            }

            // todo: make true
            if (true) {//Files.exists(path) && !Files.isReadable(path)) {
                futureList.add(CompletableFuture.supplyAsync(() -> sandboxApi.tryRequestAccess(path))
                        .thenApply(newPath -> {
                            if (srcType == Path.class) {
                                return newPath;
                            } else if (srcType == File.class) {
                                return newPath.toFile();
                            } else if (srcType == String.class) {
                                return newPath.toString();
                            } else {
                                return null;
                            }
                        }));
                changed = true;
            } else {
                futureList.add(CompletableFuture.completedFuture(o));
            }
        }

        if (!changed) {
            return list;
        }

        List<Object> newList = new ArrayList<>();
        for (CompletableFuture<Object> future : futureList) {
            Object o = future.join();
            if (o != null) {
                newList.add(o);
            }
        }
        return newList;
    }

}
