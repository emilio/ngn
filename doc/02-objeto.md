\chapter{Objeto}
\label{chap:objeto}

<!--
  En este apartado se describirán los objetivos funcionales y personales. Se
  debe presentar un objetivo principal y una serie de subobjetivos que sean
  alcanzables, razonables y medibles.

  Como recomendaciones para redactar correctamente este apartado, se deberían
  utilizar los verbos en infinitivo, para identificar de forma clara los
  resultados de cada objetivo como una acción a realizar.
-->

# Objetivos del Sistema
\label{chap:system-objectives}

El objetivo principal del trabajo es desarrollar una biblioteca que facilite la
comunicación peer-to-peer entre dispositivos, y además proporcione varias
capacidades de alto nivel como autenticación y envío de mensajes.

\noindent Los objetivos definidos son:

 * Permitir a varios dispositivos enviar mensajes entre ellos sin necesidad de
   conexión a internet.
 * Proveer una abstracción de bajo nivel sobre la tecnología física de
   comunicación, con al menos una implementación como prueba de concepto.
   Opcionalmente, la biblioteca también abstraerá diferencias entre plataformas
   y sistemas operativos.
 * Proveer una abstracción de más alto nivel que permitirá:
   * Formación de grupos lógicos dentro de un grupo físico. Opcionalmente, se
     investigará la posibilidad de que un grupo lógico abarque más de un grupo
     físico.
   * Identificación (via sistema de clave pública / privada o similar),
     independiente de la capa física.
   * Opcionalmente, enrutado de mensajes via: Broadcast / Broadcast a un grupo
     lógico / Mensaje directo entre dos nodos lógicos (identidades).
 * Desarrollar una aplicación sencilla que demuestre las capacidades de la
   biblioteca.

# Objetivos personales

Mi `$dayjob` es desarrollar el motor de renderizado web de Firefox. A pesar de
ello, mi conocimiento en las capas de redes inferiores a la capa de transporte
antes de empezar este trabajo era bastante superficial.

Similarmente, mis conocimientos de desarrollo en Android eran igualmente
superficiales: He tenido que trabajar y depurar en Android, con Java, Kotlin, y
la NDK, pero nunca he tenido que integrar algo nuevo.

Este trabajo se presentó como una buena oportunidad de ampliar el rango de mis
conocimientos en ambos campos, mientras pongo a buen uso mi conocimiento
pre-existente sobre programación de sistemas, a la vez de explorar y hacer más
fácil el uso de tecnologías con mejores características de privacidad que las
que la mayoría de la gente usa diariamente.
