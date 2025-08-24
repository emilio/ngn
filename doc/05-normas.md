\chapter{Normas y referencias}

# Métodos

<!--
  Se debe explicar la metodología a seguir desde el punto de vista técnico, de
  manera breve (si es proceso unificado, metodologías ágiles, etc.). Se pueden
  utilizar las referencias bibliográficas oportunas para no extenderse en
  exceso en los conceptos teóricos de dichas metodologías.
-->

En este apartado se explica la metodología de desarrollo software seguida, desde
un punto de vista técnico.

## Metodología ágil: Scrum
\label{subsec:scrum}

Dadas las limitaciones temporales (trabajo a tiempo completo del autor y
familia), se ha optado por usar una metodología ágil ligera (Scrum), ya que
permite una gran adaptabilidad y se centra en el progreso incremental
\cite{scrum}.

En Scrum hay tres roles definidos: el *Product Owner*, responsable de definir y
priorizar los objetivos del proyecto; el equipo de desarrollo, que se encarga
de desarrollar el producto; y el *Scrum Master*, que es responsable de facilitar
el proceso, eliminar fricciones entre los otros roles y asegurar que se siguen
las prácticas de Scrum.

La metodología se basa en intervalos cortos de trabajo llamados *sprints*, que
son períodos de tiempo fijos, generalmente de dos a cuatro semanas. El objetivo
de cada sprint es producir un incremento de producto potencialmente entregable.

Cada *sprint* comienza con una reunión de planificación donde el equipo
selecciona un conjunto de tareas del *Product Backlog* (lista de tareas
pendientes) para completar durante el sprint, y las mueve al *Sprint Backlog*
(lista de tareas pendientes del sprint).

Durante el *sprint*, el equipo trabaja en las tareas seleccionadas, y se
realizan reuniones diarias de seguimiento, conocidas como *Daily Scrum*, donde
se revisa el progreso, se identifican obstáculos y se ajusta el plan si es
necesario.

Al final de cada *sprint* se realizan dos eventos. Primero, se lleva a cabo una
reunión de revisión para demostrar el trabajo completado y recibir
retroalimentación.

### Aplicación de Scrum al TFG

En el caso de este TFG, el *Product Owner* es el tutor del TFG, Guillermo
González Talaván, mientras que el autor del TFG asume los roles de *Scrum
Master* y equipo de desarrollo.

Los sprints han tenido una duración de una semana, al final de los cuales tenía
lugar una reunión de revisión de sprint con el tutor en la que se revisaban las
tareas completadas, se recibía retroalimentación y se actualizaban los
objetivos del proyecto.

No se han llevado a cabo reuniones diarias de seguimiento, ya que el
desarrollador es el único integrante del equipo, y, por tanto, tiene una visión
holística del progreso y los obstáculos encontrados.

Para la estimación del esfuerzo de las tareas se han utilizado puntos de
historia, una unidad de medida relativa que mide el esfuerzo necesario para
completar una tarea comparándola con otras tareas del proyecto.

# Herramientas
\label{sec:tooling}

<!--
  Se describirán de forma breve las herramientas usadas para la documentación y
  para la implementación. Deberán dividirse en herramientas propias de la
  implementación, herramientas metodológicas y herramientas para la elaboración
  del TFG: Deben obviarse herramientas estándar que no tengan relevancia para
  el TFG en particular como procesadores de textos, editor de imágenes, etc.
-->

A continuación se describen las herramientas utilizadas durante el desarrollo,
y documentación del proyecto, agrupadas en cuatro categorías: herramientas de
implementación, metodológicas, de documentación y prototipado, y otras.

## Herramientas de implementación

### Lenguaje de programación: Rust
\label{subsec:lang}

