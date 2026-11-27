package com.modrinth.sandboxagent.implementation;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.InputStream;
import java.io.OutputStream;

/**
 * Process which has already exited successfully, without any output
 */
public class CompletedProcess extends Process {

    private final OutputStream stdin = new ByteArrayOutputStream();
    private final InputStream stdout = new ByteArrayInputStream(new byte[0]);
    private final InputStream stderr = new ByteArrayInputStream(new byte[0]);

    @Override
    public OutputStream getOutputStream() {
        return this.stdin;
    }

    @Override
    public InputStream getInputStream() {
        return this.stdout;
    }

    @Override
    public InputStream getErrorStream() {
        return this.stderr;
    }

    @Override
    public int waitFor() {
        return 0;
    }

    @Override
    public int exitValue() {
        return 0;
    }

    @Override
    public void destroy() {}

}
