package com.modrinth.sandboxagent.implementation;

import com.modrinth.sandboxagent.SandboxApi;
import org.lwjgl.PointerBuffer;
import org.lwjgl.sdl.*;
import org.lwjgl.system.Callback;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;

import java.nio.ByteBuffer;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CompletableFuture;

public class SDLFileDialog {

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
        System.out.println("Using launcher SandboxApi to handle file dialog");

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
                for (String individualPattern : pattern.split("[;,]")) {
                    individualPattern = individualPattern.trim();
                    if (individualPattern.isEmpty()) {
                        continue;
                    }

                    String[] parts = individualPattern.split("\\.");
                    String extension = parts[parts.length - 1];

                    String name = SDL_DialogFileFilter.nnameString(struct);
                    filters.add(extension + "," + name);
                }
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
