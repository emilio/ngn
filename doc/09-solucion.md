\chapter{Descripción de la solución propuesta}

<!--
  Se debe describir técnicamente el producto final con funcionalidades,
  capturas de pantalla, diagramas de flujo, etc.

  En este apartado se incluirá algún modo de probar el desarrollo realizado,
  bien mediante un enlace a la aplicación desplegada en algún servidor (si
  aplica al tipo de proyecto), un enlace al ejecutable o instalable (.exe,
  .apk, etc.), o si no aplica ninguna de estas, un vídeo demostrativo.
-->

En esta sección se presenta el software resultante del proyecto, se describirá
su funcionalidad, y se proveerán ejemplos de uso. Se proporcionará acceso a una
demo para Android.

# Acceso

Siendo una biblioteca de software, el producto principal está en el
repositorio, accesible tanto en [GitHub](https://github.com/emilio/ngn), como
en el [mirror personal](https://crisal.io/git/?p=ngn.git;a=summary).

La demo de Android se puede compilar con Android Studio, o descargar desde
[GitHub Releases](https://github.com/emilio/ngn/releases).

# Estructura del proyecto

A continuación se expone una visión simplificada de la estructura del proyecto:

```
├── Cargo.toml
├── doc
├── examples
│   ├── android
│   └── dbus
├── LICENSE
├── README.md
├── src
│   ├── lib.rs
│   ├── platform
│   │   ├── android
│   │   │   ├── mod.rs
│   │   │   └── src/main/java/io/crisal/ngn
│   │   │       ├── NgnListener.kt
│   │   │       └── NgnSessionProxy.java
│   │   ├── dbus
│   │   │   ├── mod.rs
│   │   │   ├── store.rs
│   │   │   └── wpa_supplicant
│   │   └── mod.rs
│   ├── protocol
│   │   ├── identity.rs
│   │   └── mod.rs
│   └── utils.rs
└── test
    ├── dbus-system-bus-mock.conf
    ├── setup-android.sh
    ├── setup.sh
    └── simple.conf
```

Todo el proyecto es parte del mismo paquete de `cargo`, definido en
`Cargo.toml`. Ahí es donde los datos básicos y dependencias están declaradas:

```toml
[package]
name = "ngn"
version = "0.1.0"
edition = "..."
license = "..."
# ...

[lib]
name = "ngn"
crate-type = ["cdylib", "lib"]

[dependencies]
tokio = { version = "1", features = ["full"] }
# ...
[target.'cfg(target_os = "android")'.dependencies]
jni = "0.21"
jni-sys = "0.3"
android_logger = "0.15"
```

También donde se declaran la estructura y dependencias de la demo de Linux:

```toml
[dev-dependencies]
gtk = { version = "0.9.6", package = "gtk4", features = ["v4_18"] }
adw = { version = "0.7.2", package = "libadwaita", features = ["v1_4"] }

[[example]]
name = "dbus"
crate-type = ["bin"]
```

La interfaz principal de la librería está en `src/lib.rs`, donde se definen...
