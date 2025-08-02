package com.modrinth.theseus.agent.transformers;

import org.objectweb.asm.ClassReader;
import org.objectweb.asm.ClassWriter;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.tree.ClassNode;

public abstract class ClassTransformer {
    public abstract boolean transform(ClassReader reader, ClassWriter writer);

    protected static boolean needsStackMap(ClassNode classNode) {
        return (classNode.version & 0xffff) >= Opcodes.V1_6;
    }
}
