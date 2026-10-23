package com.modrinth.sandboxagent.transformation;

import com.modrinth.sandboxagent.SandboxApi;
import com.modrinth.sandboxagent.transformer.TransformerMap;
import org.lwjgl.PointerBuffer;
import org.lwjgl.sdl.*;
import org.lwjgl.system.Callback;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.Type;
import org.objectweb.asm.tree.AbstractInsnNode;
import org.objectweb.asm.tree.InsnNode;
import org.objectweb.asm.tree.MethodInsnNode;
import org.objectweb.asm.tree.VarInsnNode;

import java.nio.ByteBuffer;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.ListIterator;
import java.util.concurrent.CompletableFuture;

public class FileDialogTransformation {

    private static final String SELF = FileDialogTransformation.class.getName().replace('.', '/');;

    public static void register(TransformerMap transformerMap) {
        transformerMap.registerMethod("org/lwjgl/sdl/SDLDialog", "nSDL_ShowOpenFileDialog", "(JJJJIJZ)V", FileDialogTransformation::transformSdlOpen);
        transformerMap.registerMethod("org/lwjgl/sdl/SDLDialog", "nSDL_ShowSaveFileDialog", "(JJJJIJ)V", FileDialogTransformation::transformSdlSave);
        transformerMap.registerMethod("org/lwjgl/sdl/SDLDialog", "nSDL_ShowSaveFileDialog", "(JJJJZ)V", FileDialogTransformation::transformSdlFolder);
        transformerMap.registerMethod("org/lwjgl/sdl/SDLDialog", "nSDL_ShowFileDialogWithProperties", "(IJJI)V", FileDialogTransformation::transformSdlWithProperties);
    }

    private static void overwrite(ListIterator<AbstractInsnNode> it, boolean isStatic, String name, String descriptor) {
        Type methodType = Type.getMethodType(descriptor);
        int index = isStatic ? 0 : 1;

        List<Class<?>> argumentClasses = new ArrayList<>();
        for (Type argumentType : methodType.getArgumentTypes()) {
            try {
                argumentClasses.add(toClass(argumentType));
            } catch (ClassNotFoundException e) {
                throw new RuntimeException("Unable to find class", e);
            }

            it.add(new VarInsnNode(argumentType.getOpcode(Opcodes.ILOAD), index));
            index += argumentType.getSize();
        }
        it.add(new MethodInsnNode(Opcodes.INVOKESTATIC, SELF, name, descriptor));
        it.add(new InsnNode(Opcodes.RETURN));

        // Ensure that target method exists
        try {
            FileDialogTransformation.class.getDeclaredMethod(name, argumentClasses.toArray(new Class[0]));
        } catch (NoSuchMethodException e) {
            throw new RuntimeException("Unable to find overwrite method target", e);
        }
    }

    private static Class<?> toClass(Type type) throws ClassNotFoundException {
        int sort = type.getSort();
        switch (sort) {
            case Type.VOID:
                return void.class;
            case Type.BOOLEAN:
                return boolean.class;
            case Type.CHAR:
                return char.class;
            case Type.BYTE:
                return byte.class;
            case Type.SHORT:
                return short.class;
            case Type.INT:
                return int.class;
            case Type.FLOAT:
                return float.class;
            case Type.LONG:
                return long.class;
            case Type.DOUBLE:
                return double.class;
            case Type.ARRAY:
            case Type.OBJECT:
                return Class.forName(type.getClassName(), false, FileDialogTransformation.class.getClassLoader());
            default:
                throw new RuntimeException("Unable to find class for type: " + type);
        }
    }

    private static boolean transformSdlOpen(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        overwrite(it, isStatic, "handleSdlOpen", "(JJJJIJZ)V");
        return true;
    }

    private static boolean transformSdlSave(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        overwrite(it, isStatic, "handleSdlSave", "(JJJJIJ)V");
        return true;
    }

    private static boolean transformSdlFolder(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        overwrite(it, isStatic, "handleSdlFolder", "(JJJJZ)V");
        return true;
    }

    private static boolean transformSdlWithProperties(ListIterator<AbstractInsnNode> it, boolean isStatic) {
        overwrite(it, isStatic, "handleSdlWithProperties", "(IJJI)V");
        return true;
    }