Se ha elegido [Rust](https://rust-lang.org) para la implementación principal
de la biblioteca. Rust es un lenguaje de propósito general que tiene varias
propiedades muy interesantes para un proyecto de este tipo:

 * **Multi-plataforma**: Rust soporta una amplia variedad de arquitecturas y
   plataformas \cite{rust-platform-support}.

 * **Rendimiento**: Rust es un lenguaje compilado sin ningún requerimiento de
   *runtime* ni recolector de basura. Es un lenguaje que es usable para la
   programación de sistemas con baja latencia.

 * **Seguridad**: A diferencia de otros lenguajes que cumplen los requisitos
   anteriores, como C / C++, Rust es \gls{memory-safe} y \gls{thread-safe} por
   defecto, lo que evita una gran cantidad de problemas de seguridad que siguen
   plagando el ecosistema de software actual y lo hace apropiado para
   aplicaciones críticas y a gran escala \cite{google-memory-safety}
   \cite{nsa-memory-safety}, pero también ayuda durante el desarrollo (evitando
   problemas difíciles de reproducir y solventar).

El autor de este TFG además está bastante familiarizado con Rust y la
interoperabilidad con otros lenguajes (siendo este el mantenedor de varias
bibliotecas muy populares para este propósito como
[bindgen](https://github.com/rust-lang/rust-bindgen) y
[cbindgen](https://github.com/mozilla/cbindgen), por lo que tener que
interoperar con otros lenguajes para las diferentes plataformas no parecía un
gran desafío.

Se han usado otras herramientas estándar pertenecientes al
ecosistema de Rust como [cargo](https://doc.rust-lang.org/cargo/)
para la gestión de dependencias.

### Lenguaje de programación: Kotlin

Se ha elegido [Kotlin](https://kotlinlang.org/) como lenguaje para desarrollar
la demo en Android. Kotlin es un lenguaje moderno y estáticamente tipado basado
en la \gls{JVM}.

A pesar de haber otras alternativas para hacer aplicaciones móviles como
[Dart](https://dart.dev/) (via [Flutter](https://flutter.dev/)), Kotlin es el
lenguaje mayoritario y recomendado por Google a la hora de desarrollar en
Android \cite{android-kotlin}, y soluciona bastantes problemas comunes y de
ergonomía de Java \cite{kotlin-v-java}.

### Lenguaje de programación: Java

[Java](https://www.java.com) es un lenguaje de alto nivel orientado a objetos,
de propósito general. Se ha usado este lenguaje para interoperar entre Rust y
Kotlin, via la \gls{JNI}.

La \gls{JNI} está mejor documentada para Java que para Kotlin. Kotlin y Java
interoperan de forma casi transparente, lo cual lo hizo una decisión más
conveniente a la hora de exponer la biblioteca a Android.

### Lenguaje de programación: C

Durante el desarrollo, se han tenido que investigar múltiples problemas y
situaciones inesperadas relacionadas con \gls{wpa_supplicant}, y se han
enviado y aceptado varias mejoras a este software escrito en C:

 * [P2P: Provide better failure reason for group formation
 errors](https://lists.infradead.org/pipermail/hostap/2025-January/043247.html):
 Mejora el reporte de errores de wpa_supplicant.
 * [dbus: Expose P2PDevice's own device
 address](https://lists.infradead.org/pipermail/hostap/2025-May/043428.html):
 Beneficioso para evitar colisiones durante la asociación de direcciones.
 * [dbus: Expose P2P auto_join
 behavior](https://lists.infradead.org/pipermail/hostap/2025-May/043429.html):
 Permite unirse automáticamente a un grupo existente.
 * [dbus: Expose group's GO device
 address](https://lists.infradead.org/pipermail/hostap/2025-August/043695.html):
 Expone la dirección del *Group Owner* de WiFi Direct, para poder realizar el
 intercambio de claves en menos pasos.

## Herramientas metodológicas

### Depurador: rr

[rr](https://rr-project.org/) es un depurador para Linux que permite grabar la
ejecución de un proceso (y todos sus sub-procesos) y luego reproducirla de
manera determinista con poca latencia comparada a la ejecución sin un depurador
\cite{rr-paper}.

A diferencia de otros depuradores tradicionales como
[gdb](https://sourceware.org/gdb/), que solo permiten examinar el estado actual
del programa mientras se ejecuta, rr permite depurar la misma ejecución de un
programa tantas veces como sea necesario, y también realizar una depuración
"hacia atrás". Ambas características son invaluables para depurar trabajos como
este, y software con el que el autor no estaba familiarizado como
\gls{wpa_supplicant}.

El autor de este TFG también [ha
contribuido](https://github.com/rr-debugger/rr/commits?author=emilio) a este
proyecto, aunque no como parte de este trabajo.

## Herramientas para la elaboración del TFG

### Documentación: \LaTeX
\label{subsec:latex}

[\LaTeX](https://www.latex-project.org/) es \enquote{un sistema de preparación de
documentos de alta calidad. [...] Es el estándar de facto para la comunicación
y publicación de documentos científicos y
técnicos} \cite{latex}.

### Documentación: Pandoc
\label{subsec:pandoc}

[Pandoc](https://pandoc.org/) es una herramienta de software libre escrita en
[Haskell](https://www.haskell.org/) que permite convertir entre lenguajes de
marcado. En particular, se ha usado para usar \Gls{markdown} para la mayoría
del contenido de la memoria y transpilarlo a \LaTeX, para su inclusión en el
documento final.

### Documentación: Mermaid
\label{subsec:mermaid}

[Mermaid](https://mermaid.js.org/) es una herramienta de software libre para
generar diagramas a partir de texto plano. Se ha utilizado para generar los
varios diagramas incluidos en esta memoria.

### Documentación: rustdoc

[rustdoc](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html) es la
herramienta oficial de Rust para generar documentación desde el código fuente.

## Otras herramientas

### Control de versiones: Git
\label{subsec:vcs}

[Git](https://git-scm.com) es un sistema de control de versiones distribuido de
código abierto.

Es ampliamente utilizado en la industria del software para gestionar el código
fuente y colaborar en proyectos de desarrollo.

### Alojamiento de código: GitHub
\label{subsec:hosting}

[GitHub](https://github.com) es una plataforma de alojamiento de código que
utiliza Git como sistema de control de versiones.

Proporciona funcionalidades adicionales sobre Git, como la gestión de
colaboración en proyectos de software, incluyendo la posibilidad de crear
solicitudes de incorporación de cambios (en inglés, *pull requests*), gestionar
incidencias (en inglés, *issues*), wikis, y la integración con herramientas de
\gls{CI/CD}.

<!--
# Modelos

  Se deberá realizar una introducción breve a los modelos técnicos utilizados,
  siempre que sean relevantes para el TFG y no entren dentro del ámbito
  estándar de un trabajo de TFG. Como ejemplo, se pueden describir en este
  apartado modelos de Inteligencia Artificial que se empleen, modelos
  industriales como PID, modelos electrónicos, etc.

# Prototipos

  En este apartado, si lo admite la metodología, se deberán definir wireframes
  (representación visual simplificada de la estructura y el diseño de una
  página web o aplicación). Se pueden aplicar las técnicas de prototipado, así
  como otras técnicas como el uso de mock-ups (representación virtual de un
  prototipo del proyecto que una persona quiere presentar.), diagramas de
  navegación, etc.

# Métricas

  Se describirán las métricas usadas para evaluar el proyecto (test de
  usuarios, métricas de rendimiento, etc.) si se han aplicado.
-->
