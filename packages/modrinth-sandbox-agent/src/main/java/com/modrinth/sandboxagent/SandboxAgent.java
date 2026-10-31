package com.modrinth.sandboxagent;

import java.io.DataInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.lang.instrument.Instrumentation;
import java.nio.charset.StandardCharsets;

import com.modrinth.sandboxagent.transformation.*;
import com.modrinth.sandboxagent.transformer.TransformerMap;

public class SandboxAgent {
	public static void premain(String agentArgs, Instrumentation inst) {
		instrument(inst);
	}

	private static void instrument(Instrumentation inst) {
        TransformerMap transformerMap = new TransformerMap();

        String minecraftVersionJson = readResourceToString("/version.json");
        MinecraftVersionInfo minecraftVersionInfo = new MinecraftVersionInfo(minecraftVersionJson);

        if (System.getProperty("modrinth.sandboxagent.transform.openuri", "").equals("true")) {
            // Replaces Util$OS#openUri with sandbox api call
            OpenUriTransformation.register(transformerMap, minecraftVersionInfo);
        }
        if (System.getProperty("modrinth.sandboxagent.transform.dragdrop", "").equals("true")) {
            // Requests access to certain files outside the sandbox when drag-dropping
            DropFileTransformation.register(transformerMap, minecraftVersionInfo);
        }
        if (System.getProperty("modrinth.sandboxagent.transform.filedialog", "").equals("true")) {
            // Transform various file dialog APIs (SDL, NFD, TFD) to use our api instead
            FileDialogTransformation.register(transformerMap);
        }
        if (System.getProperty("modrinth.sandboxagent.transform.auth", "").equals("true")) {
            // Replaces realms url generation to point to sandbox api
            RealmsTransformation.register(transformerMap, minecraftVersionInfo);
            // Modifies auth code on older versions of Minecraft to use our proxy
            AuthTransformation.register(transformerMap, minecraftVersionInfo);
        }

		inst.addTransformer(transformerMap);
	}

    public static String readResourceToString(String file) {
        try (InputStream is = SandboxAgent.class.getResourceAsStream(file)) {
            if (is == null) {
                return null;
            }
            return readInputStreamToString(is);
        } catch (IOException ignored) {}
        return null;
    }

    public static String readInputStreamToString(InputStream is) throws IOException {
        byte[] bytes = new byte[is.available()];

        DataInputStream dataInputStream = new DataInputStream(is);
        dataInputStream.readFully(bytes);

        return new String(bytes, StandardCharsets.UTF_8);
    }

}
