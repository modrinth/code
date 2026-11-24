package com.modrinth.sandboxagent.transformation;

import com.modrinth.sandboxagent.implementation.CompletedProcess;
import com.modrinth.sandboxagent.implementation.OpenCommand;
import com.modrinth.sandboxagent.transformer.TransformerMap;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.tree.*;

import java.net.URI;
import java.util.ArrayList;
import java.util.List;
import java.util.ListIterator;
import java.util.function.Function;

public class ProcessBuilderTransformation {

    public static void register(TransformerMap transformerMap) {
        // Runtime#exec also uses this
        transformerMap.registerMethod("java/lang/ProcessBuilder", "start", "()Ljava/lang/Process;", ProcessBuilderTransformation::transform);
    }

    private static boolean transform(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        final LabelNode after = new LabelNode();
        Transformation.createHandlerForSystemClassloader(it, StartHandler.class, Function.class);
        it.add(new VarInsnNode(Opcodes.ALOAD, 0));
        it.add(new MethodInsnNode(Opcodes.INVOKEINTERFACE, "java/util/function/Function", "apply", "(Ljava/lang/Object;)Ljava/lang/Object;"));
        it.add(new TypeInsnNode(Opcodes.CHECKCAST, "java/lang/Process"));
        it.add(new InsnNode(Opcodes.DUP));
        it.add(new JumpInsnNode(Opcodes.IFNULL, after)); // If null, jump to after to start the process normally
        it.add(new InsnNode(Opcodes.ARETURN));
        it.add(after);
        it.add(new InsnNode(Opcodes.POP));
        return true;
    }

    public static class StartHandler implements Function<ProcessBuilder, Process> {
        // Called by ProcessBuilder#start, must not throw
        @Override
        public Process apply(ProcessBuilder processBuilder) {
            try {
                List<String> command = new ArrayList<>(processBuilder.command());
                if (command.contains(null)) {
                    return null;
                }

                URI uri = OpenCommand.parse(command, processBuilder.directory());
                if (uri == null || !OpenUriTransformation.openUri(uri)) {
                    return null;
                }

                System.out.println("Using launcher SandboxApi to handle open command: " + command.get(0));
                return new CompletedProcess();
            } catch (Throwable t) {
                t.printStackTrace();
                return null;
            }
        }
    }

}
