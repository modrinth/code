package com.modrinth.theseus.agent;

// Must be kept up-to-date with quick_play_version.rs
public enum QuickPlaySingleplayerVersion {
    BUILTIN,
    IN_START_GAME;

    public static final QuickPlaySingleplayerVersion CURRENT =
            valueOf(System.getProperty("modrinth.internal.quickPlaySingleplayerVersion"));
}
