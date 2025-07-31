package com.modrinth.theseus.agent.transformers;

import com.modrinth.theseus.agent.InsnPattern;
import com.modrinth.theseus.agent.QuickPlayServerVersion;
import java.util.ListIterator;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.tree.AbstractInsnNode;
import org.objectweb.asm.tree.ClassNode;
import org.objectweb.asm.tree.FrameNode;
import org.objectweb.asm.tree.InsnNode;
import org.objectweb.asm.tree.JumpInsnNode;
import org.objectweb.asm.tree.LabelNode;
import org.objectweb.asm.tree.LdcInsnNode;
import org.objectweb.asm.tree.MethodInsnNode;
import org.objectweb.asm.tree.MethodNode;
import org.objectweb.asm.tree.VarInsnNode;

public final class MinecraftTransformer extends ClassNodeTransformer {
    private static final String SET_SERVER_NAME_DESC = "(Ljava/lang/String;I)V";
    private static final InsnPattern[] INITIALIZE_THIS_PATTERN = {InsnPattern.opcode(Opcodes.INVOKESPECIAL)};

    @Override
    protected boolean transform(ClassNode classNode) {
        if (QuickPlayServerVersion.CURRENT == QuickPlayServerVersion.INJECTED) {
            return addServerJoinSupport(classNode);
        }
        return false;
    }

    private static boolean addServerJoinSupport(ClassNode classNode) {
        String setServerName = null;
        MethodNode constructor = null;
        for (final MethodNode method : classNode.methods) {
            if (constructor == null && method.name.equals("<init>")) {
                constructor = method;
            } else if (method.desc.equals(SET_SERVER_NAME_DESC) && method.name.indexOf('$') == -1) {
                // Check for $ is because Mixin-injected methods should have $ in it
                if (setServerName == null) {
                    setServerName = method.name;
                } else {
                    // Already found a setServer method, but we found another one? Since we can't
                    // know which is real, just return so we don't call something we shouldn't.
                    // Note this can't happen unless some other mod is adding a method with this
                    // same descriptor.
                    return false;
                }
            }
        }
        if (constructor == null) {
            return false;
        }

        final ListIterator<AbstractInsnNode> it = constructor.instructions.iterator();
        if (!InsnPattern.findAndSkip(it, INITIALIZE_THIS_PATTERN)) {
            return true;
        }

        final LabelNode noQuickPlayLabel = new LabelNode();
        final LabelNode doneQuickPlayLabel = new LabelNode();
        it.add(new LdcInsnNode("modrinth.internal.quickPlay.host"));
        // String
        it.add(new MethodInsnNode(
                Opcodes.INVOKESTATIC, "java/lang/System", "getProperty", "(Ljava/lang/String;)Ljava/lang/String;"));
        // String
        it.add(new InsnNode(Opcodes.DUP));
        // String String
        it.add(new JumpInsnNode(Opcodes.IFNULL, noQuickPlayLabel));
        // String
        it.add(new VarInsnNode(Opcodes.ALOAD, 0));
        // String Minecraft
        it.add(new InsnNode(Opcodes.SWAP));
        // Minecraft String
        it.add(new LdcInsnNode("modrinth.internal.quickPlay.port"));
        // Minecraft String String
        it.add(new MethodInsnNode(
                Opcodes.INVOKESTATIC, "java/lang/System", "getProperty", "(Ljava/lang/String;)Ljava/lang/String;"));
        // Minecraft String String
        it.add(new MethodInsnNode(Opcodes.INVOKESTATIC, "java/lang/Integer", "parseInt", "(Ljava/lang/String;)I"));
        // Minecraft String int
        it.add(new MethodInsnNode(
                Opcodes.INVOKEVIRTUAL, "net/minecraft/client/Minecraft", setServerName, SET_SERVER_NAME_DESC));
        //
        it.add(new JumpInsnNode(Opcodes.GOTO, doneQuickPlayLabel));
        it.add(noQuickPlayLabel);
        if (needsStackMap(classNode)) {
            it.add(new FrameNode(Opcodes.F_SAME, 0, null, 0, null));
        }
        // String
        it.add(new InsnNode(Opcodes.POP));
        //
        it.add(doneQuickPlayLabel);
        if (needsStackMap(classNode)) {
            it.add(new FrameNode(Opcodes.F_SAME, 0, null, 0, null));
        }
        //

        return true;
    }
}
