package com.modrinth.sandboxagent;

import com.google.gson.Gson;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;

public class MinecraftVersionInfo {

    private static final Gson GSON = new Gson();

    public String id = null;
    public String name = null;
    public int dataVersion = 0;

    public MinecraftVersionInfo(String json) {
        JsonObject minecraftVersion = GSON.fromJson(json, JsonObject.class);
        if (minecraftVersion == null) {
            return;
        }

        JsonElement id = minecraftVersion.get("id");
        if (id instanceof JsonPrimitive) {
            this.id = id.getAsString();
        }

        JsonElement name = minecraftVersion.get("name");
        if (name instanceof JsonPrimitive) {
            this.name = name.getAsString();
        }

        JsonElement dataVersion = minecraftVersion.get("world_version");
        if (dataVersion instanceof JsonPrimitive) {
            this.dataVersion = dataVersion.getAsInt();
        }
    }

    public boolean isObfuscated() {
        return this.dataVersion < 4764; // Java Edition 26.1 Snapshot 1
    }

}
