package com.modrinth.theseus;

import com.modrinth.theseus.rpc.RpcHandlers;
import com.modrinth.theseus.rpc.TheseusRpc;
import java.io.IOException;
import java.lang.reflect.AccessibleObject;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.Arrays;
import java.util.concurrent.CompletableFuture;

public final class MinecraftLaunch {
    public static void main(String[] args) throws IOException, ReflectiveOperationException {
        final String mainClass = args[0];
        final String[] gameArgs = Arrays.copyOfRange(args, 1, args.length);

        System.setProperty("modrinth.process.args", String.join("\u001f", gameArgs));

        final CompletableFuture<Void> waitForLaunch = new CompletableFuture<>();
        TheseusRpc.connectAndStart(
                System.getProperty("modrinth.internal.ipc.host"),
                Integer.getInteger("modrinth.internal.ipc.port"),
                new RpcHandlers()
                        .handler("set_system_property", String.class, String.class, System::setProperty)
                        .handler("launch", () -> waitForLaunch.complete(null)));

        waitForLaunch.join();
        relaunch(mainClass, gameArgs);
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
                thisObject = forceAccessible(mainClass.getDeclaredConstructor()).newInstance();
            }

            final Object[] parameters = mainMethod.getParameterCount() > 0 ? new Object[] {args} : new Object[] {};

            mainMethod.invoke(thisObject, parameters);
        } else {
            forceAccessible(findSimpleMainMethod(mainClass)).invoke(null, new Object[] {args});
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

    private static <T extends AccessibleObject> T forceAccessible(T object) throws ReflectiveOperationException {
        try {
            final Method setAccessible0 = AccessibleObject.class.getDeclaredMethod("setAccessible0", boolean.class);
            setAccessible0.setAccessible(true);
            setAccessible0.invoke(object, true);
        } catch (NoSuchMethodException e) {
            object.setAccessible(true);
        }
        return object;
    }
}
