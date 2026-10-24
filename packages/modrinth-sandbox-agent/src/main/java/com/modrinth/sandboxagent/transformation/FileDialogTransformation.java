package com.modrinth.sandboxagent.transformation;

import com.modrinth.sandboxagent.implementation.NFDFileDialog;
import com.modrinth.sandboxagent.implementation.SDLFileDialog;
import com.modrinth.sandboxagent.transformer.TransformerMap;
import org.objectweb.asm.tree.*;

import java.util.ListIterator;

public class FileDialogTransformation {

    public static void register(TransformerMap transformerMap) {
        // SDL
        transformerMap.registerMethod("org/lwjgl/sdl/SDLDialog", "nSDL_ShowOpenFileDialog", "(JJJJIJZ)V", FileDialogTransformation::transformSdlOpen);
        transformerMap.registerMethod("org/lwjgl/sdl/SDLDialog", "nSDL_ShowSaveFileDialog", "(JJJJIJ)V", FileDialogTransformation::transformSdlSave);
        transformerMap.registerMethod("org/lwjgl/sdl/SDLDialog", "nSDL_ShowSaveFileDialog", "(JJJJZ)V", FileDialogTransformation::transformSdlFolder);
        transformerMap.registerMethod("org/lwjgl/sdl/SDLDialog", "nSDL_ShowFileDialogWithProperties", "(IJJI)V", FileDialogTransformation::transformSdlWithProperties);

        // NFD (Native File Dialogs)
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_FreePath", "(Ljava/nio/ByteBuffer;)V", FileDialogTransformation::transformNfdFreeBuffer);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_FreePath", "(J)V", FileDialogTransformation::transformNfdFreeLong);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_OpenDialog", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDFilterItem$Buffer;Ljava/nio/ByteBuffer;)I", FileDialogTransformation::transformNfdOpenBuf);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_OpenDialog", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDFilterItem$Buffer;Ljava/lang/CharSequence;)I", FileDialogTransformation::transformNfdOpenStr);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_OpenDialog_With", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDOpenDialogArgs;)I", FileDialogTransformation::transformNfdOpenWith);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_OpenDialogMultiple", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDFilterItem$Buffer;Ljava/nio/ByteBuffer;)I", FileDialogTransformation::transformNfdOpenMultipleBuf);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_OpenDialogMultiple", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDFilterItem$Buffer;Ljava/lang/CharSequence;)I", FileDialogTransformation::transformNfdOpenMultipleStr);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_OpenDialogMultiple_With", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDOpenDialogArgs;)I", FileDialogTransformation::transformNfdOpenMultipleWith);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_SaveDialog", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDFilterItem$Buffer;Ljava/nio/ByteBuffer;Ljava/nio/ByteBuffer;)I", FileDialogTransformation::transformNfdSaveBuf);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_SaveDialog", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDFilterItem$Buffer;Ljava/lang/CharSequence;Ljava/lang/CharSequence;)I", FileDialogTransformation::transformNfdSaveStr);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_SaveDialog_With", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDSaveDialogArgs;)I", FileDialogTransformation::transformNfdSaveWith);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_PickFolder", "(Lorg/lwjgl/PointerBuffer;Ljava/nio/ByteBuffer;)I", FileDialogTransformation::transformNfdFolderBuf);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_PickFolder", "(Lorg/lwjgl/PointerBuffer;Ljava/lang/CharSequence;)I", FileDialogTransformation::transformNfdFolderStr);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_PickFolder_With", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDPickFolderArgs;)I", FileDialogTransformation::transformNfdFolderWith);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_PickFolderMultiple", "(Lorg/lwjgl/PointerBuffer;Ljava/nio/ByteBuffer;)I", FileDialogTransformation::transformNfdFolderMultipleBuf);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_PickFolderMultiple", "(Lorg/lwjgl/PointerBuffer;Ljava/lang/CharSequence;)I", FileDialogTransformation::transformNfdFolderMultipleStr);
        transformerMap.registerMethod("org/lwjgl/util/nfd/NativeFileDialog", "NFD_PickFolderMultiple_With", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDPickFolderArgs;)I", FileDialogTransformation::transformNfdFolderMultipleWith);
    }

    private static boolean transformSdlOpen(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, SDLFileDialog.class, "handleSdlOpen", "(JJJJIJZ)V");
        return true;
    }

    private static boolean transformSdlSave(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, SDLFileDialog.class, "handleSdlSave", "(JJJJIJ)V");
        return true;
    }

    private static boolean transformSdlFolder(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, SDLFileDialog.class, "handleSdlFolder", "(JJJJZ)V");
        return true;
    }

    private static boolean transformSdlWithProperties(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, SDLFileDialog.class, "handleSdlWithProperties", "(IJJI)V");
        return true;
    }

    private static boolean transformNfdFreeBuffer(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, true, NFDFileDialog.class, "handleNfdFreeBuffer", "(Ljava/nio/ByteBuffer;)Z");
        return true;
    }

    private static boolean transformNfdFreeLong(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, true, NFDFileDialog.class, "handleNfdFreeLong", "(J)Z");
        return true;
    }

    private static boolean transformNfdOpenBuf(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, NFDFileDialog.class, "handleNfdOpen", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDFilterItem$Buffer;Ljava/nio/ByteBuffer;)I");
        return true;
    }

    private static boolean transformNfdOpenStr(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, NFDFileDialog.class, "handleNfdOpen", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDFilterItem$Buffer;Ljava/lang/CharSequence;)I");
        return true;
    }

    private static boolean transformNfdOpenWith(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, NFDFileDialog.class, "handleNfdOpen", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDOpenDialogArgs;)I");
        return true;
    }

    private static boolean transformNfdOpenMultipleBuf(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, NFDFileDialog.class, "handleNfdOpenMultiple", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDFilterItem$Buffer;Ljava/nio/ByteBuffer;)I");
        return true;
    }

    private static boolean transformNfdOpenMultipleStr(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, NFDFileDialog.class, "handleNfdOpenMultiple", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDFilterItem$Buffer;Ljava/lang/CharSequence;)I");
        return true;
    }

    private static boolean transformNfdOpenMultipleWith(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, NFDFileDialog.class, "handleNfdOpenMultiple", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDOpenDialogArgs;)I");
        return true;
    }

    private static boolean transformNfdSaveBuf(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, NFDFileDialog.class, "handleNfdSave", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDFilterItem$Buffer;Ljava/nio/ByteBuffer;Ljava/nio/ByteBuffer;)I");
        return true;
    }

    private static boolean transformNfdSaveStr(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, NFDFileDialog.class, "handleNfdSave", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDFilterItem$Buffer;Ljava/lang/CharSequence;Ljava/lang/CharSequence;)I");
        return true;
    }

    private static boolean transformNfdSaveWith(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, NFDFileDialog.class, "handleNfdSave", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDSaveDialogArgs;)I");
        return true;
    }

    private static boolean transformNfdFolderBuf(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, NFDFileDialog.class, "handleNfdFolder", "(Lorg/lwjgl/PointerBuffer;Ljava/nio/ByteBuffer;)I");
        return true;
    }

    private static boolean transformNfdFolderStr(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, NFDFileDialog.class, "handleNfdFolder", "(Lorg/lwjgl/PointerBuffer;Ljava/lang/CharSequence;)I");
        return true;
    }

    private static boolean transformNfdFolderWith(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, NFDFileDialog.class, "handleNfdFolder", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDPickFolderArgs;)I");
        return true;
    }

    private static boolean transformNfdFolderMultipleBuf(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, NFDFileDialog.class, "handleNfdFolderMultiple", "(Lorg/lwjgl/PointerBuffer;Ljava/nio/ByteBuffer;)I");
        return true;
    }

    private static boolean transformNfdFolderMultipleStr(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, NFDFileDialog.class, "handleNfdFolderMultiple", "(Lorg/lwjgl/PointerBuffer;Ljava/lang/CharSequence;)I");
        return true;
    }

    private static boolean transformNfdFolderMultipleWith(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        Transformation.overwrite(it, isStatic, false, NFDFileDialog.class, "handleNfdFolderMultiple", "(Lorg/lwjgl/PointerBuffer;Lorg/lwjgl/util/nfd/NFDPickFolderArgs;)I");
        return true;
    }


}
