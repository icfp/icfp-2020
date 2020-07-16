plugins {
    kotlin("jvm") version "1.3.72"
    jacoco
}

group = "org.example"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation(kotlin("stdlib-jdk8"))

    testImplementation("org.junit.jupiter:junit-jupiter:5.6.2")
}

tasks {
    compileKotlin {
        kotlinOptions.jvmTarget = "11"
    }

    compileTestKotlin {
        kotlinOptions.jvmTarget = "11"
    }
}

tasks.test {
    finalizedBy(tasks.jacocoTestReport)

    useJUnitPlatform()
}

tasks.jacocoTestReport {
    dependsOn(tasks.test) // tests are required to run before generating the report

    reports {
        xml.isEnabled = true
        html.isEnabled = false
        csv.isEnabled = false
    }
}
