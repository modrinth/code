package com.modrinth.theseus.agent;

// Must be kept up-to-date with quick_play_version.rs
public enum QuickPlayServerVersion {
    BUILTIN,
    BUILTIN_LEGACY,
    INJECTED;

    public static final QuickPlayServerVersion CURRENT =
            valueOf(System.getProperty("modrinth.internal.quickPlayServerVersion"));
}
