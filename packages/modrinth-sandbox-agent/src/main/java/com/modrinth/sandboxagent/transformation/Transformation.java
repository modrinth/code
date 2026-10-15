package com.modrinth.sandboxagent.transformation;

import com.modrinth.sandboxagent.MinecraftVersionInfo;
import com.modrinth.sandboxagent.SandboxAgent;
import com.modrinth.sandboxagent.transformer.MethodTransformer;
import com.modrinth.sandboxagent.transformer.TransformerMap;

public class Transformation {

    public static void registerObfuscated(TransformerMap transformerMap, MethodTransformer transformer, MinecraftVersionInfo minecraftVersionInfo, String mappings) {
        if (!minecraftVersionInfo.isObfuscated()) {
            return;
        }

        String csv = SandboxAgent.readResourceToString(mappings);
        if (csv == null) {
            System.err.println("Missing " + mappings);
            return;
        }
        int start = 0;
        while (true) {
            int lineEnd = csv.indexOf('\n', start);
            int column1End = csv.indexOf(',', start);
            if (column1End < 0) {
                break;
            } else if (lineEnd >= 0 && column1End > lineEnd) {
                continue;
            }

            String sub = csv.substring(start, column1End);
            if (sub.equals(minecraftVersionInfo.id) || sub.equals(minecraftVersionInfo.name)) {
                registerObfuscatedEntry(transformerMap, transformer, csv, column1End, lineEnd);
            }

            if (lineEnd < 0) {
                break;
            }
            start = lineEnd+1;
        }
    }

    private static void registerObfuscatedEntry(TransformerMap transformerMap, MethodTransformer transformer, String csv, int column1End, int lineEnd) {
        int column2End = csv.indexOf(',', column1End+1);
        int column3End = csv.indexOf(',', column2End+1);
        int column4End = csv.indexOf(',', column3End+1);
        if (lineEnd >= 0) {
            if (column2End > lineEnd) {
                return;
            }
            if (column3End > lineEnd) {
                return;
            }
            if (column4End > lineEnd) {
                column4End = lineEnd;
            }
        }
        if (column2End < 0 || column3End < 0 || column4End < 0) {
            return;
        }

        String className = csv.substring(column1End+1, column2End);
        String method = csv.substring(column2End+1, column3End);
        String desc = csv.substring(column3End+1, column4End);

        transformerMap.registerMethod(className, method, desc, transformer);
    }

}
