package com.modrinth.sandboxagent.transformation;

import com.modrinth.sandboxagent.transformer.TransformerMap;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.tree.*;

import java.io.File;
import java.net.URI;
import java.util.ListIterator;
import java.util.function.Predicate;

public class DesktopTransformation {

    public static void register(TransformerMap transformerMap) {
        String desktop = "java/awt/Desktop";

        transformerMap.registerMethod(desktop, "browse", "(Ljava/net/URI;)V", (it, isStatic) -> transform(it, BrowseHandler.class));
        transformerMap.registerMethod(desktop, "open", "(Ljava/io/File;)V", (it, isStatic) -> transform(it, OpenHandler.class));
        // Java 9+
        transformerMap.registerMethod(desktop, "browseFileDirectory", "(Ljava/io/File;)V", (it, isStatic) -> transform(it, BrowseFileDirectoryHandler.class));
    }

    // Inserts an early return if the handler returns true
    private static boolean transform(ListIterator<AbstractInsnNode> it, Class<? extends Predicate<?>> handler) {
        final LabelNode after = new LabelNode();
        Transformation.createHandlerForSystemClassloader(it, handler, Predicate.class);
        it.add(new VarInsnNode(Opcodes.ALOAD, 1));
        it.add(new MethodInsnNode(Opcodes.INVOKEINTERFACE, "java/util/function/Predicate", "test", "(Ljava/lang/Object;)Z"));
        it.add(new JumpInsnNode(Opcodes.IFEQ, after)); // If false, jump to after to skip early return
        it.add(new InsnNode(Opcodes.RETURN));
        it.add(after);
        return true;
    }

    // The handlers leave null and missing files to the original method, which throws the appropriate exception

    public static class BrowseHandler implements Predicate<URI> {
        @Override
        public boolean test(URI uri) {
            return uri != null && OpenUriTransformation.openUri(uri);
        }
    }

    public static class OpenHandler implements Predicate<File> {
        @Override
        public boolean test(File file) {
            return file != null && file.exists() && OpenUriTransformation.openUri(file.toURI());
        }
    }

    public static class BrowseFileDirectoryHandler implements Predicate<File> {
        @Override
        public boolean test(File file) {
            if (file == null || !file.exists()) {
                return false;
            }
            File directory = file.getAbsoluteFile().getParentFile();
            return directory != null && OpenUriTransformation.openUri(directory.toURI());
        }
    }

}
