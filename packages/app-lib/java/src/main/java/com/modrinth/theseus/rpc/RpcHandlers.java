package com.modrinth.theseus.rpc;

import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import java.util.HashMap;
import java.util.Map;
import java.util.function.BiConsumer;
import java.util.function.Function;

public class RpcHandlers {
    private final Map<String, Function<JsonElement[], JsonElement>> handlers = new HashMap<>();
    private boolean frozen;

    public RpcHandlers handler(String functionName, Runnable handler) {
        return addHandler(functionName, args -> {
            handler.run();
            return JsonNull.INSTANCE;
        });
    }

    public <A, B> RpcHandlers handler(
            String functionName, Class<A> arg1Type, Class<B> arg2Type, BiConsumer<A, B> handler) {
        return addHandler(functionName, args -> {
            if (args.length != 2) {
                throw new IllegalArgumentException(functionName + " expected 2 arguments");
            }
            final A arg1 = TheseusRpc.GSON.fromJson(args[0], arg1Type);
            final B arg2 = TheseusRpc.GSON.fromJson(args[1], arg2Type);
            handler.accept(arg1, arg2);
            return JsonNull.INSTANCE;
        });
    }

    private RpcHandlers addHandler(String functionName, Function<JsonElement[], JsonElement> handler) {
        if (frozen) {
            throw new IllegalStateException("Cannot add handler to frozen RpcHandlers instance");
        }
        handlers.put(functionName, handler);
        return this;
    }

    Map<String, Function<JsonElement[], JsonElement>> build() {
        frozen = true;
        return handlers;
    }
}
