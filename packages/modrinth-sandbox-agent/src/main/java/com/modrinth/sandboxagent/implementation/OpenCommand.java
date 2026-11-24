package com.modrinth.sandboxagent.implementation;

import java.io.File;
import java.net.MalformedURLException;
import java.net.URI;
import java.net.URISyntaxException;
import java.net.URL;
import java.util.List;
import java.util.Locale;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Detects commands which open a URI or file using the desktop's default application
 */
public class OpenCommand {

    // At least two characters, so that drive letters aren't detected as a scheme
    private static final Pattern HAS_SCHEME = Pattern.compile("^[a-zA-Z][a-zA-Z0-9+.-]+:.*", Pattern.DOTALL);
    private static final Pattern RUNDLL32 = Pattern.compile("^\"?([^\",]+)\"?\\s*,\\s*(\\w+)\\s+(.+)$", Pattern.DOTALL);

    // Returns null if the command isn't recognized
    public static URI parse(List<String> command, File directory) {
        if (command.isEmpty()) {
            return null;
        }

        List<String> args = command.subList(1, command.size());
        switch (baseName(command.get(0))) {
            case "open":
            case "xdg-open":
            case "exo-open":
            case "gnome-open":
            case "gvfs-open":
            case "kde-open":
            case "kde-open5":
            case "kde-open6":
            case "sensible-browser":
            case "x-www-browser":
            case "www-browser":
            case "wslview":
                return parseSingle(args, directory);
            case "gio":
                return parseSubcommand(args, directory, "open");
            case "kfmclient":
                return parseSubcommand(args, directory, "exec", "openurl");
            case "rundll32":
                return parseRundll32(args, directory);
            default:
                return null;
        }
    }

    private static URI parseSingle(List<String> args, File directory) {
        if (!args.isEmpty() && args.get(0).equals("--")) {
            args = args.subList(1, args.size());
        }
        if (args.size() != 1 || args.get(0).startsWith("-")) {
            return null;
        }
        return parseTarget(args.get(0), directory);
    }

    private static URI parseSubcommand(List<String> args, File directory, String... subcommands) {
        if (args.isEmpty()) {
            return null;
        }
        for (String subcommand : subcommands) {
            if (args.get(0).equalsIgnoreCase(subcommand)) {
                return parseSingle(args.subList(1, args.size()), directory);
            }
        }
        return null;
    }

    private static URI parseRundll32(List<String> args, File directory) {
        Matcher matcher = RUNDLL32.matcher(String.join(" ", args).trim());
        if (!matcher.matches()) {
            return null;
        }

        String library = baseName(matcher.group(1).trim());
        if (library.endsWith(".dll")) {
            library = library.substring(0, library.length() - ".dll".length());
        }
        String function = matcher.group(2).toLowerCase(Locale.ROOT);

        boolean opens;
        if (library.equals("url")) {
            opens = function.equals("fileprotocolhandler") || function.equals("fileprotocolhandlera")
                    || function.equals("openurl") || function.equals("openurla");
        } else if (library.equals("shell32")) {
            opens = function.equals("shellexec_rundll") || function.equals("shellexec_rundlla")
                    || function.equals("shellexec_rundllw");
        } else {
            opens = false;
        }
        return opens ? parseTarget(matcher.group(3), directory) : null;
    }

    private static URI parseTarget(String target, File directory) {
        target = unquote(target.trim());
        if (target.isEmpty()) {
            return null;
        }

        if (hasScheme(target)) {
            try {
                return new URI(target);
            } catch (URISyntaxException ignored) {}
            try {
                // Escapes characters which aren't allowed
                URL url = new URL(target);
                return new URI(url.getProtocol(), url.getUserInfo(), url.getHost(), url.getPort(),
                        url.getPath(), url.getQuery(), url.getRef());
            } catch (URISyntaxException | MalformedURLException e) {
                return null;
            }
        }

        File file = new File(target);
        if (!file.isAbsolute() && directory != null) {
            file = new File(directory, target);
        }
        return file.getAbsoluteFile().toURI();
    }

    private static boolean hasScheme(String target) {
        return HAS_SCHEME.matcher(target).matches();
    }

    private static String unquote(String string) {
        if (string.length() >= 2 && string.startsWith("\"") && string.endsWith("\"")) {
            return string.substring(1, string.length() - 1);
        }
        return string;
    }

    private static String baseName(String path) {
        path = unquote(path.trim());
        path = path.substring(Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\')) + 1);
        path = path.toLowerCase(Locale.ROOT);
        if (path.endsWith(".exe")) {
            path = path.substring(0, path.length() - ".exe".length());
        }
        return path;
    }

}
