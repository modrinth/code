package com.modrinth.theseus.agent;

import com.modrinth.theseus.agent.transformers.ClassTransformer;
import com.modrinth.theseus.agent.transformers.MinecraftTransformer;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.lang.instrument.Instrumentation;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.SimpleFileVisitor;
import java.nio.file.attribute.BasicFileAttributes;
import java.util.HashMap;
import java.util.Map;
import org.objectweb.asm.ClassReader;
import org.objectweb.asm.ClassWriter;

@SuppressWarnings({"NullableProblems", "CallToPrintStackTrace"})
public final class TheseusAgent {
    private static final boolean DEBUG_AGENT = Boolean.getBoolean("modrinth.debugAgent");

    public static void premain(String args, Instrumentation instrumentation) {
        final Path debugPath = Paths.get("ModrinthDebugTransformed");
        if (DEBUG_AGENT) {
            System.out.println(
                    "===== Theseus agent debugging enabled. Dumping transformed classes to " + debugPath + " =====");
            if (Files.exists(debugPath)) {
                try {
                    Files.walkFileTree(debugPath, new SimpleFileVisitor<Path>() {
                        @Override
                        public FileVisitResult visitFile(Path file, BasicFileAttributes attrs) throws IOException {
                            Files.delete(file);
                            return FileVisitResult.CONTINUE;
                        }

                        @Override
                        public FileVisitResult postVisitDirectory(Path dir, IOException exc) throws IOException {
                            Files.delete(dir);
                            return FileVisitResult.CONTINUE;
                        }
                    });
                } catch (IOException e) {
                    new UncheckedIOException("Failed to delete " + debugPath, e).printStackTrace();
                }
            }
            System.out.println("===== Quick play server version: " + QuickPlayServerVersion.CURRENT + " =====");
        }

        final Map<String, ClassTransformer> transformers = new HashMap<>();
        transformers.put("net/minecraft/client/Minecraft", new MinecraftTransformer());

        instrumentation.addTransformer((loader, className, classBeingRedefined, protectionDomain, classData) -> {
            final ClassTransformer transformer = transformers.get(className);
            if (transformer == null) {
                return null;
            }
            final ClassReader reader = new ClassReader(classData);
            final ClassWriter writer = new ClassWriter(reader, ClassWriter.COMPUTE_MAXS);
            try {
                if (!transformer.transform(reader, writer)) {
                    if (DEBUG_AGENT) {
                        System.out.println("Not writing " + className + " as its transformer returned false");
                    }
                    return null;
                }
            } catch (Throwable t) {
                new IllegalStateException("Failed to transform " + className, t).printStackTrace();
                return null;
            }
            final byte[] result = writer.toByteArray();
            if (DEBUG_AGENT) {
                try {
                    final Path path = debugPath.resolve(className + ".class");
                    Files.createDirectories(path.getParent());
                    Files.write(path, result);
                    System.out.println("Dumped class to " + path.toAbsolutePath());
                } catch (IOException e) {
                    new UncheckedIOException("Failed to dump class " + className, e).printStackTrace();
                }
            }
            return result;
        });
    }
}
