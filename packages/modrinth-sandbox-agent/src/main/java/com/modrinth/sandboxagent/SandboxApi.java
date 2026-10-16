package com.modrinth.sandboxagent;

import java.io.IOException;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.nio.file.Paths;

public class SandboxApi {

	private static SandboxApi sandboxApi = null;

    private final String sandboxApiAuthorization;
    private final String baseUrl;

    public SandboxApi(String secret, int port) {
        this.sandboxApiAuthorization = "Bearer " + secret;
        this.baseUrl = "http://127.0.0.1:" + port;
    }

    public static SandboxApi getOrInit() {
		if (sandboxApi != null) {
			return sandboxApi;
		}
		String secret = System.getProperty("modrinth.sandboxagent.secret", "");
		if (secret.isEmpty()) {
			throw new RuntimeException("Missing -Dmodrinth.sandboxagent.secret argument");
		}
		String portStr = System.getProperty("modrinth.sandboxagent.port", "");
		if (portStr.isEmpty()) {
			throw new RuntimeException("Missing -Dmodrinth.sandboxagent.port argument");
		}
		int port = Integer.parseInt(portStr);

		sandboxApi = new SandboxApi(secret, port);
		return sandboxApi;
    }

    public String getBaseUrl() {
        return this.baseUrl;
    }

    public void openUri(URI uri) {
        HttpURLConnection conn = null;
        try {
            String encodedUri = URLEncoder.encode(uri.toString(), StandardCharsets.UTF_8.toString());
            conn = (HttpURLConnection) new URL(this.baseUrl + "/sandboxapi/openUri?uri="+encodedUri).openConnection();
            conn.setRequestProperty("Authorization", this.sandboxApiAuthorization);
            conn.setRequestMethod("POST");
            int responseCode = conn.getResponseCode();
            if (responseCode != HttpURLConnection.HTTP_OK) {
                System.err.println("Unable to use sandboxapi openUri, got error code " + responseCode);
            }
        } catch (IOException e) {
            e.printStackTrace();
        } finally {
            if (conn != null) {
                conn.disconnect();
            }
        }
    }

    public Path tryRequestAccess(Path originalPath) {
        HttpURLConnection conn = null;
        try {
            String encodedPath = URLEncoder.encode(originalPath.toString(), StandardCharsets.UTF_8.toString());
            conn = (HttpURLConnection) new URL(this.baseUrl + "/sandboxapi/requestAccess?path="+encodedPath).openConnection();
            conn.setRequestProperty("Authorization", this.sandboxApiAuthorization);
            conn.setRequestMethod("GET");
            int responseCode = conn.getResponseCode();
            if (responseCode != HttpURLConnection.HTTP_OK) {
                System.err.println("Unable to use sandboxapi requestAccess, got error code " + responseCode);
                return originalPath;
            }
            return Paths.get(SandboxAgent.readInputStreamToString(conn.getInputStream()));
        } catch (IOException e) {
            e.printStackTrace();
        } finally {
            if (conn != null) {
                conn.disconnect();
            }
        }
        return originalPath;
    }

}
