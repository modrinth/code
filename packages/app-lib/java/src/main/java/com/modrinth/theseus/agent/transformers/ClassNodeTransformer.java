package com.modrinth.theseus.agent.transformers;

import org.objectweb.asm.ClassVisitor;
import org.objectweb.asm.tree.ClassNode;

public abstract class ClassNodeTransformer extends ClassVisitor {
    private final ClassNode classNode;
    private final ClassVisitor parent;

    private ClassNodeTransformer(int api, ClassNode classNode, ClassVisitor parent) {
        super(api, classNode);
        this.classNode = classNode;
        this.parent = parent;
    }

    protected ClassNodeTransformer(int api, ClassVisitor parent) {
        this(api, new ClassNode(api), parent);
    }

    protected abstract void transformClass(ClassNode classNode);

    @Override
    public void visitEnd() {
        super.visitEnd();
        transformClass(classNode);
        classNode.accept(parent);
    }
}
