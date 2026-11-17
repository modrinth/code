package com.modrinth.sandboxagent.implementation;

import com.modrinth.sandboxagent.SandboxApi;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryUtil;

import java.nio.ByteBuffer;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

public class TinyFileDialog {

    private static final String MULTIPLE_SEPARATOR = "|";

    private static String handleTinyfd(CharSequence title, CharSequence defaultPath, PointerBuffer filterPatterns, CharSequence filterDescription, boolean save, boolean multiple, boolean directory) {
        System.out.println("Using launcher SandboxApi to handle file dialog");

        List<String> filters = new ArrayList<>();
        if (filterPatterns != null) {
            String name = filterDescription == null ? null : filterDescription.toString();
            for (int i = filterPatterns.position(); i < filterPatterns.limit(); i++) {
                String pattern = MemoryUtil.memUTF8Safe(filterPatterns.get(i));
                if (pattern == null) {
                    continue;
                }

                for (String individualPattern : pattern.split("[;,\\s]")) {
                    int dot = individualPattern.lastIndexOf('.');
                    String extension = individualPattern.substring(dot + 1).trim();
                    if (extension.isEmpty() || extension.contains("*")) {
                        continue;
                    }

                    filters.add(extension + "," + (name == null ? individualPattern.trim() : name));
                }
            }
        }

        List<Path> paths = SandboxApi.getOrInit().openFileDialog(
                title == null ? null : title.toString(), null, save, multiple, directory, filters,
                defaultPath == null ? null : defaultPath.toString());

        if (paths == null || paths.isEmpty()) {
            return null;
        }

        StringBuilder result = new StringBuilder(paths.get(0).toString());
        if (multiple) {
            for (int i = 1; i < paths.size(); i++) {
                result.append(MULTIPLE_SEPARATOR).append(paths.get(i));
            }
        }
        return result.toString();
    }

    public static String handleTinyfdOpen(ByteBuffer title, ByteBuffer defaultPathAndFile, PointerBuffer filterPatterns, ByteBuffer singleFilterDescription, boolean allowMultipleSelects) {
        return handleTinyfd(MemoryUtil.memUTF8Safe(title), MemoryUtil.memUTF8Safe(defaultPathAndFile), filterPatterns,
                MemoryUtil.memUTF8Safe(singleFilterDescription), false, allowMultipleSelects, false);
    }

    public static String handleTinyfdOpen(CharSequence title, CharSequence defaultPathAndFile, PointerBuffer filterPatterns, CharSequence singleFilterDescription, boolean allowMultipleSelects) {
        return handleTinyfd(title, defaultPathAndFile, filterPatterns, singleFilterDescription, false, allowMultipleSelects, false);
    }

    public static String handleTinyfdSave(ByteBuffer title, ByteBuffer defaultPathAndFile, PointerBuffer filterPatterns, ByteBuffer singleFilterDescription) {
        return handleTinyfd(MemoryUtil.memUTF8Safe(title), MemoryUtil.memUTF8Safe(defaultPathAndFile), filterPatterns,
                MemoryUtil.memUTF8Safe(singleFilterDescription), true, false, false);
    }

    public static String handleTinyfdSave(CharSequence title, CharSequence defaultPathAndFile, PointerBuffer filterPatterns, CharSequence singleFilterDescription) {
        return handleTinyfd(title, defaultPathAndFile, filterPatterns, singleFilterDescription, true, false, false);
    }

    public static String handleTinyfdFolder(ByteBuffer title, ByteBuffer defaultPath) {
        return handleTinyfd(MemoryUtil.memUTF8Safe(title), MemoryUtil.memUTF8Safe(defaultPath), null, null, false, false, true);
    }

    public static String handleTinyfdFolder(CharSequence title, CharSequence defaultPath) {
        return handleTinyfd(title, defaultPath, null, null, false, false, true);
    }

}
