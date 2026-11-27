package com.modrinth.sandboxagent;

import java.io.IOException;
import java.io.UnsupportedEncodingException;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicBoolean;

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
            conn = (HttpURLConnection) new URL(this.baseUrl + "/sandboxapi/open-uri?uri="+encodedUri).openConnection();
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
            conn = (HttpURLConnection) new URL(this.baseUrl + "/sandboxapi/request-access?path="+encodedPath).openConnection();
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

    public List<Path> openFileDialog(String title, String acceptLabel, boolean save, boolean multiple, boolean directory, List<String> filters, String defaultLocation) {
        HttpURLConnection conn = null;
        try {
            StringBuilder url = new StringBuilder(this.baseUrl);
            url.append("/sandboxapi/open-file-dialog");

            AtomicBoolean first = new AtomicBoolean(true);
            if (title != null) {
                addQueryArg(url, first, "title", title);
            }
            if (acceptLabel != null) {
                addQueryArg(url, first, "accept_label", acceptLabel);
            }
            addQueryArg(url, first, "save", Boolean.toString(save));
            addQueryArg(url, first, "multiple", Boolean.toString(multiple));
            addQueryArg(url, first, "directory", Boolean.toString(directory));
            if (filters != null) {
                for (String filter : filters) {
                    addQueryArg(url, first, "filter", filter);
                }
            }
            if (defaultLocation != null) {
                addQueryArg(url, first, "default_location", defaultLocation);
            }

            conn = (HttpURLConnection) new URL(url.toString()).openConnection();
            conn.setRequestProperty("Authorization", this.sandboxApiAuthorization);
            conn.setRequestMethod("GET");
            int responseCode = conn.getResponseCode();
            if (responseCode != HttpURLConnection.HTTP_OK) {
                System.err.println("Unable to use sandboxapi openFileDialog, got error code " + responseCode);
                return null;
            }

            List<Path> paths = new ArrayList<>();
            String result = SandboxAgent.readInputStreamToString(conn.getInputStream());
            for (String line : result.split("\n")) {
                String path = line.trim();
                if (!path.isEmpty()) {
                    paths.add(Paths.get(path));
                }
            }
            return paths;
        } catch (IOException e) {
            e.printStackTrace();
        } finally {
            if (conn != null) {
                conn.disconnect();
            }
        }
        return null;
    }

    private static void addQueryArg(StringBuilder url, AtomicBoolean first, String key, String value) throws UnsupportedEncodingException  {
        if (first.compareAndSet(true, false)) {
            url.append('?');
        } else {
            url.append('&');
        }
        url.append(URLEncoder.encode(key, StandardCharsets.UTF_8.toString()));
        url.append('=');
        url.append(URLEncoder.encode(value, StandardCharsets.UTF_8.toString()));
    }

}
