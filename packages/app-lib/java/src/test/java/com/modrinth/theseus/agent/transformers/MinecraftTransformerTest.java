package com.modrinth.theseus.agent.transformers;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;
import org.objectweb.asm.ClassReader;
import org.objectweb.asm.ClassWriter;
import org.objectweb.asm.MethodVisitor;
import org.objectweb.asm.Opcodes;

class MinecraftTransformerTest {
    private static final String MINECRAFT = "net/minecraft/client/Minecraft";

    @ParameterizedTest
    @ValueSource(ints = {Opcodes.V1_5, Opcodes.V1_8})
    void injectedQuickPlayVerifiesAndRuns(int classVersion) throws Exception {
        System.setProperty("modrinth.internal.quickPlay.serverVersion", "INJECTED");
        System.setProperty("modrinth.internal.quickPlay.host", "example.com");
        System.setProperty("modrinth.internal.quickPlay.port", "25566");

        final ClassReader reader = new ClassReader(fakeMinecraft(classVersion));
        final ClassWriter writer = new ClassWriter(reader, ClassWriter.COMPUTE_MAXS);
        assertTrue(new MinecraftTransformer().transform(reader, writer));
        final byte[] transformed = writer.toByteArray();

        final Class<?> minecraft = new ClassLoader() {
            Class<?> define() {
                return defineClass(MINECRAFT.replace('/', '.'), transformed, 0, transformed.length);
            }
        }.define();
        minecraft.getDeclaredConstructor().newInstance();
        assertEquals("example.com", minecraft.getField("host").get(null));
        assertEquals(25566, minecraft.getField("port").get(null));
    }

    private static byte[] fakeMinecraft(int classVersion) {
        final ClassWriter writer = new ClassWriter(0);
        writer.visit(classVersion, Opcodes.ACC_PUBLIC, MINECRAFT, null, "java/lang/Object", null);
        writer.visitField(Opcodes.ACC_PUBLIC | Opcodes.ACC_STATIC, "host", "Ljava/lang/String;", null, null);
        writer.visitField(Opcodes.ACC_PUBLIC | Opcodes.ACC_STATIC, "port", "I", null, null);

        MethodVisitor method = writer.visitMethod(Opcodes.ACC_PUBLIC, "<init>", "()V", null, null);
        method.visitVarInsn(Opcodes.ALOAD, 0);
        method.visitMethodInsn(Opcodes.INVOKESPECIAL, "java/lang/Object", "<init>", "()V", false);
        method.visitInsn(Opcodes.RETURN);
        method.visitMaxs(1, 1);

        method = writer.visitMethod(Opcodes.ACC_PUBLIC, "setServer", "(Ljava/lang/String;I)V", null, null);
        method.visitVarInsn(Opcodes.ALOAD, 1);
        method.visitFieldInsn(Opcodes.PUTSTATIC, MINECRAFT, "host", "Ljava/lang/String;");
        method.visitVarInsn(Opcodes.ILOAD, 2);
        method.visitFieldInsn(Opcodes.PUTSTATIC, MINECRAFT, "port", "I");
        method.visitInsn(Opcodes.RETURN);
        method.visitMaxs(1, 3);

        return writer.toByteArray();
    }
}
