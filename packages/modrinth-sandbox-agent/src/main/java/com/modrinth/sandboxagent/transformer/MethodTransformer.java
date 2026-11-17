package com.modrinth.sandboxagent.transformer;

import org.objectweb.asm.tree.AbstractInsnNode;

import java.util.ListIterator;

public interface MethodTransformer {

    boolean transformMethod(ListIterator<AbstractInsnNode> it, boolean isStatic);

}
