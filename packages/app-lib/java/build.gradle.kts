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

tasks.jar {
  archiveFileName = "theseus.jar"
}

tasks.named<Test>("test") {
  useJUnitPlatform()
}
