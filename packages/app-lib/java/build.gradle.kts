plugins {
    java
    id("com.diffplug.spotless") version "8.0.0"
    id("com.gradleup.shadow") version "9.2.2"
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.ow2.asm:asm:9.9")
    implementation("org.ow2.asm:asm-tree:9.9")
    implementation("com.google.code.gson:gson:2.13.2")

    testImplementation(libs.junit.jupiter)
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(17)
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

    addMultiReleaseAttribute = false
    enableAutoRelocation = true
    relocationPrefix = "com.modrinth.theseus.shadow"
}

tasks.named<Test>("test") {
    useJUnitPlatform()
}

tasks.withType<AbstractArchiveTask>().configureEach {
    isPreserveFileTimestamps = false
    isReproducibleFileOrder = true
}
