---
title: Biblioteca para comunicación directa entre dispositivos basada en tecnologías P2P
subtitle: Library for direct communication between devices based on P2P technologies
author: Emilio Cobos Álvarez
date: Septiembre de 2025
institute:
 - Grado en Ingeniería Informática
 - Universidad de Salamanca
hyperrefoptions:
 - pdfusetitle
 - pdfpagemode=FullScreen
theme: Berlin
# navigation: vertical
mainfont: "NotoSans"
mainfontfallback:
 - "NotoColorEmoji:mode=harf"
---

# Introducción

Todos tenemos una radio portátil.

. . .

**¡Pero no la usamos!**

---

# Ventajas I: Privacidad y resiliencia

## Privacidad

 * Tus datos no tienen que pasar por servidores centralizados
 * Immune a censura gubernamental y judicial como puede pasar en Irán, China...
   O en España si hay Liga ⚽

## No necesita infraestructura

 * Usable en ubicaciones remotas
 * En caso de emergencia
 * O durante apagones 🫠

---

# Ventajas II: Eficiencia

\centering

\begin{figure}
\includegraphics[height=6cm, keepaspectratio, viewport=0 200pt 600pt 792pt, clip=true]{build/images/tracert-whatsapp.pdf}
\end{figure}

\centering \tiny Traceroute a web.whatsapp.com

---

# La tecnología existe!

::: incremental

 * Bluetooth/LE
 * WiFi Aware o Neighbor Awareness Networking (NAN)
 * WiFi Direct
 * Apple Wireless Device Link (AWDL)
 * Sparklink (Huawei)

:::

---

# Usos actuales

::: incremental

 * Apps de *contact tracing* durante la pandemia usaban Bluetooth LE
 * *FireChat* en las protestas de Hong Kong de 2014
 * *Nearby Share* en Android usa WiFi Direct
 * *AirDrop* usa AWDL
 * MANETS de uso militar
 * Meshtastic

:::

---

# Hipótesis

Se hipotetiza que la baja adopción de este tipo de es por:

::: incremental

 * Dificultad de desarrollo
 * Soporte para hardware variable
 * Poca interoperabilidad entre plataformas
 * Intereses económicos

:::

---

# Propuesta

Se propone crear una biblioteca para facilitar el desarrollo de aplicaciones
P2P que:

::: incremental

 * Abstraiga la capa de transporte
 * Sea multi-plataforma
 * Funcione en dispositivos de consumo
 * Soporte autenticación
 * Cifre mensajes independiente de la capa de transporte

:::

. . .

Y una aplicación demostrativa.

---

# Herramientas

::: incremental

 * Lenguajes: Rust, Java, Kotlin, C
 * Control de versiones: Git
 * UI: GTK, Jetpack Compose
 * Depuración: rr
 * Documentación: pandoc, \LaTeX, Mermaid, rustdoc

:::

---

# Metodología

Se ha elegido *Scrum* con sprints semanales como metodología de desarrollo
ágil (con algunas licencias para acomodar las restricciones existentes).

---

# Interfaz principal

```rust
pub trait P2PSession: ... {
 fn new(..., listener) -> Result<Self>;
 fn discover_peers(&self) -> Result;
 fn connect_to_peer(&self, id: PeerId) -> Result;
 fn message_peer(&self, id: PeerId, msg: &[u8]) -> Result;
}
```

# WiFi Direct

\begin{figure}
\includegraphics[height=6cm, keepaspectratio]{images/example-wifi-direct-network.png}
\end{figure}

---

# Conexión (simplificada)

\begin{figure}
\includegraphics[width=\textwidth, height=\textheight, keepaspectratio, viewport=0px 300pt 612pt 792pt, clip=true]{build/images/01-flux.pdf}
\end{figure}

---

# Seguridad

::: incremental

 * Identidad y firma de mensajes usando clave Ed25519
 * Generación de secreto usando ECDH X25519
 * Cifrado usando AES-256-GCM

:::

---

# Obstáculos I: Asignación de direcciones

::: incremental

 * Android usa IPv4 + DHCP por defecto

 * IPv6 Neighbor discovery (ICMPv6): Requiere `CAP_NET_RAW` en Linux, imposible
   en Android

 * IPv6 Link Local Address
   * Depende de la configuración del dhcp del GO
   * Android no expone la dirección MAC de la interfaz
   * Linux no expone la MAC del GO

:::

---

# Obstáculos II: Linux

::: incremental

 * Permisos necesarios para interactuar con `wpa_supplicant`
 * Interacción entre `NetworkManager` y `wpa_supplicant` (issue reportada)
 * API de D-Bus de `wpa_supplicant` subóptima:
   * Gestión de errores pobre (fix enviado y aceptado ✅)
   * No soporta auto-join (fix enviado y aceptado ✅)
   * No expone la MAC del dispositivo propio (fix enviado, pendiente)
   * No expone la MAC de la interfaz del GO (fix enviado, pendiente)
 * Configuración de dhcp (issue reportada y arreglada por upstream ✅)
 * Mejoras de rendimiento en zbus aceptadas ✅

:::

---

# Obstáculos III: Android

::: incremental

 * Excesivos permisos necesarios
 * Interacción de usuario requerida
 * Soporte sólo para un grupo físico
 * Servicios de ubicación activados necesario
 * No expone MAC del dispositivo propio
 * No expone MAC de la interfaz propia ni del GO
 * Grupos previos almacenados global e indefinidamente

:::

---

# Obstáculos IV: Pruebas

::: incremental

 * Imposible testear en un emulador Android
 * Testear en Linux requiere:
   * Desconectar `NetworkManager`
   * Desconectar `wpa_supplicant`
   * Una instancia de `wpa_supplicant`, `dbus-daemon`, y `mac80211_hwsim` por
     cada nodo a controlar

:::

---

# Demostración: Juego multijugador off-line

\begin{figure}
\includegraphics[height=6cm, keepaspectratio]{images/app-screenshot-05-game.jpg}
\end{figure}

---

# Conclusiones I

::: incremental

 * Creo que hay hueco / demanda para una biblioteca como la propuesta, si bien
   requiere mucho más trabajo de implementación (Windows, Bluetooth, WiFi Aware...)
 * Hay mucho por hacer a nivel de plataforma e interoperabilidad también
 * La presión regulatoria de la DMA puede mejorar la situación

:::

---

# Conclusiones II

\begin{figure}
\includegraphics[height=6cm, keepaspectratio]{images/dependency.png}
\end{figure}

\centering \tiny XKCD #2347: Dependency

<!--

 * Mi proyecto destaca por XXX
 * Diagrama grupo físico / lógico
 * Demostración: Mostrar mejor los móviles, video fallback

-->
