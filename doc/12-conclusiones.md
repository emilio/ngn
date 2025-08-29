\chapter{Conclusiones y trabajo futuro}

<!--
  En esta sección se debe poner de manifiesto claramente si se han alcanzado
  todos los objetivos planteados y si se han desarrollado éstos
  satisfactoriamente, proponiendo ideas, soluciones o incluso nuevos objetivos
  surgidos a raíz de los anteriores.

  El alumno tiene aquí la posibilidad de reflexionar sobre los problemas que ha
  encontrado y cómo los ha solucionado, los errores cometidos, y lo que el TFG
  ha supuesto en cuanto a aprendizaje y experiencia, tanto profesional como
  personalmente. En este punto hay que indicar también qué nuevos conocimientos
  y tecnologías han sido necesarios aprender para poder realizarlo y el dominio
  alcanzado con ello.

  Por último, se debe incluir una descripción acerca de las limitaciones del
  trabajo y las posibles líneas de desarrollo que se abren para mejorarlo en
  cuanto a eficiencia o funcionalidad.
-->


# Cumplimiento de los objetivos

## Objetivos del sistema

Los objetivos del sistema planteados en el \cref{chap:system-objectives} se han
cumplido, con la excepción de la formación de múltiples grupos lógicos en el
mismo grupo físico.

Este último es relativamente trivial de implementar tanto como parte de la
biblioteca como por encima, y no se ha realizado por falta de tiempo, que se
empleado tanto en implementar el objetivo opcional de abstraer varias
plataformas y resolver otras complicaciones que se discuten a continuación.

Se ha cumplido adicionalmente el objetivo opcional de abstraer varias
plataformas y sistemas operativos. Esto fue básicamente un imperativo, ya que
desarrollar en Linux la mayoría del protocolo permitió ignorar muchas de las
complicaciones de testear esta tecnología en Android (como tener acceso a
múltiples dispositivos y depurar remotamente).

## Requisitos no funcionales

Los requisitos no funcionales descritos en el \cref{sec:nfr} son algo más
complicado de evaluar objetivamente, pero considero que se han cumplido:

### Portabilidad

La biblioteca funciona en Linux y Android, sistemas operativos con modelos de
seguridad y limitaciones muy diferentes.

El código es mayoritariamente multi-plataforma, y el código que es específico a
cada plataforma está confinado a `src/platform`.

La extensibilidad a otras plataformas depende de su capacidad de soportar la
capa de transporte implementada, o de soportar capas de transporte
alternativas. Parece que en este sentido el futuro puede ser más brillante que
el presente, ya que dada la discusión en el \cref{subsec:transport} es posible
que en un futuro no muy lejano todas las plataformas soporten el estándar WiFi
Aware.

### Extensibilidad

El código se ha diseñado para que la librería no exponga detalles inherentes a
la capa de transporte, usando identificadores opacos (ver
\cref{subsec:handles}). No se espera que extender la biblioteca para utilizar
algo como Bluetooth como capa de transporte o incluso modos mixtos sea
problemático.

### Seguridad

Con la advertencia necesaria de que el autor no es un experto en seguridad y
criptografía (y por lo tanto uso extenso de esta biblioteca necesitaría una
auditoría externa), se han utilizado primitivas criptográficas sólidas, y una
arquitectura que se corresponde a la que usan protocolos de paso de mensajes en
producción.

### Accesibilidad

La complejidad de este TFG reside en que funcione en dispositivos de consumo, y
se ha conseguido sin lugar a dudas, dentro de las limitaciones que las
diferentes plataformas imponen.

# Problemas encontrados

Muchos de los problemas encontrados han sido ya discutidos en el
\cref{sec:restrictions}. Las limitaciones y diseño de las diferentes
plataformas han influenciado el desarrollo de manera sustantiva, y consumido
bastantes recursos de desarrollo.

Aparte de todos esos, hay otros problemas que son dignos de mención.

## Dificultad de testeo
\label{subsec:testing}

Especialmente en las fases iniciales de desarrollo, no estaba muy claro cuál
iba a ser la mejor forma de probar el código. Tras un primer prototipo de
aplicación P2P para Android, estaba claro que tener que testear en con dos
dispositivos disponibles continuamente no iba a ser factible: Android Studio
no funcionaba particularmente bien con múltiples dispositivos conectados a la
vez, y el autor no tenía un segundo dispositivo disponible de manera continua.

