package com.modrinth.theseus.agent;

import java.util.ListIterator;
import java.util.function.Predicate;
import org.objectweb.asm.Type;
import org.objectweb.asm.tree.AbstractInsnNode;
import org.objectweb.asm.tree.FieldInsnNode;

public interface InsnPattern extends Predicate<AbstractInsnNode> {
    /**
     * Advances past the first match of all instructions in the pattern.
     * @return {@code true} if the pattern was found, {@code false} if not
     */
    static boolean findAndSkip(ListIterator<AbstractInsnNode> iterator, InsnPattern... pattern) {
        if (pattern.length == 0) {
            return true;
        }
        int patternIndex = 0;
        while (iterator.hasNext()) {
            final AbstractInsnNode insn = iterator.next();
            if (insn.getOpcode() == -1) continue;
            if (pattern[patternIndex].test(insn) && ++patternIndex == pattern.length) {
                return true;
            } else {
                patternIndex = 0;
            }
        }
        return false;
    }

    static InsnPattern opcode(int opcode) {
        return insn -> insn.getOpcode() == opcode;
    }

    static InsnPattern field(int opcode, Type fieldType) {
        final String typeDescriptor = fieldType.getDescriptor();
        return insn -> {
            if (insn.getOpcode() != opcode || !(insn instanceof FieldInsnNode)) {
                return false;
            }
            final FieldInsnNode fieldInsn = (FieldInsnNode) insn;
            return typeDescriptor.equals(fieldInsn.desc);
        };
    }
}
