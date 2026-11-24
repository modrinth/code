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

    // Register a method to be unconditionally overwritten by a static handler with the same descriptor
    public static void registerOverwrite(TransformerMap transformerMap, String className, String method, String descriptor, Class<?> owner, String handlerName) {
        transformerMap.registerMethod(className, method, descriptor, (it, isStatic) -> {
            overwrite(it, isStatic, owner, handlerName, descriptor);
            return true;
        });
    }

    // Register a method to be conditionally overwritten, see overwriteConditional for the handler's return type
    public static void registerOverwriteConditional(TransformerMap transformerMap, String className, String method, String descriptor, Class<?> owner, String handlerName) {
        transformerMap.registerMethod(className, method, descriptor, (it, isStatic) -> {
            overwriteConditional(it, isStatic, owner, handlerName, descriptor);
            return true;
        });
    }

    // Unconditionally overwrite method by inserting an early return
    public static void overwrite(ListIterator<AbstractInsnNode> it, boolean isStatic, Class<?> owner, String name, String descriptor) {
        Type methodType = Type.getMethodType(descriptor);
        String ownerName = owner.getName().replace('.', '/');

        loadArguments(it, isStatic, methodType);
        it.add(new MethodInsnNode(Opcodes.INVOKESTATIC, ownerName, name, descriptor));
        it.add(new InsnNode(methodType.getReturnType().getOpcode(Opcodes.IRETURN)));
    }

    // Conditionally overwrite method by inserting an early return depending on the handler's result
    // void methods: the handler returns a boolean, true to return early
    // int methods: the handler returns an OptionalInt, present to return early with that value
    public static void overwriteConditional(ListIterator<AbstractInsnNode> it, boolean isStatic, Class<?> owner, String name, String descriptor) {
        Type methodType = Type.getMethodType(descriptor);
        Type returnType = methodType.getReturnType();
        String ownerName = owner.getName().replace('.', '/');
        final LabelNode after = new LabelNode();

        if (returnType == Type.VOID_TYPE) {
            String handlerDescriptor = Type.getMethodDescriptor(Type.BOOLEAN_TYPE, methodType.getArgumentTypes());

            loadArguments(it, isStatic, methodType);
            it.add(new MethodInsnNode(Opcodes.INVOKESTATIC, ownerName, name, handlerDescriptor));
            it.add(new JumpInsnNode(Opcodes.IFEQ, after));
            it.add(new InsnNode(Opcodes.RETURN));
        } else if (returnType == Type.INT_TYPE) {
            String optionalInt = "java/util/OptionalInt";
            String handlerDescriptor = Type.getMethodDescriptor(Type.getObjectType(optionalInt), methodType.getArgumentTypes());
            final LabelNode empty = new LabelNode();

            loadArguments(it, isStatic, methodType);
            it.add(new MethodInsnNode(Opcodes.INVOKESTATIC, ownerName, name, handlerDescriptor));
            it.add(new InsnNode(Opcodes.DUP));
            it.add(new MethodInsnNode(Opcodes.INVOKEVIRTUAL, optionalInt, "isPresent", "()Z"));
            it.add(new JumpInsnNode(Opcodes.IFEQ, empty));
            it.add(new MethodInsnNode(Opcodes.INVOKEVIRTUAL, optionalInt, "getAsInt", "()I"));
            it.add(new InsnNode(Opcodes.IRETURN));
            it.add(empty);
            it.add(new InsnNode(Opcodes.POP));
        } else {
            throw new RuntimeException("overwriteConditional can only be used with void or int methods");
        }

        it.add(after);
    }

    // Creates an instance of the handler, which must be public and have a public constructor without arguments
    // Classes loaded by the bootstrap class loader can't reference our classes directly, so this uses the
    // system class loader and casts to an interface that they can reference
    public static void createHandlerForSystemClassloader(ListIterator<AbstractInsnNode> it, Class<?> handler, Class<?> handlerInterface) {
        it.add(new MethodInsnNode(Opcodes.INVOKESTATIC, "java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;"));
        it.add(new LdcInsnNode(handler.getName()));
        it.add(new MethodInsnNode(Opcodes.INVOKEVIRTUAL, "java/lang/ClassLoader", "loadClass", "(Ljava/lang/String;)Ljava/lang/Class;"));
        it.add(new MethodInsnNode(Opcodes.INVOKEVIRTUAL, "java/lang/Class", "newInstance", "()Ljava/lang/Object;"));
        it.add(new TypeInsnNode(Opcodes.CHECKCAST, Type.getInternalName(handlerInterface)));
    }

    private static void loadArguments(ListIterator<AbstractInsnNode> it, boolean isStatic, Type methodType) {
        int index = isStatic ? 0 : 1;
        for (Type argumentType : methodType.getArgumentTypes()) {
            it.add(new VarInsnNode(argumentType.getOpcode(Opcodes.ILOAD), index));
            index += argumentType.getSize();
        }
    }

}
