package com.modrinth.sandboxagent.transformation;

import com.modrinth.sandboxagent.MinecraftVersionInfo;
import com.modrinth.sandboxagent.SandboxAgent;
import com.modrinth.sandboxagent.transformer.MethodTransformer;
import com.modrinth.sandboxagent.transformer.TransformerMap;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.Type;
import org.objectweb.asm.tree.*;

import java.util.ArrayList;
import java.util.List;
import java.util.ListIterator;

public class Transformation {

    public static void registerObfuscated(TransformerMap transformerMap, MethodTransformer transformer, MinecraftVersionInfo minecraftVersionInfo, String mappings) {
        if (!minecraftVersionInfo.isObfuscated()) {
            return;
        }

        String csv = SandboxAgent.readResourceToString(mappings);
        if (csv == null) {
            System.err.println("Missing " + mappings);
            return;
        }
        int start = 0;
        while (true) {
            int lineEnd = csv.indexOf('\n', start);
            int column1End = csv.indexOf(',', start);
            if (column1End < 0) {
                break;
            } else if (lineEnd >= 0 && column1End > lineEnd) {
                continue;
            }

            String sub = csv.substring(start, column1End);
            if (sub.equals(minecraftVersionInfo.id) || sub.equals(minecraftVersionInfo.name)) {
                registerObfuscatedEntry(transformerMap, transformer, csv, column1End, lineEnd);
            }

            if (lineEnd < 0) {
                break;
            }
            start = lineEnd+1;
        }
    }

    private static void registerObfuscatedEntry(TransformerMap transformerMap, MethodTransformer transformer, String csv, int column1End, int lineEnd) {
        int column2End = csv.indexOf(',', column1End+1);
        int column3End = csv.indexOf(',', column2End+1);
        int column4End = csv.indexOf(',', column3End+1);
        if (lineEnd >= 0) {
            if (column2End > lineEnd) {
                return;
            }
            if (column3End > lineEnd) {
                return;
            }
            if (column4End > lineEnd) {
                column4End = lineEnd;
            }
        }
        if (column2End < 0 || column3End < 0 || column4End < 0) {
            return;
        }

        String className = csv.substring(column1End+1, column2End);
        String method = csv.substring(column2End+1, column3End);
        String desc = csv.substring(column3End+1, column4End);

        transformerMap.registerMethod(className, method, desc, transformer);
    }

    public static void overwrite(ListIterator<AbstractInsnNode> it, boolean isStatic, boolean conditional, Class<?> owner, String name, String descriptor) {
        Type methodType = Type.getMethodType(descriptor);
        int index = isStatic ? 0 : 1;

        // Load arguments onto stack
        for (Type argumentType : methodType.getArgumentTypes()) {
            it.add(new VarInsnNode(argumentType.getOpcode(Opcodes.ILOAD), index));
            index += argumentType.getSize();
        }

        // Invoke overwrite method
        it.add(new MethodInsnNode(Opcodes.INVOKESTATIC, owner.getName().replace('.', '/'), name, descriptor));

        // Add ret logic
        if (methodType.getReturnType() == Type.BOOLEAN_TYPE && conditional) {
            final LabelNode after = new LabelNode();
            it.add(new JumpInsnNode(Opcodes.IFEQ, after)); // If false, jump to after to skip early return
            it.add(new InsnNode(Opcodes.RETURN));
            it.add(after);
        } else {
            it.add(new InsnNode(methodType.getReturnType().getOpcode(Opcodes.IRETURN)));
        }
    }

}
