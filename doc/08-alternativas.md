\chapter{Estudio de alternativas y viabilidad}

<!--
  En este apartado se deberá analizar brevemente por qué se usan las
  herramientas que se han descrito frente a otras existentes, incluyendo APIs,
  lenguajes de programación, paquetes, etc.

  El mensaje final que se debe transmitir en esta sección es que el autor del
  TFG ha sabido evaluar alternativas y ha elegido una solución adecuada o que,
  al menos, cumple una serie de restricciones.

  También se incluirá un estudio cualitativo sobre posibles formas de
  monetización y beneficios teniendo en cuenta el coste total del desarrollo.
-->

En este capítulo se detallan las alternativas consideradas al tomar varias
decisiones, tanto técnicas como no. Finalmente se realiza un pequeño comentario
sobre la viabilidad del proyecto.

# Alternativas consideradas

## Lenguaje de programación

La elección de Rust ha sido explicada en el \cref{subsec:lang}.

Se consideró C y C++ como lenguajes con similar rendimiento a Rust en tiempo de
ejecución y buena integración con otras plataformas, pero varias
características de Rust decantaron la balanza:

 * Gestión de paquetes: Rust tiene un [ecosistema de paquetes
   enorme](https://crates.io), y un gestor de paquetes integrado. Adquirir
   dependencias complejas con C y C++, especialmente cuando estás compilando para
   diferentes plataformas, es mucho más complejo.

 * Sistema de tipos: El sistema de tipos de Rust es muy flexible, y previene
   muchos errores comunes en C y C++. El [soporte para programación
   asíncrona](https://rust-lang.github.io/async-book/) en el lenguaje es muy
   útil y conveniente cuando se programa código de red, que por naturaleza es
   asíncrono.

 * Características de seguridad: Saber que tu código no tiene data races no
   tiene precio y ayuda con la depuración inmensamente.

 * Compilación cruzada fácil: Rust provee *toolchains* pre-compiladas para una
   gran variedad de plataformas, y compilar para otra plataforma es tan
   sencillo como hacer `cargo build --target=x86_64-linux-android`.

Se consideraron otras alternativa a estos tres lenguajes también:

 * [Zig](https://ziglang.org/): A pesar de que tiene funcionalidad muy
   interesante, [sobre todo con respecto a la
   metaprogramación](https://zig.guide/language-basics/comptime/), no tiene las
   características de seguridad de Rust, y el autor no conoce el lenguaje tan
   profundamente, lo que hubiera supuesto un desafío extra.

 * [Java](https://java.com/): Depender de la \Gls{JVM} dificulta usarlo en
   plataformas como iOS, ya que Apple no permite la generación de código
   dinámico en esa plataforma \cite{ios-dynamic-code}. OpenJDK tenía un
   [proyecto para soportar iOS](https://openjdk.org/projects/mobile/ios.html),
   pero se quedó en un experimento en JDK 9 y parece estar muerto.

 * [Kotlin](https://kotlinlang.org/): Tiene la misma restricción que Java, pero
   [Kotlin Native](https://kotlinlang.org/docs/native-overview.html) es una
   alternativa más reciente. En cualquier caso, ese tipo de problemas lo hacen
   más complejo que usar un lenguaje completamente compilado como C, C++, o
   Rust.

## Capa de transporte

Para la capa de transporte, sólo Bluetooth y WiFi Direct son alternativas
estándar en hardware de consumo. Bluetooth está disponible también en
plataformas Apple, pero tiene un rango menor.

Alternativas no estándar incluyen \Gls{AWDL} de Apple y
[Sparklink](https://www.sparklink.org.cn/) de Huawei. Dado que el desarrollo no
iba a poder realizarse en dichas plataformas de todas maneras (porque el autor
no tiene acceso a ellas), se eligió WiFi Direct como la plataforma más
prometedora.

WiFi Aware, también conocido como *Neighbor Awareness Networking*
\cite{wifi-aware} es la siguiente iteración de la WiFi Alliance sobre WiFi
Direct, y [funciona en
Android](https://developer.android.com/develop/connectivity/wifi/wifi-aware),
pero no en ninguna otra plataforma.

Dado el panorama, se eligió WiFi direct porque sus características (como rango)
eran más favorables que Bluetooth, y permitía ser implementado en múltiples
plataformas.

Parece que la Unión Europea, via \Gls{DMA}, está presionando a Apple para abrir AWDL
e implementar WiFi Aware \cite{dma-wifi-case} \cite{dma-wifi-proposed-measures}
(sec 2.1).

> Apple shall provide effective interoperability with the high-bandwidth
> peer-to-peer ("P2P") Wi-Fi connection feature.
>
> [...]
>
> Apple shall:
>  * Implement Wi-Fi Aware in its iOS devices and iOS in accordance with the
>    Wi-Fi Aware specification.
>
>  * Allow third-party iOS app developers to establish a Wi-Fi Aware connection
>    between an iOS device and any third-party connected physical device that
>    supports Wi-Fi Aware.

Y parece que Apple está implementando [WiFi
Aware](https://developer.apple.com/documentation/WiFiAware) (actualmente en iOS
Beta). Así que tal vez en un futuro no tan lejano los consumidores tengan el
futuro que se merecen con WiFi Aware...

## Control de versiones y alojamiento

Para el control de versiones del proyecto se ha utilizado Git (ver
\cref{subsec:vcs}) y para el alojamiento del código GitHub (ver
\cref{subsec:hosting}) y un
[mirror](https://crisal.io/git/?p=ngn.git;a=summary) propio para compartir
progreso con el tutor (ya que el repositorio era inicialmente privado).

El autor conoce también [Mercurial](https://www.mercurial-scm.org/), y de hecho
considera a Mercurial superior en muchos aspectos a Git. Sin embargo, Git es
mucho más utilizado en la industria \cite{stack-overflow-vcs}, el autor también
lo conoce en profundidad, y existen herramientas como
[Jujutsu](https://jj-vcs.github.io/jj/latest/) inspiradas en la experiencia de
usuario de Mercurial. Eso, junto a la existencia de servicios para alojar el
código como GitHub, hicieron a Git la elección simple.

Tampoco se ha planteado no utilizar un sistema de control de versiones, ya que
es una herramienta esencial para el desarrollo de software, y permite llevar un
seguimiento de los cambios realizados en el código, así como colaborar con
otros desarrolladores de forma más eficiente en un futuro.
