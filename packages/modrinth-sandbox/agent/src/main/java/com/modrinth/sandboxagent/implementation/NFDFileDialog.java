package com.modrinth.sandboxagent.implementation;

import com.modrinth.sandboxagent.SandboxApi;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryUtil;
import org.lwjgl.util.nfd.*;

import java.io.File;
import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.OptionalInt;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

public class NFDFileDialog {

    private static final Set<Long> overridenAllocations = ConcurrentHashMap.newKeySet();
    private static final Map<Long, List<String>> overriddenPathSets = new ConcurrentHashMap<>();
    private static final Map<Long, PathSetCursor> overriddenEnums = new ConcurrentHashMap<>();

    private static final class PathSetCursor {
        private final List<String> paths;
        private int next = 0;

        private PathSetCursor(List<String> paths) {
            this.paths = paths;
        }
    }

    private static long allocatePath(String path) {
        long allocation = MemoryUtil.memAddress(MemoryUtil.memUTF8(path));
        overridenAllocations.add(allocation);
        return allocation;
    }

    public static boolean handleNfdFreeBuffer(ByteBuffer path) {
        return path != null && handleNfdFreeLong(MemoryUtil.memAddress(path));
    }

    public static boolean handleNfdFreeLong(long path) {
        if (overridenAllocations.remove(path)) {
            MemoryUtil.nmemFree(path);
            return true;
        }
        return false;
    }

    public static OptionalInt handlePathSetGetCount(long pathSet, IntBuffer count) {
        List<String> paths = overriddenPathSets.get(pathSet);
        if (paths == null) {
            return OptionalInt.empty();
        }
        count.put(count.position(), paths.size());
        return OptionalInt.of(NativeFileDialog.NFD_OKAY);
    }

    public static OptionalInt handlePathSetGetCount(long pathSet, int[] count) {
        List<String> paths = overriddenPathSets.get(pathSet);
        if (paths == null) {
            return OptionalInt.empty();
        }
        count[0] = paths.size();
        return OptionalInt.of(NativeFileDialog.NFD_OKAY);
    }

    public static OptionalInt handlePathSetGetPath(long pathSet, int index, PointerBuffer outPath) {
        List<String> paths = overriddenPathSets.get(pathSet);
        if (paths == null) {
            return OptionalInt.empty();
        }
        if (index < 0 || index >= paths.size()) {
            return OptionalInt.of(NativeFileDialog.NFD_ERROR);
        }
        outPath.put(outPath.position(), allocatePath(paths.get(index)));
        return OptionalInt.of(NativeFileDialog.NFD_OKAY);
    }

    public static OptionalInt handlePathSetGetEnum(long pathSet, NFDPathSetEnum outEnumerator) {
        List<String> paths = overriddenPathSets.get(pathSet);
        if (paths == null) {
            return OptionalInt.empty();
        }
        long cursor = MemoryUtil.nmemAllocChecked(1);
        overriddenEnums.put(cursor, new PathSetCursor(paths));
        MemoryUtil.memPutAddress(outEnumerator.address(), cursor);
        return OptionalInt.of(NativeFileDialog.NFD_OKAY);
    }

    public static OptionalInt handlePathSetEnumNext(NFDPathSetEnum enumerator, PointerBuffer outPath) {
        if (enumerator == null) {
            return OptionalInt.empty();
        }
        PathSetCursor cursor = overriddenEnums.get(MemoryUtil.memGetAddress(enumerator.address()));
        if (cursor == null) {
            return OptionalInt.empty();
        }
        synchronized (cursor) {
            if (cursor.next < cursor.paths.size()) {
                outPath.put(outPath.position(), allocatePath(cursor.paths.get(cursor.next++)));
            } else {
                outPath.put(outPath.position(), MemoryUtil.NULL);
            }
        }
        return OptionalInt.of(NativeFileDialog.NFD_OKAY);
    }

    public static boolean handlePathSetFreeEnum(NFDPathSetEnum enumerator) {
        if (enumerator == null) {
            return false;
        }
        long cursor = MemoryUtil.memGetAddress(enumerator.address());
        if (overriddenEnums.remove(cursor) != null) {
            MemoryUtil.nmemFree(cursor);
            MemoryUtil.memPutAddress(enumerator.address(), MemoryUtil.NULL);
            return true;
        }
        return false;
    }

    public static boolean handlePathSetFree(long pathSet) {
        if (overriddenPathSets.remove(pathSet) != null) {
            MemoryUtil.nmemFree(pathSet);
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
        } else if (multiple) {
            List<String> pathStrings = new ArrayList<>(paths.size());
            for (Path path : paths) {
                pathStrings.add(path.toString());
            }
            long pathSet = MemoryUtil.nmemAllocChecked(1);
            overriddenPathSets.put(pathSet, pathStrings);
            outPath.put(outPath.position(), pathSet);
            return NativeFileDialog.NFD_OKAY;
        } else {
            outPath.put(outPath.position(), allocatePath(paths.get(0).toString()));
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