    public static void handleSdlOpen(long callback, long userdata, long window, long filters, int nfilters, long default_location, boolean allow_many) {
        int props = SDLProperties.SDL_CreateProperties();

        SDLProperties.SDL_SetPointerProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_FILTERS_POINTER, filters);
        SDLProperties.SDL_SetNumberProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_NFILTERS_NUMBER, nfilters);
        SDLProperties.SDL_SetPointerProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_WINDOW_POINTER, window);
        SDLProperties.SDL_SetStringProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_LOCATION_STRING, MemoryUtil.memUTF8Safe(default_location));
        SDLProperties.SDL_SetBooleanProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_MANY_BOOLEAN, allow_many);

        handleSdlWithProperties(SDLDialog.SDL_FILEDIALOG_OPENFILE, callback, userdata, props);

        SDLProperties.SDL_DestroyProperties(props);
    }

    public static void handleSdlSave(long callback, long userdata, long window, long filters, int nfilters, long default_location) {
        int props = SDLProperties.SDL_CreateProperties();

        SDLProperties.SDL_SetPointerProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_FILTERS_POINTER, filters);
        SDLProperties.SDL_SetNumberProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_NFILTERS_NUMBER, nfilters);
        SDLProperties.SDL_SetPointerProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_WINDOW_POINTER, window);
        SDLProperties.SDL_SetStringProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_LOCATION_STRING, MemoryUtil.memUTF8Safe(default_location));

        handleSdlWithProperties(SDLDialog.SDL_FILEDIALOG_SAVEFILE, callback, userdata, props);

        SDLProperties.SDL_DestroyProperties(props);
    }

    public static void handleSdlFolder(long callback, long userdata, long window, long default_location, boolean allow_many) {
        int props = SDLProperties.SDL_CreateProperties();

        SDLProperties.SDL_SetPointerProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_WINDOW_POINTER, window);
        SDLProperties.SDL_SetStringProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_LOCATION_STRING, MemoryUtil.memUTF8Safe(default_location));
        SDLProperties.SDL_SetBooleanProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_MANY_BOOLEAN, allow_many);

        handleSdlWithProperties(SDLDialog.SDL_FILEDIALOG_OPENFOLDER, callback, userdata, props);

        SDLProperties.SDL_DestroyProperties(props);
    }

    public static void handleSdlWithProperties(int type, long callback, long userdata, int props) {
        boolean save = type == SDLDialog.SDL_FILEDIALOG_SAVEFILE;
        boolean multiple = SDLProperties.SDL_GetBooleanProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_MANY_BOOLEAN, false);
        boolean directory = type == SDLDialog.SDL_FILEDIALOG_OPENFOLDER;

        long filtersPtr = SDLProperties.SDL_GetPointerProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_FILTERS_POINTER, 0);
        long nfilters = SDLProperties.SDL_GetNumberProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_NFILTERS_NUMBER, 0);

        List<String> filters = new ArrayList<>((int) Math.min(32, nfilters));
        if (filtersPtr != 0 && nfilters > 0) {
            for (long i = 0; i < Math.min(32, nfilters); i++) {
                long struct = filtersPtr + SDL_DialogFileFilter.SIZEOF * i;

                String pattern = SDL_DialogFileFilter.npatternString(struct);
                String[] parts = pattern.split("\\.");
                String extension = parts[parts.length - 1].replace(",", "");

                String name = SDL_DialogFileFilter.nnameString(struct);
                filters.add(extension + "," + name);
            }
        }

        String locationString = SDLProperties.SDL_GetStringProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_LOCATION_STRING, null);
        String titleString = SDLProperties.SDL_GetStringProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_TITLE_STRING, null);
        String acceptString = SDLProperties.SDL_GetStringProperty(props, SDLDialog.SDL_PROP_FILE_DIALOG_ACCEPT_STRING, null);

        CompletableFuture.runAsync(() -> {
            List<Path> paths = SandboxApi.getOrInit().openFileDialog(titleString, acceptString, save, multiple,
                    directory, filters, locationString);
            if (paths == null) {
                SDLError.SDL_SetError("Failed to use launcher SandboxApi to create file dialog");
                SDL_DialogFileCallbackI dialogFileCallback = Callback.getSafe(callback);
                if (dialogFileCallback == null) {
                    throw new RuntimeException("Failed to get SDL_DialogFileCallbackI from callback ptr");
                }
                dialogFileCallback.invoke(userdata, MemoryUtil.NULL, -1);
            } else {
                try (MemoryStack stack = MemoryStack.stackPush()) {
                    PointerBuffer pointerBuffer = stack.callocPointer(paths.size() + 1);

                    for (int i = 0; i < paths.size(); i++) {
                        Path path = paths.get(i);
                        ByteBuffer encoded = stack.UTF8(path.toString(), true);
                        pointerBuffer.put(i, encoded);
                    }

                    SDL_DialogFileCallbackI dialogFileCallback = Callback.getSafe(callback);
                    if (dialogFileCallback == null) {
                        throw new RuntimeException("Failed to get SDL_DialogFileCallbackI from callback ptr");
                    }
                    dialogFileCallback.invoke(userdata, MemoryUtil.memAddress(pointerBuffer), -1);
                }
            }
        });
    }


}
