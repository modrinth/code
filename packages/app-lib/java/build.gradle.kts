plugins {
  java
}

repositories {
  mavenCentral()
}

dependencies {
  testImplementation(libs.junit.jupiter)
  testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

java {
  toolchain {
    languageVersion = JavaLanguageVersion.of(8)
  }
}

tasks.withType<JavaCompile>().configureEach {
  options.compilerArgs.addAll(listOf("-Xlint:all", "-Werror"))
}

tasks.jar {
  archiveFileName = "theseus.jar"
}

tasks.named<Test>("test") {
  useJUnitPlatform()
}

tasks.withType<AbstractArchiveTask>().configureEach {
  isPreserveFileTimestamps = false
  isReproducibleFileOrder = true
}
