package com.modrinth.sandboxagent;

import java.io.ByteArrayOutputStream;
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

        boolean transformOpenUri = System.getProperty("modrinth.sandboxagent.transform.openuri", "").equals("true");
        if (transformOpenUri) {
            // Replaces Util$OS#openUri with sandbox api call
            OpenUriTransformation.register(transformerMap, minecraftVersionInfo);
            // Replaces running open commands (xdg-open, rundll32, etc.) with sandbox api call
            ProcessBuilderTransformation.register(transformerMap);
            // Replaces awt Desktop#browse/open with sandbox api call
            DesktopTransformation.register(transformerMap);
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

		inst.addTransformer(transformerMap, true);

        if (transformOpenUri) {
            // ProcessBuilder might have been loaded before our transformer was added
            try {
                inst.retransformClasses(ProcessBuilder.class);
            } catch (Throwable t) {
                System.err.println("Unable to transform ProcessBuilder");
                t.printStackTrace();
            }
        }
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
        try (ByteArrayOutputStream baos = new ByteArrayOutputStream()) {
            byte[] chunk = new byte[8192];
            int read;
            while ((read = is.read(chunk)) != -1) {
                baos.write(chunk, 0, read);
            }
            return new String(baos.toByteArray(), StandardCharsets.UTF_8);
        }
    }

}
