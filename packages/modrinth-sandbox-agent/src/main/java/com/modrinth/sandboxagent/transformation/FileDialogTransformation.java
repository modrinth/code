package com.modrinth.sandboxagent.transformation;

import com.modrinth.sandboxagent.implementation.NFDFileDialog;
import com.modrinth.sandboxagent.implementation.SDLFileDialog;
import com.modrinth.sandboxagent.implementation.TinyFileDialog;
import com.modrinth.sandboxagent.transformer.TransformerMap;

public class FileDialogTransformation {

    public static void register(TransformerMap transformerMap) {
        // SDL
        String sdl = "org/lwjgl/sdl/SDLDialog";
        Transformation.registerOverwrite(transformerMap, sdl, "nSDL_ShowOpenFileDialog", "(JJJJIJZ)V", SDLFileDialog.class, "handleSdlOpen");
        Transformation.registerOverwrite(transformerMap, sdl, "nSDL_ShowSaveFileDialog", "(JJJJIJ)V", SDLFileDialog.class, "handleSdlSave");
        Transformation.registerOverwrite(transformerMap, sdl, "nSDL_ShowOpenFolderDialog", "(JJJJZ)V", SDLFileDialog.class, "handleSdlFolder");
        Transformation.registerOverwrite(transformerMap, sdl, "nSDL_ShowFileDialogWithProperties", "(IJJI)V", SDLFileDialog.class, "handleSdlWithProperties");

        // NFD (Native File Dialogs)
        String nfd = "org/lwjgl/util/nfd/NativeFileDialog";
        String pointerBuffer = "Lorg/lwjgl/PointerBuffer;";
        String filterList = "Lorg/lwjgl/util/nfd/NFDFilterItem$Buffer;";
        String byteBuffer = "Ljava/nio/ByteBuffer;";
        String charSequence = "Ljava/lang/CharSequence;";
        String pathSetEnum = "Lorg/lwjgl/util/nfd/NFDPathSetEnum;";

        Transformation.registerOverwriteConditional(transformerMap, nfd, "NFD_FreePath", "(" + byteBuffer + ")V", NFDFileDialog.class, "handleNfdFreeBuffer");
        Transformation.registerOverwriteConditional(transformerMap, nfd, "NFD_FreePath", "(J)V", NFDFileDialog.class, "handleNfdFreeLong");

        Transformation.registerOverwrite(transformerMap, nfd, "NFD_OpenDialog", "(" + pointerBuffer + filterList + byteBuffer + ")I", NFDFileDialog.class, "handleNfdOpen");
        Transformation.registerOverwrite(transformerMap, nfd, "NFD_OpenDialog", "(" + pointerBuffer + filterList + charSequence + ")I", NFDFileDialog.class, "handleNfdOpen");
        Transformation.registerOverwrite(transformerMap, nfd, "NFD_OpenDialog_With", "(" + pointerBuffer + "Lorg/lwjgl/util/nfd/NFDOpenDialogArgs;)I", NFDFileDialog.class, "handleNfdOpen");

        Transformation.registerOverwrite(transformerMap, nfd, "NFD_OpenDialogMultiple", "(" + pointerBuffer + filterList + byteBuffer + ")I", NFDFileDialog.class, "handleNfdOpenMultiple");
        Transformation.registerOverwrite(transformerMap, nfd, "NFD_OpenDialogMultiple", "(" + pointerBuffer + filterList + charSequence + ")I", NFDFileDialog.class, "handleNfdOpenMultiple");
        Transformation.registerOverwrite(transformerMap, nfd, "NFD_OpenDialogMultiple_With", "(" + pointerBuffer + "Lorg/lwjgl/util/nfd/NFDOpenDialogArgs;)I", NFDFileDialog.class, "handleNfdOpenMultiple");

        Transformation.registerOverwrite(transformerMap, nfd, "NFD_SaveDialog", "(" + pointerBuffer + filterList + byteBuffer + byteBuffer + ")I", NFDFileDialog.class, "handleNfdSave");
        Transformation.registerOverwrite(transformerMap, nfd, "NFD_SaveDialog", "(" + pointerBuffer + filterList + charSequence + charSequence + ")I", NFDFileDialog.class, "handleNfdSave");
        Transformation.registerOverwrite(transformerMap, nfd, "NFD_SaveDialog_With", "(" + pointerBuffer + "Lorg/lwjgl/util/nfd/NFDSaveDialogArgs;)I", NFDFileDialog.class, "handleNfdSave");

        Transformation.registerOverwrite(transformerMap, nfd, "NFD_PickFolder", "(" + pointerBuffer + byteBuffer + ")I", NFDFileDialog.class, "handleNfdFolder");
        Transformation.registerOverwrite(transformerMap, nfd, "NFD_PickFolder", "(" + pointerBuffer + charSequence + ")I", NFDFileDialog.class, "handleNfdFolder");
        Transformation.registerOverwrite(transformerMap, nfd, "NFD_PickFolder_With", "(" + pointerBuffer + "Lorg/lwjgl/util/nfd/NFDPickFolderArgs;)I", NFDFileDialog.class, "handleNfdFolder");

        Transformation.registerOverwrite(transformerMap, nfd, "NFD_PickFolderMultiple", "(" + pointerBuffer + byteBuffer + ")I", NFDFileDialog.class, "handleNfdFolderMultiple");
        Transformation.registerOverwrite(transformerMap, nfd, "NFD_PickFolderMultiple", "(" + pointerBuffer + charSequence + ")I", NFDFileDialog.class, "handleNfdFolderMultiple");
        Transformation.registerOverwrite(transformerMap, nfd, "NFD_PickFolderMultiple_With", "(" + pointerBuffer + "Lorg/lwjgl/util/nfd/NFDPickFolderArgs;)I", NFDFileDialog.class, "handleNfdFolderMultiple");

        // NFD path sets, returned by the "Multiple" dialogs
        Transformation.registerOverwriteConditional(transformerMap, nfd, "NFD_PathSet_GetCount", "(JLjava/nio/IntBuffer;)I", NFDFileDialog.class, "handlePathSetGetCount");
        Transformation.registerOverwriteConditional(transformerMap, nfd, "NFD_PathSet_GetCount", "(J[I)I", NFDFileDialog.class, "handlePathSetGetCount");
        Transformation.registerOverwriteConditional(transformerMap, nfd, "NFD_PathSet_GetPath", "(JI" + pointerBuffer + ")I", NFDFileDialog.class, "handlePathSetGetPath");
        Transformation.registerOverwriteConditional(transformerMap, nfd, "NFD_PathSet_GetEnum", "(J" + pathSetEnum + ")I", NFDFileDialog.class, "handlePathSetGetEnum");
        Transformation.registerOverwriteConditional(transformerMap, nfd, "NFD_PathSet_EnumNext", "(" + pathSetEnum + pointerBuffer + ")I", NFDFileDialog.class, "handlePathSetEnumNext");
        Transformation.registerOverwriteConditional(transformerMap, nfd, "NFD_PathSet_FreePath", "(" + byteBuffer + ")V", NFDFileDialog.class, "handleNfdFreeBuffer");
        Transformation.registerOverwriteConditional(transformerMap, nfd, "NFD_PathSet_FreePath", "(J)V", NFDFileDialog.class, "handleNfdFreeLong");
        Transformation.registerOverwriteConditional(transformerMap, nfd, "NFD_PathSet_FreeEnum", "(" + pathSetEnum + ")V", NFDFileDialog.class, "handlePathSetFreeEnum");
        Transformation.registerOverwriteConditional(transformerMap, nfd, "NFD_PathSet_Free", "(J)V", NFDFileDialog.class, "handlePathSetFree");

        // TFD (Tiny File Dialogs)
        String tfd = "org/lwjgl/util/tinyfd/TinyFileDialogs";
        String string = "Ljava/lang/String;";

        Transformation.registerOverwrite(transformerMap, tfd, "tinyfd_openFileDialog", "(" + byteBuffer + byteBuffer + pointerBuffer + byteBuffer + "Z)" + string, TinyFileDialog.class, "handleTinyfdOpen");
        Transformation.registerOverwrite(transformerMap, tfd, "tinyfd_openFileDialog", "(" + charSequence + charSequence + pointerBuffer + charSequence + "Z)" + string, TinyFileDialog.class, "handleTinyfdOpen");
        Transformation.registerOverwrite(transformerMap, tfd, "tinyfd_saveFileDialog", "(" + byteBuffer + byteBuffer + pointerBuffer + byteBuffer + ")" + string, TinyFileDialog.class, "handleTinyfdSave");
        Transformation.registerOverwrite(transformerMap, tfd, "tinyfd_saveFileDialog", "(" + charSequence + charSequence + pointerBuffer + charSequence + ")" + string, TinyFileDialog.class, "handleTinyfdSave");
        Transformation.registerOverwrite(transformerMap, tfd, "tinyfd_selectFolderDialog", "(" + byteBuffer + byteBuffer + ")" + string, TinyFileDialog.class, "handleTinyfdFolder");
        Transformation.registerOverwrite(transformerMap, tfd, "tinyfd_selectFolderDialog", "(" + charSequence + charSequence + ")" + string, TinyFileDialog.class, "handleTinyfdFolder");
    }

}
