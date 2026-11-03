package com.modrinth.sandboxagent.implementation;

import com.modrinth.sandboxagent.SandboxApi;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryUtil;
import org.lwjgl.util.nfd.*;

import java.io.File;
import java.nio.ByteBuffer;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;

public class NFDFileDialog {

    private static final HashSet<Long> overridenAllocations = new HashSet<>();

    public static boolean handleNfdFreeBuffer(ByteBuffer path) {
        return handleNfdFreeLong(MemoryUtil.memAddress(path));
    }

    public static boolean handleNfdFreeLong(long path) {
        if (overridenAllocations.remove(path)) {
            MemoryUtil.nmemFree(path);
            return true;
        }
        return false;
    }

    public static int handleNfd(PointerBuffer outPath, NFDFilterItem.Buffer filterList, CharSequence defaultPath, boolean save, boolean multiple, boolean directory) {
        System.out.println("Using launcher SandboxApi to handle file dialog");

        String locationString = defaultPath == null ? null : defaultPath.toString();

        List<String> filters = new ArrayList<>();
        if (filterList != null) {
            for (NFDFilterItem item : filterList) {
                for (String individualPattern : item.specString().split("[;,]")) {
                    individualPattern = individualPattern.trim();
                    if (individualPattern.isEmpty()) {
                        continue;
                    }

                    String[] parts = individualPattern.split("\\.");
                    String extension = parts[parts.length - 1];

                    String name = item.nameString();
                    filters.add(extension + "," + name);
                }
            }
        }

        List<Path> paths = SandboxApi.getOrInit().openFileDialog(null, null, save, multiple,
                directory, filters, locationString);

        if (paths == null) {
            return NativeFileDialog.NFD_ERROR;
        } else if (paths.isEmpty()) {
            return NativeFileDialog.NFD_CANCEL;
        } else {
            Path path = paths.get(0);
            long allocation = MemoryUtil.memAddress(MemoryUtil.memUTF8(path.toString()));
            overridenAllocations.add(allocation);
            outPath.put(allocation);
            return NativeFileDialog.NFD_OKAY;
        }
    }

    public static int handleNfdOpen(PointerBuffer outPath, NFDFilterItem.Buffer filterList, ByteBuffer defaultPath) {
        return handleNfd(outPath, filterList, MemoryUtil.memUTF8Safe(defaultPath), false, false, false);
    }

    public static int handleNfdOpen(PointerBuffer outPath, NFDFilterItem.Buffer filterList, CharSequence defaultPath) {
        return handleNfd(outPath, filterList, defaultPath, false, false, false);
    }

    public static int handleNfdOpen(PointerBuffer outPath, NFDOpenDialogArgs args) {
        return handleNfd(outPath, args.filterList(), args.defaultPathString(), false, false, false);
    }

    public static int handleNfdOpenMultiple(PointerBuffer outPath, NFDFilterItem.Buffer filterList, ByteBuffer defaultPath) {
        return handleNfd(outPath, filterList, MemoryUtil.memUTF8Safe(defaultPath), false, true, false);
    }

    public static int handleNfdOpenMultiple(PointerBuffer outPath, NFDFilterItem.Buffer filterList, CharSequence defaultPath) {
        return handleNfd(outPath, filterList, defaultPath, false, true, false);
    }

    public static int handleNfdOpenMultiple(PointerBuffer outPath, NFDOpenDialogArgs args) {
        return handleNfd(outPath, args.filterList(), args.defaultPathString(), false, true, false);
    }

    private static String makeDefaultPathFromFolderAndName(CharSequence defaultFolder, CharSequence defaultName) {
        StringBuilder path = new StringBuilder();
        if (defaultFolder != null) {
            path.append(defaultFolder);
        } else {
            path.append(System.getProperty("user.dir", "."));
        }

        if (!path.toString().endsWith(File.separator)) {
            path.append(File.separatorChar);
        }

        if (defaultName != null) {
            path.append(defaultName);
        }

        return path.toString();
    }

    public static int handleNfdSave(PointerBuffer outPath, NFDFilterItem.Buffer filterList, ByteBuffer defaultFolder, ByteBuffer defaultName) {
        String defaultPath = makeDefaultPathFromFolderAndName(MemoryUtil.memUTF8Safe(defaultFolder), MemoryUtil.memUTF8Safe(defaultName));
        return handleNfd(outPath, filterList, defaultPath, true, false, false);
    }

    public static int handleNfdSave(PointerBuffer outPath, NFDFilterItem.Buffer filterList, CharSequence defaultFolder, CharSequence defaultName) {
        String defaultPath = makeDefaultPathFromFolderAndName(defaultFolder, defaultName);
        return handleNfd(outPath, filterList, defaultPath, true, false, false);
    }

    public static int handleNfdSave(PointerBuffer outPath, NFDSaveDialogArgs args) {
        String defaultPath = makeDefaultPathFromFolderAndName(args.defaultPathString(), args.defaultNameString());
        return handleNfd(outPath, args.filterList(), defaultPath, true, false, false);
    }

    public static int handleNfdFolder(PointerBuffer outPath, ByteBuffer defaultPath) {
        return handleNfd(outPath, null, MemoryUtil.memUTF8Safe(defaultPath), false, false, true);
    }

    public static int handleNfdFolder(PointerBuffer outPath, CharSequence defaultPath) {
        return handleNfd(outPath, null, defaultPath, false, false, true);
    }

    public static int handleNfdFolder(PointerBuffer outPath, NFDPickFolderArgs args) {
        return handleNfd(outPath, null, args.defaultPathString(), false, false, true);
    }

    public static int handleNfdFolderMultiple(PointerBuffer outPath, ByteBuffer defaultPath) {
        return handleNfd(outPath, null, MemoryUtil.memUTF8Safe(defaultPath), false, true, true);
    }

    public static int handleNfdFolderMultiple(PointerBuffer outPath, CharSequence defaultPath) {
        return handleNfd(outPath, null, defaultPath, false, true, true);
    }

    public static int handleNfdFolderMultiple(PointerBuffer outPath, NFDPickFolderArgs args) {
        return handleNfd(outPath, null, args.defaultPathString(), false, true, true);
    }

}
