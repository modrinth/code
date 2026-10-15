package com.modrinth.sandboxagent.transformer;

import org.objectweb.asm.ClassReader;
import org.objectweb.asm.ClassWriter;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.tree.ClassNode;
import org.objectweb.asm.tree.MethodNode;

import java.lang.instrument.ClassFileTransformer;
import java.security.ProtectionDomain;
import java.util.HashMap;
import java.util.List;

public class TransformerMap implements ClassFileTransformer {

    private final HashMap<String, HashMap<String, HashMap<String, MethodTransformer>>> transformers = new HashMap<>();

    public void registerMethod(String className, String method, String desc, MethodTransformer transformer) {
        HashMap<String, HashMap<String, MethodTransformer>> byMethod = transformers.computeIfAbsent(className, k -> new HashMap<>());
        HashMap<String, MethodTransformer> byDesc = byMethod.computeIfAbsent(method, k -> new HashMap<>());
        byDesc.put(desc, transformer);
    }

    @Override
    public byte[] transform(ClassLoader classLoader, String className, Class<?> clazz, ProtectionDomain protectionDomain, byte[] classBytes) {
        HashMap<String, HashMap<String, MethodTransformer>> byMethod = transformers.get(className);
        if (byMethod == null) {
            return null;
        }

        final ClassNode classNode = new ClassNode();
        ClassReader reader = new ClassReader(classBytes);
        reader.accept(classNode, 0);

        boolean modified = false;
        for (MethodNode method : classNode.methods) {
            HashMap<String, MethodTransformer> byDesc = byMethod.get(method.name);
            if (byDesc == null) {
                continue;
            }

            MethodTransformer transformer = byDesc.get(method.desc);
            if (transformer == null) {
                continue;
            }

            boolean isStatic = (method.access & Opcodes.ACC_STATIC) != 0;

            modified |= transformer.transformMethod(method.instructions.iterator(), isStatic);
        }

        if (!modified) {
            return null;
        }

        ClassWriter classWriter = new ClassWriter(reader, ClassWriter.COMPUTE_FRAMES | ClassWriter.COMPUTE_MAXS);
        classNode.accept(classWriter);
        return classWriter.toByteArray();



    }
}
