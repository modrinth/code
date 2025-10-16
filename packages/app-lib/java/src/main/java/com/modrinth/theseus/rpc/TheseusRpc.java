package com.modrinth.theseus.rpc;

import com.google.gson.*;
import com.google.gson.reflect.TypeToken;
import java.io.*;
import java.net.Socket;
import java.nio.charset.StandardCharsets;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.BlockingQueue;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Function;

public final class TheseusRpc {
    static final Gson GSON = new GsonBuilder()
            .setStrictness(Strictness.STRICT)
            .setFieldNamingPolicy(FieldNamingPolicy.LOWER_CASE_WITH_UNDERSCORES)
            .disableHtmlEscaping()
            .create();
    private static final TypeToken<RpcMessage> MESSAGE_TYPE = TypeToken.get(RpcMessage.class);

    private static final AtomicReference<TheseusRpc> RPC = new AtomicReference<>();

    private final BlockingQueue<RpcMessage> mainThreadQueue = new LinkedBlockingQueue<>();
    private final Map<UUID, ResponseWaiter<?>> awaitingResponse = new ConcurrentHashMap<>();
    private final Map<String, Function<JsonElement[], JsonElement>> handlers;
    private final Socket socket;

    private TheseusRpc(Socket socket, RpcHandlers handlers) {
        this.socket = socket;
        this.handlers = handlers.build();
    }

    public static void connectAndStart(String host, int port, RpcHandlers handlers) throws IOException {
        if (RPC.get() != null) {
            throw new IllegalStateException("Can only connect to RPC once");
        }

        final Socket socket = new Socket(host, port);
        final TheseusRpc rpc = new TheseusRpc(socket, handlers);
        final Thread mainThread = new Thread(rpc::mainThread, "Theseus RPC Main");
        final Thread readThread = new Thread(rpc::readThread, "Theseus RPC Read");
        mainThread.setDaemon(true);
        readThread.setDaemon(true);
        mainThread.start();
        readThread.start();
        RPC.set(rpc);
    }

    public static TheseusRpc getRpc() {
        final TheseusRpc rpc = RPC.get();
        if (rpc == null) {
            throw new IllegalStateException("Called getRpc before RPC initialized");
        }
        return rpc;
    }

    public <T> CompletableFuture<T> callMethod(TypeToken<T> returnType, String method, Object... args) {
        final JsonElement[] jsonArgs = new JsonElement[args.length];
        for (int i = 0; i < args.length; i++) {
            jsonArgs[i] = GSON.toJsonTree(args[i]);
        }

        final RpcMessage message = new RpcMessage(method, jsonArgs);
        final ResponseWaiter<T> responseWaiter = new ResponseWaiter<>(returnType);
        awaitingResponse.put(message.id, responseWaiter);
        mainThreadQueue.add(message);
        return responseWaiter.future;
    }

    private void mainThread() {
        try {
            final Writer writer = new OutputStreamWriter(socket.getOutputStream(), StandardCharsets.UTF_8);
            while (true) {
                final RpcMessage message = mainThreadQueue.take();
                final RpcMessage toSend;
                if (message.isForSending) {
                    toSend = message;
                } else {
                    final Function<JsonElement[], JsonElement> handler = handlers.get(message.method);
                    if (handler == null) {
                        System.err.println("Unknown theseus RPC method " + message.method);
                        continue;
                    }
                    RpcMessage response;
                    try {
                        response = new RpcMessage(message.id, handler.apply(message.args));
                    } catch (Exception e) {
                        response = new RpcMessage(message.id, e.toString());
                    }
                    toSend = response;
                }
                GSON.toJson(toSend, writer);
                writer.write('\n');
                writer.flush();
            }
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        } catch (InterruptedException ignored) {
        }
    }

    private void readThread() {
        try {
            final BufferedReader reader =
                    new BufferedReader(new InputStreamReader(socket.getInputStream(), StandardCharsets.UTF_8));
            while (true) {
                final RpcMessage message = GSON.fromJson(reader.readLine(), MESSAGE_TYPE);
                if (message.method == null) {
                    final ResponseWaiter<?> waiter = awaitingResponse.get(message.id);
                    if (waiter != null) {
                        handleResponse(waiter, message);
                    }
                } else {
                    mainThreadQueue.put(message);
                }
            }
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        } catch (InterruptedException ignored) {
        }
    }

    private <T> void handleResponse(ResponseWaiter<T> waiter, RpcMessage message) {
        if (message.error != null) {
            waiter.future.completeExceptionally(new RpcMethodException(message.error));
            return;
        }
        try {
            waiter.future.complete(GSON.fromJson(message.response, waiter.type));
        } catch (JsonSyntaxException e) {
            waiter.future.completeExceptionally(e);
        }
    }

    private static class RpcMessage {
        final UUID id;
        final String method; // Optional
        final JsonElement[] args; // Optional
        final JsonElement response; // Optional
        final String error; // Optional
        final transient boolean isForSending;

        RpcMessage(String method, JsonElement[] args) {
            id = UUID.randomUUID();
            this.method = method;
            this.args = args;
            response = null;
            error = null;
            isForSending = true;
        }

        RpcMessage(UUID id, JsonElement response) {
            this.id = id;
            method = null;
            args = null;
            this.response = response;
            error = null;
            isForSending = true;
        }

        RpcMessage(UUID id, String error) {
            this.id = id;
            method = null;
            args = null;
            response = null;
            this.error = error;
            isForSending = true;
        }
    }

    private static class ResponseWaiter<T> {
        final TypeToken<T> type;
        final CompletableFuture<T> future = new CompletableFuture<>();

        ResponseWaiter(TypeToken<T> type) {
            this.type = type;
        }
    }
}
