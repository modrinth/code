import com.github.jengelman.gradle.plugins.shadow.tasks.ShadowCopyAction
import com.github.jengelman.gradle.plugins.shadow.transformers.CacheableTransformer
import com.github.jengelman.gradle.plugins.shadow.transformers.ResourceTransformer
import com.github.jengelman.gradle.plugins.shadow.transformers.TransformerContext
import org.apache.tools.zip.ZipEntry
import org.apache.tools.zip.ZipOutputStream
import java.io.IOException
import java.util.jar.JarFile
import java.util.jar.Attributes as JarAttributes
import java.util.jar.Manifest as JarManifest

plugins {
    java
    id("com.diffplug.spotless") version "7.0.4"
    id("com.gradleup.shadow") version "9.0.0-rc2"
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.ow2.asm:asm:9.8")
    implementation("org.ow2.asm:asm-tree:9.8")
    implementation("com.google.code.gson:gson:2.13.1")

    testImplementation(libs.junit.jupiter)
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(11)
    }
}

tasks.withType<JavaCompile>().configureEach {
    options.release = 8
    options.compilerArgs.addAll(listOf("-Xlint:all", "-Werror"))
}

spotless {
    java {
        palantirJavaFormat()
        removeUnusedImports()
    }
}

tasks.jar {
    enabled = false
}

tasks.shadowJar {
    archiveFileName = "theseus.jar"
    manifest {
        attributes["Premain-Class"] = "com.modrinth.theseus.agent.TheseusAgent"
    }

    enableRelocation = true
    relocationPrefix = "com.modrinth.theseus.shadow"

    // Adapted from ManifestResourceTransformer to do one thing: remove Multi-Release.
    // Multi-Release gets added by shadow because gson has Multi-Release set to true, however
    // shadow strips the actual versions directory, as gson only has a module-info.class in there.
    // However, older versions of SecureJarHandler crash if Multi-Release is set to true but the
    // versions directory is missing.
    transform(@CacheableTransformer object : ResourceTransformer {
        private var manifestDiscovered = false
        private var manifest: JarManifest? = null

        override fun canTransformResource(element: FileTreeElement): Boolean {
            return JarFile.MANIFEST_NAME.equals(element.path, ignoreCase = true)
        }

        override fun transform(context: TransformerContext) {
            if (!manifestDiscovered) {
                try {
                    manifest = JarManifest(context.inputStream)
                    manifestDiscovered = true
                } catch (e: IOException) {
                    logger.warn("Failed to read MANIFEST.MF", e)
                }
            }
        }

        override fun hasTransformedResource(): Boolean = true

        override fun modifyOutputStream(
            os: ZipOutputStream,
            preserveFileTimestamps: Boolean
        ) {
            // If we didn't find a manifest, then let's create one.
            if (manifest == null) {
                manifest = JarManifest()
            }

            manifest!!.mainAttributes.remove(JarAttributes.Name.MULTI_RELEASE)

            os.putNextEntry(ZipEntry(JarFile.MANIFEST_NAME).apply {
                time = ShadowCopyAction.CONSTANT_TIME_FOR_ZIP_ENTRIES
            })
            manifest!!.write(os)
        }
    })
}

tasks.named<Test>("test") {
    useJUnitPlatform()
}

tasks.withType<AbstractArchiveTask>().configureEach {
    isPreserveFileTimestamps = false
    isReproducibleFileOrder = true
}
