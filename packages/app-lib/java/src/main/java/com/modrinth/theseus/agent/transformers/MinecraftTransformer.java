package com.modrinth.theseus.agent.transformers;

import com.modrinth.theseus.agent.InsnPattern;
import com.modrinth.theseus.agent.QuickPlayServerVersion;
import java.util.ListIterator;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.tree.AbstractInsnNode;
import org.objectweb.asm.tree.ClassNode;
import org.objectweb.asm.tree.FrameNode;
import org.objectweb.asm.tree.InsnList;
import org.objectweb.asm.tree.InsnNode;
import org.objectweb.asm.tree.JumpInsnNode;
import org.objectweb.asm.tree.LabelNode;
import org.objectweb.asm.tree.LdcInsnNode;
import org.objectweb.asm.tree.MethodInsnNode;
import org.objectweb.asm.tree.MethodNode;
import org.objectweb.asm.tree.VarInsnNode;

public final class MinecraftTransformer extends ClassNodeTransformer {
    private static final String SET_SERVER_NAME_DESC = "(Ljava/lang/String;I)V";
    private static final String QUICK_PLAY_NAME = "modrinth$quickPlay";
    private static final String QUICK_PLAY_DESC = "(Lnet/minecraft/client/Minecraft;)V";
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
        if (constructor == null || setServerName == null) {
            return false;
        }

        final ListIterator<AbstractInsnNode> it = constructor.instructions.iterator();
        if (!InsnPattern.findAndSkip(it, INITIALIZE_THIS_PATTERN)) {
            return true;
        }

        it.add(new VarInsnNode(Opcodes.ALOAD, 0));
        it.add(new MethodInsnNode(Opcodes.INVOKESTATIC, classNode.name, QUICK_PLAY_NAME, QUICK_PLAY_DESC));

        final MethodNode quickPlay = new MethodNode(
                Opcodes.ACC_PRIVATE | Opcodes.ACC_STATIC | Opcodes.ACC_SYNTHETIC,
                QUICK_PLAY_NAME,
                QUICK_PLAY_DESC,
                null,
                null);
        final InsnList insns = quickPlay.instructions;
        final LabelNode doneLabel = new LabelNode();
        insns.add(new LdcInsnNode("modrinth.internal.quickPlay.host"));
        insns.add(new MethodInsnNode(
                Opcodes.INVOKESTATIC, "java/lang/System", "getProperty", "(Ljava/lang/String;)Ljava/lang/String;"));
        insns.add(new VarInsnNode(Opcodes.ASTORE, 1));
        insns.add(new VarInsnNode(Opcodes.ALOAD, 1));
        insns.add(new JumpInsnNode(Opcodes.IFNULL, doneLabel));
        insns.add(new VarInsnNode(Opcodes.ALOAD, 0));
        insns.add(new VarInsnNode(Opcodes.ALOAD, 1));
        insns.add(new LdcInsnNode("modrinth.internal.quickPlay.port"));
        insns.add(new MethodInsnNode(
                Opcodes.INVOKESTATIC, "java/lang/System", "getProperty", "(Ljava/lang/String;)Ljava/lang/String;"));
        insns.add(new MethodInsnNode(Opcodes.INVOKESTATIC, "java/lang/Integer", "parseInt", "(Ljava/lang/String;)I"));
        insns.add(new MethodInsnNode(Opcodes.INVOKEVIRTUAL, classNode.name, setServerName, SET_SERVER_NAME_DESC));
        insns.add(doneLabel);
        if (needsStackMap(classNode)) {
            insns.add(new FrameNode(Opcodes.F_APPEND, 1, new Object[] {"java/lang/String"}, 0, null));
        }
        insns.add(new InsnNode(Opcodes.RETURN));
        classNode.methods.add(quickPlay);

        return true;
    }
}
