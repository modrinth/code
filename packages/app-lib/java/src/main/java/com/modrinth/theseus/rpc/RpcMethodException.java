package com.modrinth.theseus.rpc;

public class RpcMethodException extends RuntimeException {
    private static final long serialVersionUID = 1922360184188807964L;

    public RpcMethodException(String message) {
        super(message);
    }
}