Esto fue lo que hizo que Linux fuera la primera plataforma que la librería
soportó (el autor disponía de manera continua dos ordenadores con tarjeta de
red inalámbrica).

Aún así, testear con dos ordenadores en paralelo no era particularmente ideal,
ya que es necesario tener el entorno de desarrollo en ambos, y asegurarse de
que el proyecto está en el mismo estado, lo cual es propenso a errores.

El autor pensó (inocentemente) que mientras el ordenador tuviera múltiples
tarjetas de red sería posible controlarlas independientemente, y por lo tanto
adquirió una tarjeta de red extra para su escritorio. Esto resulta no ser
posible tal y como se detalla en una
[discusión](https://lists.infradead.org/pipermail/hostap/2015-September/033754.html)
en la lista de hostap.

Junto con la interacción entre `wpa_supplicant` y `NetworkManager` descrita en
el \cref{subsec:wpa-nm-interaction}, hizo un quebradero de cabeza
conseguir un flujo de desarrollo que no requiriera múltiples dispositivos. Hubo
que indagar en cómo `hostap` testea su propio código, y sólo así se llegó a la
solución actual (en `test/setup.sh`) de correr múltiples instancias de
`wpa_supplicant`, cada una controlando una interfaz creada por el driver
`mac80211_hwsim`, y un bus independiente.

Esta solución sigue sin ser ideal, porque tener que desconectar
`NetworkManager` y `wpa_supplicant` del sistema implica que si no tienes
Ethernet estás (temporalmente) offline. Una alternativa o mejora potencial
sería usar virtualización más agresivamente (via \gls{kvm} o similar), pero eso
vuelve a traer algunos de los problemas de asegurarse de que el código que
corre en la máquina virtual sea el que estás editando.

## Depuración complicada

Depurar sistemas que son dependientes de factores externos y poco deterministas
es notablemente difícil. Descubrir muchas de las interacciones problemáticas
entre componentes requirió sesiones complejas de depuración que, siendo
realistas, sólo fueron posibles gracias a rr (ver \cref{subsec:rr}).

Los scripts de testeo incluyen opciones para depurar tanto `wpa_supplicant`
como la aplicación. También compilan la aplicación con \gls{ASan}, que es útil
para cazar errores de memoria o sincronización (Rust los previene por defecto,
pero aún así [el autor es estúpido](https://github.com/emilio/ngn/commit/e24706ad567969c2a47663ab8de2e0808086a7db)).

Similarmente, fue difícil encontrar el por qué el provisionamiento de
direcciones usando direcciones IPv6 de link-local funcionaba en un ordenador y
no otro (siendo por supuesto DHCP la causa, ver \cref{subsec:dhcp}).

# Aprendizaje personal

Este proyecto ha sido guiado mayormente por la curiosidad personal del autor,
más que por la necesidad del reconocimiento académico. En ese sentido el
proyecto ha sido muy exitoso, ya que el autor ha aprendido mucho más de lo que
esperaba en un alto espectro de áreas: arquitectura y estándares de redes,
criptografía, desarrollo en Android, comunicación entre procesos via DBus.

## Contribuciones a proyectos externos
\label{subsec:external-contributions}

Una de las partes más gratificantes de este proyecto ha sido salir del
ecosistema donde el autor suele manejarse en su día a día (navegadores y
estándares web, Linux, GTK), y ver cómo la experiencia adquirida es útil y
valiosa en otros campos.

El autor ha sido capaz de hacer contribuciones técnicas a `wpa_supplicant`
(descritas en el \cref{subsec:lang-c}), dchpcd (descrita en el
\cref{subsec:dhcp}), y [zbus](https://github.com/dbus2/zbus), las cuales
incluyen [una mejora de rendimiento](https://github.com/dbus2/zbus/pull/1188),
[una mejora de rendimiento acepada pero
pendiente](https://github.com/dbus2/zbus/pull/1189) y
[una discusión sobre el generador de
código](https://github.com/dbus2/zbus/issues/1180).

# Línea de tiempo alternativa
\label{sec:alternate-timeline}

Hay una cosa en particular que, con el beneficio de mirar hacia atrás, tal
vez hubiera hecho de manera diferente (o al menos me gustaría tener una bola de
cristal para comparar la versión alternativa).

La decisión que más ha marcado tanto el tiempo como la dirección técnica del
proyecto ha sido sin lugar a dudas la de comenzar por una abstracción de la
capa de transporte en Linux (ver \cref{subsec:transport-impl}). Esta fue una
decisión unilateral en contra de la sugerencia inicial de Guillermo, razonada
con que sin una capa de transporte funcional, todo el proyecto sería
[*vaporware*](https://en.wikipedia.org/wiki/Vaporware). 

Por una parte, ignorarla en su momento hubiera ahorrado una gran cantidad de
tiempo y esfuerzo que se hizo por adelantado.

![Problemas autoinfligidos (Fixing Problems --- XKCD 1739)](images/fixing-problems.png)

Por otra parte, muchas de las complicaciones que se encontraron y de la
investigación necesaria para solucionarlas, fueron las que hicieron posible que
la implementación en Android fuera relativamente rápida, y muchas de las
decisiones que se hicieron para acomodar a las APIs de `wpa_supplicant` fueron
útiles en Android.

Mi hipótesis de cómo sería esta línea de tiempo alternativa sería que la demo
de Android sería mucho más completa (tal vez persistencia, identidades
conocidas, etc), y la biblioteca expondría mucha más funcionalidad, pero o bien
no tendría una capa de transporte robusta (tal vez abusando que Android nos
proporciona la IP del GO directamente), o no sería multi-plataforma (estoy
seguro de que no me hubiera dado tiempo a solventar todos los inconvenientes
que encontré en Linux).

En cualquier caso, estoy seguro de que hubiera acabado con conocimiento mucho
más superficial de los protocolos utilizados e involucrados en mover unos y
ceros por el aire.

# Conclusión final y trabajo futuro

Hay muchas cosas que se han quedado en el tintero. Muchas son simplemente
trabajo de implementación por hacer, como implementar más capas de transporte
(Bluetooth, WiFi Aware) o plataformas (Windows siendo el candidato obvio, pero
macOS e iOS también es posible especialmente si implementan WiFi Aware, ver el
\cref{subsec:transport}), o implementar la funcionalidad de grupos lógicos para
simplificar el direccionamiento de mensajes.

Otras son más complicadas. Este tipo de tecnologías tienen un potencial
inmenso, pero desafortunadamente las plataformas móviles actualmente no
permiten los casos de uso más interesantes que permiten crear redes más
complejas. Se deja como ejercicio al lector hipotetizar si esto es por falta de
recursos o casos de uso, o intereses corporativos.

Una serie de mejoras concretas que al autor le gustaría ver en Android serían:

 * Soporte para múltiples grupos de WiFi Direct en Android: WiFi Aware permite
   el equivalente, que podría ser una alternativa.
 * Parece que las asociaciones de WiFi Direct se almacenan persistentemente a
   nivel del sistema, e indefinidamente, lo cual puede ser no ideal para casos
   de uso ad-hoc.
 * Hacer el servicio de localización no requerido para usar WiFi Direct / WiFi
   Aware.
 * Permitir conexión entre dispositivos no conocidos sin interacción de
   usuario. El autor es consciente de los problemas de seguridad y privacidad
   que esto podría conllevar, dicho eso...

Soporte para WiFi Aware en otras plataformas que no sean Android sería genial,
ya que WiFi Aware tiene mejoras significativas sobre WiFi Direct. Por ejemplo,
permite enviar algunos datos entre nodos antes del emparejamiento (podría
usarse para la identidad lógica por ejemplo), y no requiere un nodo coordinador
como el *Group Owner* de WiFi direct.

Parece que esto viene en camino en plataformas de Apple gracias a la DMA (ver
\cref{subsec:transport}). En Linux existe una implementación llamada
[OpenNAN](https://github.com/seemoo-lab/opennan), pero no parece activa o
mantenida. Implementar WiFi Aware en `wpa_supplicant` sería un proyecto
futuro particularmente interesante.
