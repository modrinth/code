package com.modrinth.theseus;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.Arrays;

public final class MinecraftLaunch {
    public static void main(String[] args) throws IOException, ReflectiveOperationException {
        final String mainClass = args[0];
        final String[] gameArgs = Arrays.copyOfRange(args, 1, args.length);

        System.setProperty("modrinth.process.args", String.join("\u001f", gameArgs));
        parseInput();

        relaunch(mainClass, gameArgs);
    }

    private static void parseInput() throws IOException {
        final ByteArrayOutputStream line = new ByteArrayOutputStream();
        while (true) {
            final int b = System.in.read();
            if (b < 0) {
                throw new IllegalStateException("Stdin terminated while parsing");
            }
            if (b != '\n') {
                line.write(b);
                continue;
            }
            if (handleLine(line.toString("UTF-8"))) {
                break;
            }
            line.reset();
        }
    }

    private static boolean handleLine(String line) {
        final String[] parts = line.split("\t", 2);
        switch (parts[0]) {
            case "property": {
                final String[] keyValue = parts[1].split("\t", 2);
                System.setProperty(keyValue[0], keyValue[1]);
                return false;
            }
            case "launch":
                return true;
        }

        System.err.println("Unknown input line " + line);
        return false;
    }

    private static void relaunch(String mainClassName, String[] args) throws ReflectiveOperationException {
        final int javaVersion = getJavaVersion();
        final Class<?> mainClass = Class.forName(mainClassName);

        if (javaVersion >= 25) {
            Method mainMethod;
            try {
                mainMethod = findMainMethodJep512(mainClass);
            } catch (ReflectiveOperationException e) {
                System.err.println(
                        "[MODRINTH] Unable to call JDK findMainMethod. Falling back to pre-Java 25 main method finding.");
                // If the above fails due to JDK implementation details changing
                try {
                    mainMethod = findSimpleMainMethod(mainClass);
                } catch (ReflectiveOperationException innerE) {
                    e.addSuppressed(innerE);
                    throw e;
                }
            }
            if (mainMethod == null) {
                throw new IllegalArgumentException("Could not find main() method");
            }

            Object thisObject = null;
            if (!Modifier.isStatic(mainMethod.getModifiers())) {
                thisObject = mainClass.getDeclaredConstructor().newInstance();
            }

            final Object[] parameters = mainMethod.getParameterCount() > 0 ? new Object[] {args} : new Object[] {};

            mainMethod.invoke(thisObject, parameters);
        } else {
            findSimpleMainMethod(mainClass).invoke(null, new Object[] {args});
        }
    }

    private static int getJavaVersion() {
        String javaVersion = System.getProperty("java.version");

        final int dotIndex = javaVersion.indexOf('.');
        if (dotIndex != -1) {
            javaVersion = javaVersion.substring(0, dotIndex);
        }

        final int dashIndex = javaVersion.indexOf('-');
        if (dashIndex != -1) {
            javaVersion = javaVersion.substring(0, dashIndex);
        }

        return Integer.parseInt(javaVersion);
    }

    private static Method findMainMethodJep512(Class<?> mainClass) throws ReflectiveOperationException {
        // BEWARE BELOW: This code may break if JDK internals to find the main method
        // change.
        final Class<?> methodFinderClass = Class.forName("jdk.internal.misc.MethodFinder");
        final Method methodFinderMethod = methodFinderClass.getDeclaredMethod("findMainMethod", Class.class);
        final Object result = methodFinderMethod.invoke(null, mainClass);
        return (Method) result;
    }

    private static Method findSimpleMainMethod(Class<?> mainClass) throws NoSuchMethodException {
        return mainClass.getMethod("main", String[].class);
    }
}
