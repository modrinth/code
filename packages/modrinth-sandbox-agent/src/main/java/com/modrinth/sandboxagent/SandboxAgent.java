package com.modrinth.sandboxagent;

import java.io.DataInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.lang.instrument.ClassFileTransformer;
import java.lang.instrument.IllegalClassFormatException;
import java.lang.instrument.Instrumentation;
import java.net.URI;
import java.nio.charset.Charset;
import java.nio.charset.StandardCharsets;
import java.security.ProtectionDomain;
import java.util.List;
import java.util.ListIterator;
import java.util.concurrent.CompletableFuture;

import com.google.gson.Gson;
import com.google.gson.JsonObject;
import com.modrinth.sandboxagent.transformation.AuthTransformation;
import com.modrinth.sandboxagent.transformation.DropFileTransformation;
import com.modrinth.sandboxagent.transformation.OpenUriTransformation;
import com.modrinth.sandboxagent.transformation.RealmsTransformation;
import com.modrinth.sandboxagent.transformer.TransformerMap;
import org.objectweb.asm.*;
import org.objectweb.asm.tree.*;

public class SandboxAgent {
	public static void premain(String agentArgs, Instrumentation inst) {
		instrument(inst);
	}

	private static void instrument(Instrumentation inst) {
        TransformerMap transformerMap = new TransformerMap();

        String minecraftVersionJson = readResourceToString("/version.json");
        MinecraftVersionInfo minecraftVersionInfo = new MinecraftVersionInfo(minecraftVersionJson);

        // Replaces Util$OS#openUri with sandbox api call
        OpenUriTransformation.register(transformerMap, minecraftVersionInfo);
        // Replaces realms url generation to point to sandbox api
        RealmsTransformation.register(transformerMap, minecraftVersionInfo);
        // Requests access to certain files outside the sandbox when drag-dropping
        DropFileTransformation.register(transformerMap, minecraftVersionInfo);
        // Modifies auth code on older versions of Minecraft to use our proxy
        AuthTransformation.register(transformerMap, minecraftVersionInfo);

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
