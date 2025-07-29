package com.modrinth.theseus.agent;

import java.util.ListIterator;
import org.objectweb.asm.ClassVisitor;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.tree.AbstractInsnNode;
import org.objectweb.asm.tree.ClassNode;
import org.objectweb.asm.tree.InsnNode;
import org.objectweb.asm.tree.JumpInsnNode;
import org.objectweb.asm.tree.LabelNode;
import org.objectweb.asm.tree.LdcInsnNode;
import org.objectweb.asm.tree.MethodInsnNode;
import org.objectweb.asm.tree.MethodNode;
import org.objectweb.asm.tree.VarInsnNode;

public final class MinecraftTransformer extends ClassNodeTransformer {
    private static final String SET_SERVER_NAME_DESC = "(Ljava/lang/String;I)V";

    public MinecraftTransformer(ClassVisitor parent) {
        super(Opcodes.ASM9, parent);
    }

    @Override
    protected void transformClass(ClassNode classNode) {
        System.out.println("Transforming " + classNode.name);
        addServerJoinSupport(classNode);
    }

    private static void addServerJoinSupport(ClassNode classNode) {
        String setServerName = null;
        for (final MethodNode method : classNode.methods) {
            if (method.desc.equals(SET_SERVER_NAME_DESC)) {
                if (setServerName == null) {
                    setServerName = method.name;
                } else {
                    // Already found a setServer method, but we found another one? Since we can't
                    // know which is real, just return so we don't call something we shouldn't.
                    // Note this can't happen unless some other mod is adding a method with this
                    // same descriptor.
                    return;
                }
            }
        }

        final String fSetServerName = setServerName;
        classNode.methods.stream()
                .filter(m -> m.name.equals("<init>"))
                .findFirst()
                .ifPresent(constructor -> {
                    final ListIterator<AbstractInsnNode> iter =
                            constructor.instructions.iterator(constructor.instructions.size());
                    iter.previous();

                    final LabelNode noQuickPlayLabel = new LabelNode();
                    final LabelNode doneQuickPlayLabel = new LabelNode();
                    iter.add(new LdcInsnNode("modrinth.quickPlayHost"));
                    // String
                    iter.add(new MethodInsnNode(
                            Opcodes.INVOKESTATIC,
                            "java/lang/System",
                            "getProperty",
                            "(Ljava/lang/String;)Ljava/lang/String;"));
                    // String
                    iter.add(new InsnNode(Opcodes.DUP));
                    // String String
                    iter.add(new JumpInsnNode(Opcodes.IFNULL, noQuickPlayLabel));
                    // String
                    iter.add(new VarInsnNode(Opcodes.ALOAD, 0));
                    // String Minecraft
                    iter.add(new InsnNode(Opcodes.SWAP));
                    // Minecraft String
                    iter.add(new LdcInsnNode("modrinth.quickPlayPort"));
                    // Minecraft String String
                    iter.add(new MethodInsnNode(
                            Opcodes.INVOKESTATIC,
                            "java/lang/System",
                            "getProperty",
                            "(Ljava/lang/String;)Ljava/lang/String;"));
                    // Minecraft String String
                    iter.add(new MethodInsnNode(
                            Opcodes.INVOKESTATIC, "java/lang/Integer", "parseInt", "(Ljava/lang/String;)I"));
                    // Minecraft String int
                    iter.add(new MethodInsnNode(
                            Opcodes.INVOKEVIRTUAL,
                            "net/minecraft/client/Minecraft",
                            fSetServerName,
                            SET_SERVER_NAME_DESC));
                    //
                    iter.add(new JumpInsnNode(Opcodes.GOTO, doneQuickPlayLabel));
                    iter.add(noQuickPlayLabel);
                    // String
                    iter.add(new InsnNode(Opcodes.POP));
                    //
                    iter.add(doneQuickPlayLabel);
                    //
                });
    }
}
