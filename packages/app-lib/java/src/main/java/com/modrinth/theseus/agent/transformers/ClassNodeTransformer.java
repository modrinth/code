package com.modrinth.theseus.agent.transformers;

import org.objectweb.asm.ClassReader;
import org.objectweb.asm.ClassWriter;
import org.objectweb.asm.tree.ClassNode;

public abstract class ClassNodeTransformer extends ClassTransformer {
    protected abstract boolean transform(ClassNode classNode);

    @Override
    public final boolean transform(ClassReader reader, ClassWriter writer) {
        final ClassNode classNode = new ClassNode();
        reader.accept(classNode, 0);
        if (!transform(classNode)) {
            return false;
        }
        classNode.accept(writer);
        return true;
    }
}
